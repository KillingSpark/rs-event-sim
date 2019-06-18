use crate::event::{Event};
use crate::id_mngmnt::id_types::{ModuleId, ModuleTypeId};
use crate::messages::message::Message;
use crate::modules::module::{HandleContext, HandleResult, Module};

pub struct Sink {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
}


pub static IN_GATE: u64 = 0;
pub static TYPE_STR: &str = "SinkModule";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_sink(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) -> Sink {
    Sink {
        id: id_reg.new_id(),
        type_id: *id_reg.lookup_id(TYPE_STR.to_owned()).unwrap(),
    }
}

impl Module for Sink {
    fn get_gate_ids(&self) -> Vec<u64> {
        vec![IN_GATE]
    }

    fn handle_message(
        &mut self,
        msg: &Message,
        _ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        println!(
            "Sink with ID: {} swallowed message with ID: {}!",
            self.id,
            msg.msg_id(),
        );

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        _ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        println!(
            "Sink with ID: {} swallowed event with ID: {}!",
            self.id,
            ev.event_id(),
        );

        Ok(HandleResult {})
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }
}