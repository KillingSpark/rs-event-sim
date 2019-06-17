use crate::event::{Event};
use crate::id_mngmnt::id_types::{ModuleId, ModuleTypeId};
use crate::messages::message::Message;
use crate::modules::module::{HandleContext, HandleResult, Module};

pub struct Sink {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
}

impl Module for Sink {
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