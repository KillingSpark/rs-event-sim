use crate::core::connection::connection::Port;
use crate::core::contexts::EventHandleContext;
use crate::core::events::event::Event;
use crate::core::id_mngmnt::id_registrar::IdRegistrar;
use crate::core::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::core::messages::message::Message;
use crate::core::modules::module::{FinalizeResult, HandleResult, Module};

pub struct Splitter {
    type_id: ModuleTypeId,
    id: ModuleId,
    name: String,
}

//messages get sent out here, on the port from IN_GATE
pub const SPLIT_OUT_GATE: GateId = GateId(0);
//messages get sent out here, on the port from IN_GATE
pub const SPLIT_IN_GATE: GateId = GateId(1);

//messages come in here (0..n) ports
pub const IN_OUT_GATE: GateId = GateId(2);

pub static TYPE_STR: &str = "SplitModule";

pub fn register(id_reg: &mut IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new(id_reg: &mut IdRegistrar, name: String) -> Splitter {
    Splitter {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,
    }
}

impl Module for Splitter {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![SPLIT_OUT_GATE, SPLIT_IN_GATE, IN_OUT_GATE]
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        match gate {
            SPLIT_IN_GATE => {
                ctx.msgs_to_send.push_back((msg, IN_OUT_GATE, port));
            }
            IN_OUT_GATE => {
                ctx.msgs_to_send.push_back((msg, SPLIT_OUT_GATE, port));
            }
            SPLIT_OUT_GATE => panic!("Should not receive messages on SPLIT_OUT_GATE"),
            _ => panic!("Should not receive messages on other gates"),
        }

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        panic!("Should never receive timer events")
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
        //println!("Finalize Splitter: {}", &self.name);
        None
    }
}
