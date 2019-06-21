#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ModuleId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ModuleTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct EventsId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct EventsTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct MessageId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct MessageTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ConnectionId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ConnectionTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PortId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct GateId(pub u64);


impl ModuleId {
    pub fn raw(&self)-> u64 {
        match self {ModuleId(id) => *id}
    }
}

impl EventsId {
    pub fn raw(&self)-> u64 {
        match self {EventsId(id) => *id}
    }
}

impl MessageId {
    pub fn raw(&self)-> u64 {
        match self {MessageId(id) => *id}
    }
}

impl ConnectionId {
    pub fn raw(&self)-> u64 {
        match self {ConnectionId(id) => *id}
    }
}