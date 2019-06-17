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

pub struct ConnectionMesh {
    //all connections are in here and are referenced in the other two maps
    pub connections: std::collections::HashMap<ConnectionId, Box<Connection>>,

    //connections_in.get(conn_id).unwrap() is ID of the module that receives the message
    pub connections_in: std::collections::HashMap<ConnectionId, ModuleId>,
    //connections_out.get(mod_id).unwrap().get(port_id).unwrap() is ID of the connection to push the message to
    pub connections_out:
        std::collections::HashMap<ModuleId, std::collections::HashMap<PortId, ConnectionId>>,

    pub messages: std::collections::BinaryHeap<TimedMessage>,
}

impl ConnectionMesh {
    pub fn connect_modules(
        &mut self,
        conn: Box<Connection>,
        mod_out: ModuleId,
        out_port: PortId,
        mod_in: ModuleId,
    ) -> Result<(), Box<std::error::Error>> {
        let mod_ports = self.connections_out.get_mut(&mod_out);
        match mod_ports {
            None => panic!("Module: {} has no ports table registered?", mod_out),
            Some(ports) => match ports.get_mut(&out_port) {
                Some(_) => panic!(
                    "Tried to set port: {} on module: {} that was already set",
                    mod_out, out_port
                ),
                None => {
                    ports.insert(out_port, conn.connection_id());
                    self.connections_in.insert(conn.connection_id(), mod_in);
                    self.connections.insert(conn.connection_id(), conn);
                }
            },
        }

        Ok(())
    }

    pub fn get_ports(&mut self, mod_id: ModuleId) -> Option<Vec<PortId>> {
        match self.connections_out.get(&mod_id) {
            Some(ports) => Some(ports.keys().map(|key_ref| *key_ref).collect()),
            None => None,
        }
    }

    pub fn send_message(
        &mut self,
        msg: Box<Message>,
        sender_mod_id: ModuleId,
        port: PortId,
        ctx: &mut HandleContext,
    ) {
        let conn = self
            .connections
            .get_mut(
                &self
                    .connections_out
                    .get(&sender_mod_id)
                    .unwrap()
                    .get(&port)
                    .unwrap(),
            )
            .unwrap();

        match conn.handle_message(msg, ctx) {
            Some((time, msg)) => {
                let recipient = self.connections_in.get(&conn.connection_id()).unwrap();
                self.messages.push(TimedMessage {
                    time: time,
                    msg: msg,
                    recipient: *recipient,

                });
            }
            None => {}
        }
    }
}
