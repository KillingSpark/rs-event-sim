use std::any::Any;

pub trait Message {
    fn msg_type_id(&self) -> u64;
    fn msg_id(&self) -> u64;

    //To downcast use this:
    //let tev: &TextMsg = event.as_any().downcast_ref::<TextMsg>().unwrap();
    fn as_any(&self) -> &dyn Any;
}