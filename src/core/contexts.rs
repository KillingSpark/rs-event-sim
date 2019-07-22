use crate::core::clock::Clock;
use crate::core::events::event::TimerEvent;
use crate::core::id_mngmnt::id_registrar::IdRegistrar;
use crate::core::id_mngmnt::id_types::{GateId, PortId};
use crate::core::messages::message::Message;
use std::collections::{BinaryHeap, VecDeque};

pub struct EventHandleContext<'a> {
    pub mctx: SimulationContext<'a>,

    //output variables
    pub timer_queue: &'a mut BinaryHeap<TimerEvent>,
    pub msgs_to_send: &'a mut VecDeque<(Box<Message>, GateId, PortId)>,
}

pub struct SimulationContext<'a> {
    pub time: &'a Clock,
    pub id_reg: &'a mut IdRegistrar,
    pub prng: &'a mut rand::prng::XorShiftRng,
}
