use crate::core::connection::connection::Port;
use crate::core::contexts::EventHandleContext;
use crate::core::events::event::Event;
use crate::core::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::core::messages::message::Message;

use std::collections::HashMap;

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
        gates: &HashMap<GateId, HashMap<PortId, Port>>,
        ctx: &mut EventHandleContext,
    );
    fn finalize(&mut self, ctx: &mut EventHandleContext) -> Option<FinalizeResult>;
}
