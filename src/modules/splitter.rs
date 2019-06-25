use crate::event::Event;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{FinalizeResult, HandleContext, HandleResult, Module};

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

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar, name: String) -> Splitter {
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
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        match gate {
            SPLIT_IN_GATE => {
                let mut mctx = crate::connection::connection::HandleContext {
                    time: ctx.time,
                    id_reg: ctx.id_reg,
                    prng: ctx.prng,
                };
                ctx.connections
                    .send_message(msg, self.id, IN_OUT_GATE, port, &mut mctx)
            }
            IN_OUT_GATE => {
                let mut mctx = crate::connection::connection::HandleContext {
                    time: ctx.time,
                    id_reg: ctx.id_reg,
                    prng: ctx.prng,
                };
                ctx.connections
                    .send_message(msg, self.id, SPLIT_OUT_GATE, port, &mut mctx)
            }
            SPLIT_OUT_GATE => panic!("Should not receive messages on SPLIT_OUT_GATE"),
            _ => panic!("Should not receive messages on other gates"),
        }

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut HandleContext,
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

    fn initialize(&mut self, _ctx: &mut HandleContext) {}

    fn finalize(&mut self, _ctx: &mut HandleContext) -> Option<FinalizeResult> {
        println!("Finalize Queue: {}", &self.name);
        None
    }
}
