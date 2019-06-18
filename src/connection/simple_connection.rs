use crate::messages::message::{Message};
use crate::id_mngmnt::id_types::{ConnectionId, ConnectionTypeId};
use crate::connection::connection::HandleContext;

pub struct SimpleConnection {
    pub buf: Vec<(u64, Box<Message>)>,
    pub id: ConnectionId,
    pub type_id: ConnectionTypeId,

    pub delay: u64,
}

pub static TYPE_STR: &str = "SimpleConnection";
pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}
pub fn new_simple_connection(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar, delay: u64) -> SimpleConnection {
    SimpleConnection {
        id: id_reg.new_id(),
        type_id: *id_reg.lookup_id(TYPE_STR.to_owned()).unwrap(),
        delay: delay,
        buf: Vec::new(),
    }
}

impl crate::connection::connection::Connection for SimpleConnection {
    fn handle_message(&mut self, message: Box<Message>, ctx: &mut HandleContext) -> Option<(u64,Box<Message>)> {
        Some((
            ctx.time.now() + self.delay,
            message,
        ))
    }

    fn connection_id(&self) -> ConnectionId {
        self.id
    }
    fn connection_type_id(&self) -> ConnectionTypeId {
        self.type_id
    }
}