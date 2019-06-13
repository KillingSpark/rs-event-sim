use crate::messages::message::Message;
use crate::id_mngmnt::id_types::{ConnectionId, ConnectionTypeId};

pub struct SimpleConnection {
    pub buf: Vec<Box<Message>>,
    pub id: ConnectionId,
    pub type_id: ConnectionTypeId,
}

impl crate::connection::connection::Connection for SimpleConnection {
    fn push(&mut self, msgs: Vec<Box<Message>>) -> Result<(), Box<std::error::Error>>{
        let mut lmsgs = msgs;
        self.buf.append(&mut lmsgs);
        Ok(())
    }
    fn pull(&mut self) -> Result<Option<Vec<Box<Message>>>, Box<std::error::Error>>{
        let mut lmsgs = Vec::new();
        lmsgs.append(&mut self.buf);
        Ok(Some(lmsgs))
    }

    fn connection_id(&self) -> ConnectionId {
        self.id
    }
    fn connection_type_id(&self) -> ConnectionTypeId {
        self.type_id
    }
}