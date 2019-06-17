use crate::event::{Event, TimerEvent};
use crate::events::text_event::TextEvent;
use crate::id_mngmnt::id_types::{ModuleId, ModuleTypeId};
use crate::messages::message::Message;
use crate::modules::module::{HandleContext, HandleResult, Module};

pub struct SimpleModule {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,

    pub msg_counter: u64,
    pub msg_time: u64,
}

impl Module for SimpleModule {
    fn handle_message(
        &mut self,
        _ev: &Message,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        println!(
            "Module with ID: {} Wohoo message received at time!",
            self.id
        );
        let ports = ctx.connections.get_ports(self.module_id());

        match ports {
            Some(ports) => {
                for port in ports {
                    let sig = Box::new(crate::text_message::TextMsg {
                        type_id: *ctx.id_reg.lookup_id("TextSignal".to_owned()).unwrap(),
                        id: ctx.id_reg.new_id(),
                        data: "Received Event".to_owned(),
                    });
                    let mut mctx = crate::connection::connection::HandleContext {
                        time: ctx.time,
                        id_reg: ctx.id_reg,
                    };
                    ctx.connections.send_message(sig, self.id, port, &mut mctx);
                }
                self.msg_counter += 1;
            }
            None => {}
        }
        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        if self.msg_time != ctx.time.now() {
            self.msg_counter = 0;
            self.msg_time = ctx.time.now();
        }

        let te_type = *ctx.id_reg.lookup_id("TextEvent".to_owned()).unwrap();
        if self.msg_counter == 0 {
            println!(
                "Module with Id: {} Handled timer event: {}",
                self.id,
                ev.event_id()
            );

            if ev.event_type_id() == te_type {
                let tev: &TextEvent = ev.as_any().downcast_ref::<TextEvent>().unwrap();
                println!("Was Textevent with data: {}", tev.data);
                if ctx.time.now() < 16 {
                    ctx.timer_queue.push(TimerEvent {
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
        }

        if self.msg_counter < 3 {
            ctx.timer_queue.push(TimerEvent {
                event: Box::new(TextEvent {
                    data: "just a wakeup".to_owned(),
                    id: ctx.id_reg.new_id(),
                    type_id: te_type,
                }),
                time: ctx.time.now(),
                mod_id: self.module_id(),
            });

            let ports = ctx.connections.get_ports(self.module_id());

            match ports {
                Some(ports) => {
                    for port in ports {
                        let sig = Box::new(crate::text_message::TextMsg {
                            type_id: *ctx.id_reg.lookup_id("TextSignal".to_owned()).unwrap(),
                            id: ctx.id_reg.new_id(),
                            data: "Received Event".to_owned(),
                        });
                        let mut mctx = crate::connection::connection::HandleContext {
                            time: ctx.time,
                            id_reg: ctx.id_reg,
                        };
                        ctx.connections.send_message(sig, self.id, port, &mut mctx);
                    }
                    self.msg_counter += 1;
                }
                None => {}
            }
        }

        Ok(HandleResult {})
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }
}
