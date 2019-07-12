use crate::event::Event;
use std::any::Any;
use crate::core::id_mngmnt::id_types::{EventsId, EventsTypeId};
use crate::core::id_mngmnt::id_registrar::IdRegistrar;

#[derive(Clone)]
pub struct TextEvent {
    pub data: String,
    pub id: EventsId,
    pub type_id: EventsTypeId,
}

pub static TYPE_STR: &str = "TextEvent";
pub fn register(id_reg: &mut IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}
pub fn new_text_event(id_reg: &mut IdRegistrar, data: String) -> TextEvent {
    TextEvent {
        id: id_reg.new_event_id(),
        type_id: id_reg.lookup_event_id(TYPE_STR.to_owned()).unwrap(),
        data: data,
    }
}

impl Event for TextEvent {
    fn event_type_id(&self) -> EventsTypeId {
        self.type_id
    }
    fn event_id(&self) -> EventsId {
        self.id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}