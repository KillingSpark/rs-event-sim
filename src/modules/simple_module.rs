use crate::event::{Event, TimerEvent};
use crate::messages::message::Message;
use crate::modules::module::{HandleContext, HandleResult, Module};
use crate::events::text_event::TextEvent;
use crate::id_types::{ModuleId, ModuleTypeId};

pub struct SimpleModule {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
}

impl Module for SimpleModule {
    fn handle_message(
        &mut self,
        _ev: &Message,
        _ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        println!("Wohoo message received!");
        Ok(HandleResult { timer_events: None })
    }

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        println!("Id: {} Handled timer event: {}", self.id, ev.event_id());

        let mut new_events = Vec::new();

        let te_type = *ctx.id_reg.lookup_id("TextEvent".to_owned()).unwrap();

        if ev.event_type_id() == te_type {
            let tev: &TextEvent = ev.as_any().downcast_ref::<TextEvent>().unwrap();
            println!("Was Textevent with data: {}", tev.data);
            if ctx.time.now() < 20 {
                new_events.push(TimerEvent {
                    event: Box::new(TextEvent {
                        data: tev.data.clone(),
                        id: ctx.id_reg.new_id(),
                        type_id: te_type,
                    }),
                    time: ctx.time.now() + 1,
                    mod_id: self.module_id(),
                });
            }
        } else {
            println!(
                "Was {}. Dont know what to do with it though.",
                ctx.id_reg.lookup_id_reverse(ev.event_type_id()).unwrap()
            );
        }

        let sig = Box::new(crate::text_message::TextMsg {
            type_id: *ctx.id_reg.lookup_id("TextSignal".to_owned()).unwrap(),
            id: ctx.id_reg.new_id(),
            data: "Received Event".to_owned(),
        });

        let conn = ctx.connections.get_out_connection(self.module_id(), 0);
        match conn {
            None => {/*No conn found*/},
            Some(c) => {c.push(vec![sig]).unwrap();}
        }

        Ok(HandleResult {
            timer_events: Some(new_events),
        })
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }
}
