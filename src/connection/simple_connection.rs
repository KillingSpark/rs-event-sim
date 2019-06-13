use crate::messages::message::Message;

pub struct SimpleConnection {
    pub buf: Vec<Box<Message>>,
    pub conn_id: u64,
    pub type_id: u64,
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

    fn connection_id(&self) -> u64 {
        self.conn_id
    }
    fn connection_type_id(&self) -> u64 {
        self.type_id
    }
}