use crate::event::Event;
use std::any::Any;

#[derive(Clone)]
pub struct TextEvent {
    pub data: String,
    pub ev_id: u64,
    pub type_id: u64,
}

impl Event for TextEvent {
    fn event_type_id(&self) -> u64 {
        self.type_id
    }
    fn event_id(&self) -> u64 {
        self.ev_id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}