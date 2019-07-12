use crate::connection::connection::Port;
use crate::event::Event;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{FinalizeResult, HandleResult, Module};
use crate::contexts::EventHandleContext;

pub struct ModuleContainer {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
    pub inner_to_outer_gates: std::collections::HashMap<GateId, GateId>,
    pub outer_to_inner_gates: std::collections::HashMap<GateId, GateId>,

    name: String,
}

pub static TYPE_STR: &str = "ModuleContainer";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_module_container(
    id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar,
    name: String,
    gates: Vec<(GateId, GateId)>,
) -> ModuleContainer {
    let mut container = ModuleContainer {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        inner_to_outer_gates: std::collections::HashMap::new(),
        outer_to_inner_gates: std::collections::HashMap::new(),

        name: name,
    };

    for (outer, inner) in gates {
        container.inner_to_outer_gates.insert(inner, outer);
        container.outer_to_inner_gates.insert(outer, inner);
    }

    container
}

impl ModuleContainer {
    fn redirect_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut EventHandleContext,
    ) {
        let inner = self.outer_to_inner_gates.get(&gate);
        let redirect_gate = match inner {
            Some(inner_gate) => Some(inner_gate),
            None => match self.inner_to_outer_gates.get(&gate) {
                Some(outer_gate) => Some(outer_gate),
                None => None,
            },
        };

        match redirect_gate {
            Some(redirect_gate) => {
                ctx.msgs_to_send.push_back((msg, *redirect_gate, port));
            }
            None => {
                panic!("No gate found");
            }
        }
    }
}

impl Module for ModuleContainer {
    fn get_gate_ids(&self) -> Vec<GateId> {
        let mut gates = Vec::with_capacity(self.inner_to_outer_gates.len() * 2);
        for (i, o) in &self.inner_to_outer_gates {
            gates.push(*i);
            gates.push(*o);
        }

        gates
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        self.redirect_message(msg, gate, port, ctx);
        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        panic!("Should never receive timer events");
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }

    fn initialize(
        &mut self,
        _gates: &std::collections::HashMap<GateId, std::collections::HashMap<PortId, Port>>,
        _ctx: &mut EventHandleContext,
    ) {
    }

    fn finalize(&mut self, _ctx: &mut EventHandleContext) -> Option<FinalizeResult> {
        //println!("Finalized: {}, {}", &self.name, self.id.raw());
        None
    }
}
