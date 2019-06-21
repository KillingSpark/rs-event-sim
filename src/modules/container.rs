use crate::event::Event;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{HandleContext, HandleResult, FinalizeResult, Module};

pub struct ModuleContainer {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,

    name: String,
}

pub static OUTER_GATE: GateId = GateId(0);
pub static INNER_GATE: GateId = GateId(1);
pub static TYPE_STR: &str = "ModuleContainer";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_module_container(
    id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar,
    name: String,
) -> ModuleContainer {
    ModuleContainer {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),

        name: name,
    }
}

impl ModuleContainer {
    fn redirect_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut HandleContext,
    ) {
        let redirect_gate = if gate == OUTER_GATE {
            INNER_GATE
        } else {
            OUTER_GATE
        };

        let mut mctx = crate::connection::connection::HandleContext {
            time: ctx.time,
            id_reg: ctx.id_reg,
            prng: ctx.prng,
        };

        ctx.connections
            .send_message(msg, self.id, redirect_gate, port, &mut mctx);
    }
}

impl Module for ModuleContainer {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUTER_GATE, INNER_GATE]
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        self.redirect_message(msg, gate, port, ctx);
        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        panic!("Should never receive timer events");
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }

    fn initialize(&mut self, _ctx: &mut HandleContext) {}

    fn finalize(&mut self, _ctx: &mut HandleContext) -> Option<FinalizeResult>{
        println!("Finalized: {}, {}", &self.name, self.id.raw());
        None
    }
}
