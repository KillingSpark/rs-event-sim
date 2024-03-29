use crate::core::connection::connection::Port;
use crate::core::contexts::EventHandleContext;
use crate::core::events::event::Event;
use crate::core::id_mngmnt::id_registrar::IdRegistrar;
use crate::core::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::core::messages::message::Message;
use crate::core::modules::module::{FinalizeResult, HandleResult, Module};

pub struct EchoModule {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
    pub name: String,

    msgs_echoed: u64,
}

pub static OUT_GATE: GateId = GateId(0);
pub static IN_GATE: GateId = GateId(1);
pub static TYPE_STR: &str = "EchoModule";

pub fn register(id_reg: &mut IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_echo_module(id_reg: &mut IdRegistrar, name: String) -> EchoModule {
    EchoModule {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,

        msgs_echoed: 0,
    }
}

impl Module for EchoModule {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUT_GATE, IN_GATE]
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        //println!(
        //    "EchoModule with ID: {} echoed message with ID: {}!",
        //    self.id.raw(),
        //    msg.msg_id().raw(),
        //);
        ctx.msgs_to_send.push_back((msg, gate, port));
        self.msgs_echoed += 1;

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        //println!(
        //    "EchoModule with ID: {} swallowed event with ID: {}!",
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
        //println!("Finalize Echo: {}", self.id.raw());
        Some(FinalizeResult {
            results: vec![(
                self.name(),
                "echoed_msgs".to_owned(),
                self.msgs_echoed.to_string(),
            )],
        })
    }
}
