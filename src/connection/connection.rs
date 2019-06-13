use crate::id_types::{ConnectionId, ConnectionTypeId, ModuleId, PortId};

pub trait Connection {
    fn push(&mut self, msgs: Vec<Box<crate::messages::message::Message>>) -> Result<(), Box<std::error::Error>>;
    fn pull(&mut self) -> Result<Option<Vec<Box<crate::messages::message::Message>>>, Box<std::error::Error>>;

    fn connection_id(&self) -> ConnectionId;
    fn connection_type_id(&self) -> ConnectionTypeId;
}

pub struct ConnectionMesh{
    //all connections are in here and are referenced in the other two maps
    pub connections: std::collections::HashMap<ConnectionId, Box<Connection>>,

    //connections_in.get(conn_id).unwrap() is ID of the module that receives the message
    pub connections_in: std::collections::HashMap<ConnectionId, ModuleId>,
    //connections_out.get(mod_id).unwrap().get(port_id).unwrap() is ID of the connection to push the message to
    pub connections_out: std::collections::HashMap<ModuleId, std::collections::HashMap<PortId, ConnectionId>>,
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

    pub fn get_out_connection(&mut self, mod_id: ModuleId, port: PortId) -> Option<&mut Box<Connection>>{
        match self.connections_out.get(&mod_id) {
            Some(c) => {
                match c.get(&port) {
                    Some(conn_id) => {
                        match self.connections.get_mut(conn_id) {
                            Some(conn) => Some(conn),
                            None => {None},
                        }
                    },
                    None => {None},
                }
            }
            None => {None},
        }
    }
}