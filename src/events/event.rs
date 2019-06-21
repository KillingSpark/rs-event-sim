use std::any::Any;
use crate::id_mngmnt::id_types::{EventsId, EventsTypeId, ModuleId};

pub trait Event {
    fn event_type_id(&self) -> EventsTypeId;
    fn event_id(&self) -> EventsId;

    //To downcast use this:
    //let tev: &TextEvent = event.as_any().downcast_ref::<TextEvent>().unwrap();
    fn as_any(&self) -> &dyn Any;
}


pub struct TimerEvent {
    pub time: u64,
    pub mod_id: ModuleId,
    pub event: Box<Event>,
}

impl Ord for TimerEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.time.cmp(&other.time) {
            std::cmp::Ordering::Equal => self.event.event_id().raw().cmp(&other.event.event_id().raw()),
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for TimerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.cmp(self)) //reverse ordering, because binheap is a maxqueue
    }
}

impl PartialEq for TimerEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for TimerEvent {}