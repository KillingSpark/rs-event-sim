use crate::clock::Clock;
use crate::event::{Event, TimerEvent};
use crate::messages::message::Message;
use crate::connection::connection::ConnectionMesh;
use crate::id_mngmnt::id_registrar::IdRegistrar;
use crate::id_mngmnt::id_types::{ModuleId, ModuleTypeId};

pub struct HandleContext<'a> {
    pub time: &'a Clock,
    pub id_reg: &'a mut IdRegistrar,

    pub connections: &'a mut ConnectionMesh,
    pub timer_queue: &'a mut std::collections::BinaryHeap<TimerEvent>,
}

pub struct HandleResult {
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

    fn get_gate_ids(&self) -> Vec<u64>;
}
