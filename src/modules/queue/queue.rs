use crate::connection::connection::Gate;
use crate::event::Event;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{FinalizeResult, HandleResult, Module};
use crate::contexts::EventHandleContext;

pub struct Queue {
    type_id: ModuleTypeId,
    id: ModuleId,
    name: String,

    msgs: Vec<Box<Message>>,
    receive_ready: Vec<PortId>,
}

//messages get sent out here, on the port where the trigger came
pub const OUT_GATE: GateId = GateId(0);

//messages come in here (0..n) ports
pub const IN_GATE: GateId = GateId(1);

//if a message is received here a new message is collected from the buffer if not empty
//and sent on the port on OUT_GATE
pub const TRIGG_GATE: GateId = GateId(2);

pub static TYPE_STR: &str = "QueueModule";

pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}

pub fn new(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar, name: String) -> Queue {
    Queue {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,

        msgs: Vec::new(),
        receive_ready: Vec::new(),
    }
}

impl Module for Queue {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUT_GATE, IN_GATE, TRIGG_GATE]
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
                //if some port signaled readyness push to this port instead of queuing
                //else put into queue
                if !self.receive_ready.is_empty() {
                    let bufferd_port = self.receive_ready.remove(0);
                    ctx.msgs_to_send.push_back((msg, OUT_GATE, bufferd_port));
                } else {
                    self.msgs.push(msg);
                }
            }

            //if triggered send message from queue to the port on OUT_GATE
            //else remember readiness in receive_ready
            TRIGG_GATE => {
                if !self.msgs.is_empty() {
                    let bufferd_msg = self.msgs.remove(0);
                    ctx.msgs_to_send.push_back((bufferd_msg, OUT_GATE, port));
                } else {
                    self.receive_ready.push(port);
                }
            }
            OUT_GATE => panic!("Should not receive messages on OUT_GATE"),
            _ => panic!("Should not receive messages on other gates"),
        }

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut EventHandleContext,
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

    fn initialize(
        &mut self,
        _gates: &std::collections::HashMap<GateId, Gate>,
        _ctx: &mut EventHandleContext,
    ) {
    }

    fn finalize(&mut self, _ctx: &mut EventHandleContext) -> Option<FinalizeResult> {
        println!("Finalize Queue: {}", &self.name);
        None
    }
}
