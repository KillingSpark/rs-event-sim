use crate::core::connection::connection::*;
use crate::core::contexts::SimulationContext;
use crate::core::id_mngmnt::id_types::{ConnectionId, GateId, ModuleId, PortId};
use crate::core::messages::message::{Message, TimedMessage};

pub enum ConnectionKind {
    Onedirectional,
    Bidrectional,
}

pub struct ConnectionMesh {
    //all connections are in here and are referenced in the other two maps
    pub connections: std::collections::HashMap<ConnectionId, Box<Connection>>,

    pub gates: std::collections::HashMap<(ModuleId, GateId, PortId), Port>,

    //messages that will be handled in the future
    pub messages: std::collections::BinaryHeap<TimedMessage>,

    //messages that will be handled in this point in time. No need to sift them through the heap and immediately pop them
    pub messages_now: std::collections::VecDeque<TimedMessage>,
}

impl ConnectionMesh {
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
            match self.gates.get(&(mod_in, gate_in, in_port)) {
                Some(_) => {
                    panic!("Tried to overwrite in-going port");
                }
                None => {}
            }

            self.gates.insert(
                (mod_in, gate_in, in_port),
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

        match self.gates.get(&(mod_out, gate_out, out_port)) {
            Some(_) => {
                panic!("Tried to overwrite out-going port");
            }
            None => {}
        }

        self.gates.insert(
            (mod_out, gate_out, out_port),
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

    pub fn send_message(
        &mut self,
        msg: Box<Message>,
        sender_mod_id: ModuleId,
        gate_id: GateId,
        port: PortId,
        ctx: &mut SimulationContext,
    ) {
        let triple = (sender_mod_id, gate_id, port);
        let out_port = match self.gates.get(&triple) {
            Some(port) => port,
            None => panic!("illegal port {}", port.0),
        };

        match out_port.kind {
            PortKind::In => {
                panic!("Tried to send message over port that is not an out-going port");
            }
            PortKind::Out => { /* OK */ }
            PortKind::InOut => { /* OK */ }
        }

        let conn = self.connections.get_mut(&out_port.conn_id).unwrap();

        match conn.handle_message(msg, ctx) {
            Some((time, msg)) => {
                if time == ctx.time.now() {
                    self.messages_now.push_back(TimedMessage {
                        time: time,
                        msg: msg,
                        recipient: out_port.rcv_mod,
                        recp_gate: out_port.rcv_gate,
                        recp_port: out_port.rcv_port,
                    });
                } else {
                    self.messages.push(TimedMessage {
                        time: time,
                        msg: msg,
                        recipient: out_port.rcv_mod,
                        recp_gate: out_port.rcv_gate,
                        recp_port: out_port.rcv_port,
                    });
                }
            }
            None => {}
        }
    }
}
