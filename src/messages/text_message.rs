use crate::messages::message::Message;
use std::any::Any;
use crate::id_types::{MessageId, MessageTypeId};

pub struct TextMsg {
    pub id: MessageId,
    pub type_id: MessageTypeId,
    pub data: String,
}

impl Message for TextMsg {
    fn msg_type_id(&self) -> MessageTypeId {
        self.type_id
    }
    fn msg_id(&self) -> MessageId {
        self.id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}