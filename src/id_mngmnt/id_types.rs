#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ModuleTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EventsId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EventsTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MessageTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionTypeId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PortId(pub u64);
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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