use crate::event::{Event, TimerEvent};
use crate::events::text_event::{new_text_event, TextEvent};
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::messages::text_message;
use crate::modules::module::{HandleContext, HandleResult, Module};

pub struct SimpleModule {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,

    pub msg_counter: u64,
    pub msg_time: u64,
}

pub static OUT_GATE: GateId = GateId(0);
pub static IN_GATE: GateId = GateId(1);
pub static TYPE_STR: &str = "SimpleModule";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_simple_module(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) -> SimpleModule {
    SimpleModule {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),

        msg_counter: 0,
        msg_time: 0,
    }
}

impl SimpleModule {
    fn send_to_all(&mut self, ctx: &mut HandleContext) {
        let ports = ctx.connections.get_ports(self.module_id(), OUT_GATE);

        match ports {
            Some(ports) => {
                for port in ports {
                    let sig = Box::new(text_message::new_text_msg(
                        ctx.id_reg,
                        "Received Event".to_owned(),
                    ));
                    let mut mctx = crate::connection::connection::HandleContext {
                        time: ctx.time,
                        id_reg: ctx.id_reg,
                        prng: ctx.prng,
                    };
                    ctx.connections
                        .send_message(sig, self.id, OUT_GATE, port, &mut mctx);
                }
                self.msg_counter += 1;
            }
            None => {}
        }
    }
}

impl Module for SimpleModule {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUT_GATE, IN_GATE]
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        _gate: GateId,
        _port: PortId,
        _ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
         println!(
            "SimpleModule with ID: {} swallowed message with ID: {}!",
            self.id.raw(),
            msg.msg_id().raw(),
        );

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

        let te_type = ctx.id_reg.lookup_event_id("TextEvent".to_owned()).unwrap();
        if self.msg_counter == 0 {
            println!(
                "Module with Id: {} Handled timer event: {}",
                self.id.raw(),
                ev.event_id().raw(),
            );

            if ev.event_type_id() == te_type {
                let tev: &TextEvent = ev.as_any().downcast_ref::<TextEvent>().unwrap();
                println!("Was Textevent with data: {}", tev.data);
                if ctx.time.now() < 16 {
                    ctx.timer_queue.push(TimerEvent {
                        event: Box::new(TextEvent {
                            data: tev.data.clone(),
                            id: ctx.id_reg.new_event_id(),
                            type_id: te_type,
                        }),
                        time: ctx.time.now() + 1,
                        mod_id: self.module_id(),
                    });
                }
            } else {
                println!(
                    "Was {}. Dont know what to do with it though.",
                    ctx.id_reg
                        .lookup_event_id_reverse(ev.event_type_id())
                        .unwrap()
                );
            }
        }

        if self.msg_counter < 3 {
            ctx.timer_queue.push(TimerEvent {
                event: Box::new(TextEvent {
                    data: "just a wakeup".to_owned(),
                    id: ctx.id_reg.new_event_id(),
                    type_id: te_type,
                }),
                time: ctx.time.now(),
                mod_id: self.module_id(),
            });

            self.send_to_all(ctx);
        }

        Ok(HandleResult {})
    }

    fn module_type_id(&self) -> ModuleTypeId {
        self.type_id
    }

    fn module_id(&self) -> ModuleId {
        self.id
    }

    fn initialize(&mut self, ctx: &mut HandleContext) {
        ctx.timer_queue.push(TimerEvent {
            time: 10,
            mod_id: self.id,
            event: Box::new(new_text_event(ctx.id_reg, "StarterEvent".to_owned())),
        });
    }

    fn finalize(&mut self, _ctx: &mut HandleContext) {}
}
