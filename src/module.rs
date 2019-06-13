use crate::clock::Clock;
use crate::event::{Event, TimerEvent};
use crate::id_registrar::IdRegistrar;
use crate::messages::message::Message;
use crate::connection::connection::ConnectionMesh;
use crate::id_types::{ModuleId, ModuleTypeId};

pub struct HandleContext<'a> {
    pub time: &'a Clock,
    pub id_reg: &'a mut IdRegistrar,

    pub connections: &'a mut ConnectionMesh,
}

pub struct HandleResult {
    pub timer_events: Option<Vec<TimerEvent>>,
}

pub trait Module {
    fn handle_message(
        &mut self,
        ev: &Message,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>>;

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>>;

    fn module_type_id(&self) -> ModuleTypeId;
    fn module_id(&self) -> ModuleId;
}
