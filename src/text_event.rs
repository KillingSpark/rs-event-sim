use crate::event::Event;
use std::any::Any;
use crate::id_types::{EventsId, EventsTypeId};

#[derive(Clone)]
pub struct TextEvent {
    pub data: String,
    pub id: u64,
    pub type_id: u64,
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