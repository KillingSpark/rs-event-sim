use crate::connection::connection::Port;
use crate::event::{Event, TimerEvent};
use crate::id_mngmnt::id_registrar::IdRegistrar;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{FinalizeResult, HandleResult, Module};
use crate::contexts::EventHandleContext;
use crate::text_event;

pub struct RatePuller {
    type_id: ModuleTypeId,
    id: ModuleId,
    name: String,

    rate: u64, //how often to request a message
    last_time_requested: u64,
    ports: Vec<PortId>,
}

//messages get sent out here (and buffered)
pub const OUT_GATE: GateId = GateId(0);

//messages are received here, after triggering the buffer
pub const IN_GATE: GateId = GateId(1);

// send triggers to the buffer to request a new message from the buffer
pub const TRIG_GATE: GateId = GateId(2);

pub static TYPE_STR: &str = "RatePullerModule";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new(id_reg: &mut IdRegistrar, name: String, rate: u64) -> RatePuller {
    RatePuller {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,

        rate: rate,
        last_time_requested: 0,
        ports: Vec::new(),
    }
}

impl Module for RatePuller {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUT_GATE, IN_GATE, TRIG_GATE]
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        match gate {
            IN_GATE => {
                if ctx.mctx.time.now() - self.last_time_requested > self.rate {
                    self.last_time_requested = ctx.mctx.time.now();
                    let sig = Box::new(crate::messages::text_message::new_text_msg(
                        ctx.mctx.id_reg,
                        "New Message Plz".to_owned(),
                    ));

                    ctx.msgs_to_send.push_back((sig, TRIG_GATE, port));
                } else {
                    let time_till_next_pull =
                        self.rate - (ctx.mctx.time.now() - self.last_time_requested);
                    ctx.timer_queue.push(TimerEvent {
                        time: ctx.mctx.time.now() + time_till_next_pull,
                        mod_id: self.id,

                        event: Box::new(text_event::new_text_event(
                            ctx.mctx.id_reg,
                            "Pull new message".to_owned(),
                        )),
                    });
                }

                ctx.msgs_to_send.push_back((msg, OUT_GATE, port));
            }
            OUT_GATE => panic!("Should never receive message on OUT_GATE"),
            _ => panic!("Should never receive message on other gate"),
        }

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        ctx: &mut EventHandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        let sig = Box::new(crate::messages::text_message::new_text_msg(
            ctx.mctx.id_reg,
            "New Message Plz".to_owned(),
        ));

        ctx.msgs_to_send.push_back((sig, TRIG_GATE, PortId(0)));

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
        // initial request for a message
        self.ports = gates
            .get(&TRIG_GATE)
            .unwrap()
            .keys()
            .map(|id| *id)
            .collect();

        for port in &self.ports {
            let sig = Box::new(crate::messages::text_message::new_text_msg(
                ctx.mctx.id_reg,
                "New Message Plz".to_owned(),
            ));

            ctx.msgs_to_send.push_back((sig, TRIG_GATE, *port));
        }
    }

    fn finalize(&mut self, _ctx: &mut EventHandleContext) -> Option<FinalizeResult> {
        println!("Finalize Queue: {}", &self.name);
        None
    }
}
