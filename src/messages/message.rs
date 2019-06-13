use std::any::Any;
use crate::id_mngmnt::id_types::MessageId;
use crate::id_mngmnt::id_types::MessageTypeId;

pub trait Message {
    fn msg_type_id(&self) -> MessageTypeId;
    fn msg_id(&self) -> MessageId;

    //To downcast use this:
    //let tev: &TextMsg = event.as_any().downcast_ref::<TextMsg>().unwrap();
    fn as_any(&self) -> &dyn Any;
}