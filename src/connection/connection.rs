use crate::id_mngmnt::id_types::{ConnectionId, ConnectionTypeId, ModuleId, PortId};
use crate::messages::message::{Message, TimedMessage};

pub struct HandleContext<'a> {
    pub time: &'a crate::clock::Clock,
    pub id_reg: &'a crate::id_mngmnt::id_registrar::IdRegistrar,
}

pub trait Connection {
    fn handle_message(
        &mut self,
        message: Box<Message>,
        ctx: &mut HandleContext,
    ) -> Option<(u64, Box<Message>)>;

    fn connection_id(&self) -> ConnectionId;
    fn connection_type_id(&self) -> ConnectionTypeId;
}

pub enum PortKind {
    In,
    Out,
}

pub struct Port {
    pub id: PortId,
    pub kind: PortKind,
    pub conn_id: ConnectionId,

    pub rcv_mod: ModuleId,
    pub rcv_gate: u64,
    pub rcv_port: PortId,
}

pub struct Gate {
    id: u64,
    ports: std::collections::HashMap<PortId, Port>,
}

pub struct ConnectionMesh {
    //all connections are in here and are referenced in the other two maps
    pub connections: std::collections::HashMap<ConnectionId, Box<Connection>>,

    pub gates: std::collections::HashMap<ModuleId, std::collections::HashMap<u64, Gate>>,
    pub messages: std::collections::BinaryHeap<TimedMessage>,
}

impl ConnectionMesh {
    pub fn add_gate(&mut self, module: ModuleId, gate: u64) {
        self.gates.get_mut(&module).unwrap().insert(gate, Gate{
            id: gate,
            ports: std::collections::HashMap::new(),
        });
    }

    pub fn connect_modules(
        &mut self,
        conn: Box<Connection>,

        mod_out: ModuleId,
        gate_out: u64,
        out_port: PortId,

        mod_in: ModuleId,
        gate_in: u64,
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
                    kind: PortKind::In,

                    rcv_gate: 0,
                    rcv_mod: 0,
                    rcv_port: 0,
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
                kind: PortKind::Out,

                rcv_gate: gate_in,
                rcv_mod: mod_in,
                rcv_port: in_port,
            },
        );

        self.connections.insert(conn.connection_id(), conn);

        Ok(())
    }

    pub fn get_ports(&mut self, mod_id: ModuleId, gate_id: u64) -> Option<Vec<PortId>> {
        match self.gates.get(&mod_id).unwrap().get(&gate_id) {
            Some(gate) => Some(gate.ports.keys().map(|key_ref| *key_ref).collect()),
            None => None,
        }
    }

    pub fn send_message(
        &mut self,
        msg: Box<Message>,
        sender_mod_id: ModuleId,
        gate_id: u64,
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

        match out_port.kind  {
            PortKind::In => {
                panic!("Tried to send message over port that is not an out-going port");
            }
            PortKind::Out => {},
        }
        
        let conn = self.connections.get_mut(&out_port.conn_id).unwrap();

        match conn.handle_message(msg, ctx) {
            Some((time, msg)) => {
                self.messages.push(TimedMessage {
                    time: time,
                    msg: msg,
                    recipient: out_port.rcv_mod,
                });
            }
            None => {}
        }
    }
}
