use crate::connection::connection::Port;
use crate::event::Event;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{FinalizeResult, HandleResult, Module};
use crate::contexts::EventHandleContext;

pub struct Sink {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
    pub name: String,

    messages_sunk: u64,
}

pub static IN_GATE: GateId = GateId(0);
pub static TYPE_STR: &str = "SinkModule";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_sink(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar, name: String) -> Sink {
    Sink {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,

        messages_sunk: 0,
    }
}

impl Module for Sink {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![IN_GATE]
    }

    fn handle_message(
        &mut self,
        _msg: Box<Message>,
        _gate: GateId,
        _port: PortId,
        _ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        //println!(
        //    "Sink with ID: {} swallowed message with ID: {}!",
        //    self.id.raw(),
        //    msg.msg_id().raw(),
        //);
        self.messages_sunk += 1;

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        //println!(
        //    "Sink with ID: {} swallowed event with ID: {}!",
        //    self.id.raw(),
        //    ev.event_id().raw(),
        //);

        Ok(HandleResult {})
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn initialize(
        &mut self,
        _gates: &std::collections::HashMap<GateId, std::collections::HashMap<PortId, Port>>,
        _ctx: &mut EventHandleContext,
    ) {

    }
    fn finalize(&mut self, _ctx: &mut EventHandleContext) -> Option<FinalizeResult> {
        println!("Finalize Sink: {}", self.id.raw());
        Some(FinalizeResult {
            results: vec![(
                self.name(),
                "sunk_msgs".to_owned(),
                self.messages_sunk.to_string(),
            )],
        })
    }
}
