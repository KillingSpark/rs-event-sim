use crate::id_mngmnt::id_types::GateId;
use crate::id_mngmnt::id_types::MessageId;
use crate::id_mngmnt::id_types::MessageTypeId;
use crate::id_mngmnt::id_types::ModuleId;
use crate::id_mngmnt::id_types::PortId;
use std::any::Any;

pub trait Message {
    fn msg_type_id(&self) -> MessageTypeId;
    fn msg_id(&self) -> MessageId;

    //To downcast use this:
    //let tev: &TextMsg = event.as_any().downcast_ref::<TextMsg>().unwrap();
    fn as_any(&self) -> &dyn Any;
}

pub struct TimedMessage {
    pub time: u64,
    pub msg: Box<Message>,
    pub recipient: ModuleId,
    pub recp_port: PortId,
    pub recp_gate: GateId,
}

impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.time.cmp(&other.time) {
            std::cmp::Ordering::Equal => self.msg.msg_id().raw().cmp(&other.msg.msg_id().raw()),
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.cmp(self)) //reverse ordering, because binheap is a maxqueue
    }
}

impl PartialEq for TimedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for TimedMessage {}
