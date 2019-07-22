use crate::core::connection::connection::Port;
use crate::core::contexts::EventHandleContext;
use crate::core::events::event::{Event, TimerEvent};
use crate::core::events::text_event::{new_text_event, TextEvent};
use crate::core::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::core::messages::message::Message;
use crate::core::messages::text_message;
use crate::core::modules::module::{FinalizeResult, HandleResult, Module};

use crate::core::id_mngmnt::id_registrar::IdRegistrar;

pub struct SimpleModule {
    pub type_id: ModuleTypeId,
    pub id: ModuleId,
    pub name: String,
    ports: Vec<PortId>,

    pub msg_counter: u64,
    pub msg_time: u64,

    pub messages_sent: u64,
}

pub static OUT_GATE: GateId = GateId(0);
pub static IN_GATE: GateId = GateId(1);
pub static TYPE_STR: &str = "SimpleModule";

pub fn register(id_reg: &mut IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new_simple_module(id_reg: &mut IdRegistrar, name: String) -> SimpleModule {
    SimpleModule {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,

        msg_counter: 0,
        msg_time: 0,

        messages_sent: 0,
        ports: Vec::new(),
    }
}

impl SimpleModule {
    fn send_to_all(&mut self, ctx: &mut EventHandleContext) {
        for port in &self.ports {
            let sig = Box::new(text_message::new_text_msg(
                ctx.mctx.id_reg,
                "Received Event".to_owned(),
            ));
            ctx.msgs_to_send.push_back((sig, OUT_GATE, *port));
            self.messages_sent += 1;
        }
        self.msg_counter += 1;
    }
}

impl Module for SimpleModule {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUT_GATE, IN_GATE]
    }

    fn handle_message(
        &mut self,
        _msg: Box<Message>,
        _gate: GateId,
        _port: PortId,
        _ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        //println!(
        //    "SimpleModule with ID: {} swallowed message with ID: {}!",
        //    self.id.raw(),
        //    msg.msg_id().raw(),
        //);

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        ev: &Event,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        if self.msg_time != ctx.mctx.time.now() {
            self.msg_counter = 0;
            self.msg_time = ctx.mctx.time.now();
        }

        let te_type = ctx
            .mctx
            .id_reg
            .lookup_event_id("TextEvent".to_owned())
            .unwrap();
        if self.msg_counter == 0 {
            //println!(
            //    "Module with Id: {} Handled timer event: {}",
            //    self.id.raw(),
            //    ev.event_id().raw(),
            //);

            if ev.event_type_id() == te_type {
                let tev: &TextEvent = ev.as_any().downcast_ref::<TextEvent>().unwrap();
                ctx.timer_queue.push(TimerEvent {
                    event: Box::new(TextEvent {
                        data: tev.data.clone(),
                        id: ctx.mctx.id_reg.new_event_id(),
                        type_id: te_type,
                    }),
                    time: ctx.mctx.time.now() + 1,
                    mod_id: self.module_id(),
                });
            } else {
                println!(
                    "Was {}. Dont know what to do with it though.",
                    ctx.mctx
                        .id_reg
                        .lookup_event_id_reverse(ev.event_type_id())
                        .unwrap()
                );
            }
        }

        if self.msg_counter < 3 {
            ctx.timer_queue.push(TimerEvent {
                event: Box::new(TextEvent {
                    data: "just a wakeup".to_owned(),
                    id: ctx.mctx.id_reg.new_event_id(),
                    type_id: te_type,
                }),
                time: ctx.mctx.time.now(),
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

    fn name(&self) -> String {
        self.name.clone()
    }

    fn initialize(
        &mut self,
        gates: &std::collections::HashMap<GateId, std::collections::HashMap<PortId, Port>>,
        ctx: &mut EventHandleContext,
    ) {
        self.ports = gates.get(&OUT_GATE).unwrap().keys().map(|id| *id).collect();

        ctx.timer_queue.push(TimerEvent {
            time: 10,
            mod_id: self.id,
            event: Box::new(new_text_event(ctx.mctx.id_reg, "StarterEvent".to_owned())),
        });
    }

    fn finalize(&mut self, _ctx: &mut EventHandleContext) -> Option<FinalizeResult> {
        //println!("Finalize SimpleModule: {}", self.id.raw());
        Some(FinalizeResult {
            results: vec![(
                self.name(),
                "sent_msgs".to_owned(),
                self.messages_sent.to_string(),
            )],
        })
    }
}
