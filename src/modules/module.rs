use crate::event::{Event, TimerEvent};
use crate::messages::message::{Message};
use crate::connection::connection;
use crate::id_mngmnt::id_types::{ModuleId, ModuleTypeId, PortId, GateId};
use std::collections::{BinaryHeap, VecDeque};
use crate::connection::connection::Gate;

pub struct HandleContext<'a> {
    pub mctx: connection::HandleContext<'a>,

    //output variables
    pub timer_queue: &'a mut BinaryHeap<TimerEvent>,
    pub msgs_to_send: &'a mut VecDeque<(Box<Message>, GateId, PortId)>,

}

pub struct HandleResult {
}

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
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>>;

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>>;

    fn module_type_id(&self) -> ModuleTypeId;
    fn module_id(&self) -> ModuleId;
    fn name(&self) -> String;

    fn get_gate_ids(&self) -> Vec<GateId>;
    fn initialize(&mut self, gates: &std::collections::HashMap<GateId, Gate>, ctx: &mut HandleContext);
    fn finalize(&mut self, ctx: &mut HandleContext) -> Option<FinalizeResult>;
}
