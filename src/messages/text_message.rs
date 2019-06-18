use crate::messages::message::Message;
use std::any::Any;
use crate::id_mngmnt::id_types::{MessageId, MessageTypeId};

pub struct TextMsg {
    pub id: MessageId,
    pub type_id: MessageTypeId,
    pub data: String,
}

pub static TYPE_STR: &str = "TextMessage";
pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}
pub fn new_text_msg(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar, data: String) -> TextMsg {
    TextMsg {
        id: id_reg.new_id(),
        type_id: *id_reg.lookup_id(TYPE_STR.to_owned()).unwrap(),
        data: data,
    }
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