use crate::connection::connection::Gate;
use crate::contexts::EventHandleContext;
use crate::event::Event;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;

pub struct HandleResult {}

pub struct FinalizeResult {
    //module-name, field-name, value
    pub results: Vec<(String, String, String)>,
}

pub trait Module {
    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>>;

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>>;

    fn module_type_id(&self) -> ModuleTypeId;
    fn module_id(&self) -> ModuleId;
    fn name(&self) -> String;

    fn get_gate_ids(&self) -> Vec<GateId>;
    fn initialize(
        &mut self,
        gates: &std::collections::HashMap<GateId, Gate>,
        ctx: &mut EventHandleContext,
    );
    fn finalize(&mut self, ctx: &mut EventHandleContext) -> Option<FinalizeResult>;
}
