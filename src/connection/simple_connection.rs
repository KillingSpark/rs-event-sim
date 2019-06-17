use crate::messages::message::{Message};
use crate::id_mngmnt::id_types::{ConnectionId, ConnectionTypeId};
use crate::connection::connection::HandleContext;

pub struct SimpleConnection {
    pub buf: Vec<(u64, Box<Message>)>,
    pub id: ConnectionId,
    pub type_id: ConnectionTypeId,

    pub delay: u64,
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