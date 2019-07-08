use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::connection::connection::{Connection, HandleContext};
use crate::id_mngmnt::id_types::{ConnectionId, GateId, ModuleId, PortId};
use crate::messages::message::Message;
use crate::modules::module::Module;
use crate::events::event::Event;

type Wrapper<T> = Rc<RefCell<T>>;

pub struct Network {
    conns: HashMap<ConnectionId, Wrapper<Box<Connection>>>,
    mod_infos: HashMap<ModuleId, ModuleInfo>,

    //messages that will be handled in the future
    pub messages: std::collections::BinaryHeap<TimedMessage>,
    //messages that will be handled in this point in time. No need to sift them through the heap and immediately pop them
    pub messages_now: std::collections::VecDeque<TimedMessage>,

    timer_queue: std::collections::BinaryHeap<TimerEvent>,
}

#[derive(Clone)]
struct ConnInfo {
    conn: Wrapper<Box<Connection>>,
    rcv: Wrapper<Box<Module>>,
    rcv_gate: GateId,
    rcv_port: PortId,
}

pub struct TimerEvent {
    pub time: u64,
    pub module: Wrapper<Box<Module>>,
    pub event: Box<Event>,
}

impl Ord for TimerEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.time.cmp(&other.time) {
            std::cmp::Ordering::Equal => self.event.event_id().raw().cmp(&other.event.event_id().raw()),
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for TimerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.cmp(self)) //reverse ordering, because binheap is a maxqueue
    }
}

impl PartialEq for TimerEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for TimerEvent {}

pub struct TimedMessage {
    pub time: u64,
    pub msg: Box<Message>,
    info: ConnInfo,
}

impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        //reverse so maxqueue takes event with smaller time
        match self.time.cmp(&other.time) {
            std::cmp::Ordering::Equal => self.msg.msg_id().cmp(&other.msg.msg_id()),
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for TimedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for TimedMessage {}

struct ModuleInfo {
    module: Wrapper<Box<Module>>,
    conns: HashMap<(GateId, PortId), ConnInfo>,
}

pub struct ConnDesc {
    pub port: PortId,
    pub gate: GateId,
    pub module: ModuleId,
}

pub enum Direction {
    Bidir,
    Onedir,
}

impl Network {
    pub fn add_module(&mut self, module: Box<Module>) {
        self.mod_infos.insert(
            module.module_id(),
            ModuleInfo {
                module: Wrapper::new(RefCell::new(module)),
                conns: HashMap::new(),
            },
        );
    }

    pub fn add_connection(
        &mut self,
        conn1: ConnDesc,
        conn2: ConnDesc,
        conn: ConnectionId,
        direction: Direction,
    ) {
        {
            let target_mod = self
                .mod_infos
                .get_mut(&conn2.module)
                .unwrap()
                .module
                .clone();
            let source_mod_info = self.mod_infos.get_mut(&conn1.module).unwrap();
            source_mod_info.conns.insert(
                (conn1.gate, conn1.port),
                ConnInfo {
                    conn: self.conns.get(&conn).unwrap().clone(),
                    rcv: target_mod,
                    rcv_gate: conn2.gate,
                    rcv_port: conn2.port,
                },
            );
        }

        match direction {
            Direction::Onedir => { /* dont connect back */ }
            Direction::Bidir => {
                let target_mod = self
                    .mod_infos
                    .get_mut(&conn1.module)
                    .unwrap()
                    .module
                    .clone();
                let source_mod_info = self.mod_infos.get_mut(&conn2.module).unwrap();
                source_mod_info.conns.insert(
                    (conn2.gate, conn2.port),
                    ConnInfo {
                        conn: self.conns.get(&conn).unwrap().clone(),
                        rcv: target_mod,
                        rcv_gate: conn1.gate,
                        rcv_port: conn1.port,
                    },
                );
            }
        }
    }

    pub fn send(
        &mut self,
        msg: Box<Message>,
        ctx: &mut HandleContext,
        mod_id: ModuleId,
        gate: GateId,
        port: PortId,
    ) {
        let conn_info = self
            .mod_infos
            .get_mut(&mod_id)
            .unwrap()
            .conns
            .get_mut(&(gate, port))
            .unwrap();
        match conn_info.conn.borrow_mut().handle_message(msg, ctx) {
            Some((time, msg)) => {
                let tmsg = TimedMessage {
                    time: time,
                    msg: msg,
                    info: conn_info.clone(),
                };
                if time == ctx.time.now() {
                    self.messages_now.push_back(tmsg);
                } else {
                    self.messages.push(tmsg);
                }
            }
            None => {
                //yey less messages in queue
            }
        }
    }
}

pub struct NetworkManager {
    network: Network,
}

impl NetworkManager {
    fn process_message_queue(&mut self, time_now: u64, ctx: &mut crate::modules::module::HandleContext) -> u64{
        let mut msg_counter = 0;
        loop {
            match self.network.messages.peek() {
                Some(tmsg) => {
                    if tmsg.time > time_now {
                        break;
                    }
                }
                None => {
                    break;
                }
            }

            let tmsg = self.network.messages.pop().unwrap();
            tmsg.info.rcv.borrow_mut().handle_message(tmsg.msg, tmsg.info.rcv_gate, tmsg.info.rcv_port, ctx).unwrap();
            msg_counter += 1;
        }

        loop {
            if self.network.messages_now.len() == 0 {
                break;
            }

            let tmsg = self.network.messages.pop().unwrap();
            tmsg.info.rcv.borrow_mut().handle_message(tmsg.msg, tmsg.info.rcv_gate, tmsg.info.rcv_port, ctx).unwrap();
            msg_counter += 1;
        }

        msg_counter
    }

    fn process_event_queue(&mut self, time_now: u64, ctx: &mut crate::modules::module::HandleContext) -> Result<u64, Box<std::error::Error>> {
        let mut events_counter = 0;
        loop {
            match self.network.timer_queue.peek() {
                Some(ev) => {
                    if ev.time > time_now {
                        break;
                    }
                }
                None => {
                    break;
                }
            }

            let ev = self.network.timer_queue.pop().unwrap();
            ev.module.borrow_mut().handle_timer_event(ev.event.as_ref(), ctx).unwrap();

            events_counter += 1;
        }
        Ok(events_counter)
    }
}
