use crate::connection::connection::*;
use crate::id_mngmnt::id_types::{ConnectionId, GateId, ModuleId, PortId};
use crate::messages::message::{Message, TimedMessage};

pub enum ConnectionKind {
    Onedirectional,
    Bidrectional,
}

pub struct ConnectionMesh {
    //all connections are in here and are referenced in the other two maps
    pub connections: std::collections::HashMap<ConnectionId, Box<Connection>>,

    pub gates: std::collections::HashMap<ModuleId, std::collections::HashMap<GateId, Gate>>,
    pub messages: std::collections::BinaryHeap<TimedMessage>,
}

impl ConnectionMesh {
    pub fn add_gate(&mut self, module: ModuleId, gate: GateId) {
        self.gates.get_mut(&module).unwrap().insert(
            gate,
            Gate {
                id: gate,
                ports: std::collections::HashMap::new(),
            },
        );
    }

    pub fn connect_modules(
        &mut self,
        conn: Box<Connection>,

        con_kind: ConnectionKind,

        mod_out: ModuleId,
        gate_out: GateId,
        out_port: PortId,

        mod_in: ModuleId,
        gate_in: GateId,
        in_port: PortId,
    ) -> Result<(), Box<std::error::Error>> {
        {
            let in_gate = self
                .gates
                .get_mut(&mod_in)
                .unwrap()
                .get_mut(&gate_in)
                .unwrap();
            match in_gate.ports.get(&in_port) {
                Some(_) => {
                    panic!("Tried to overwrite in-going port");
                }
                None => {}
            }

            in_gate.ports.insert(
                in_port,
                Port {
                    id: in_port,
                    conn_id: conn.connection_id(),

                    kind: match con_kind {
                        ConnectionKind::Onedirectional => PortKind::In,
                        ConnectionKind::Bidrectional => PortKind::InOut,
                    },

                    rcv_gate: gate_out,
                    rcv_mod: mod_out,
                    rcv_port: out_port,
                },
            );
        }

        let out_gate = self
            .gates
            .get_mut(&mod_out)
            .unwrap()
            .get_mut(&gate_out)
            .unwrap();
        match out_gate.ports.get(&out_port) {
            Some(_) => {
                panic!("Tried to overwrite out-going port");
            }
            None => {}
        }

        out_gate.ports.insert(
            out_port,
            Port {
                id: out_port,
                conn_id: conn.connection_id(),

                kind: match con_kind {
                    ConnectionKind::Onedirectional => PortKind::Out,
                    ConnectionKind::Bidrectional => PortKind::InOut,
                },

                rcv_gate: gate_in,
                rcv_mod: mod_in,
                rcv_port: in_port,
            },
        );

        self.connections.insert(conn.connection_id(), conn);

        Ok(())
    }

    pub fn get_ports(&mut self, mod_id: ModuleId, gate_id: GateId) -> Option<Vec<PortId>> {
        match self.gates.get(&mod_id).unwrap().get(&gate_id) {
            Some(gate) => Some(gate.ports.keys().map(|key_ref| *key_ref).collect()),
            None => None,
        }
    }

    pub fn send_message(
        &mut self,
        msg: Box<Message>,
        sender_mod_id: ModuleId,
        gate_id: GateId,
        port: PortId,
        ctx: &mut HandleContext,
    ) {
        let out_port = &self
            .gates
            .get(&sender_mod_id)
            .unwrap()
            .get(&gate_id)
            .unwrap()
            .ports
            .get(&port)
            .unwrap();

        match out_port.kind {
            PortKind::In => {
                panic!("Tried to send message over port that is not an out-going port");
            }
            PortKind::Out => {/* OK */}
            PortKind::InOut => {/* OK */}
        }

        let conn = self.connections.get_mut(&out_port.conn_id).unwrap();

        match conn.handle_message(msg, ctx) {
            Some((time, msg)) => {
                self.messages.push(TimedMessage {
                    time: time,
                    msg: msg,
                    recipient: out_port.rcv_mod,
                    recp_gate: out_port.rcv_gate,
                    recp_port: out_port.rcv_port,
                });
            }
            None => {}
        }
    }
}
