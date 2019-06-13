use crate::messages::message::Message;
use std::any::Any;

pub struct TextMsg {
    pub sig_id: u64,
    pub type_id: u64,
    pub data: String,
}

impl Message for TextMsg {
    fn msg_type_id(&self) -> u64 {
        self.type_id
    }
    fn msg_id(&self) -> u64 {
        self.sig_id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}