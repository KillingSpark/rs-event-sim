use crate::id_mngmnt::id_types::*;

pub struct IdRegistrar {
    pub last_id: u64,
    pub last_type_id: u64,
    pub type_ids: std::collections::HashMap<String, u64>,
    pub type_ids_reverse: std::collections::HashMap<u64, String>,
}

impl IdRegistrar {
    pub fn new_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }

    fn new_type_id(&mut self) -> u64 {
        self.last_type_id += 1;
        self.last_type_id
    }

    pub fn register_type(&mut self, type_id: String) -> u64 {
        //if type is already registered just return old value. This might happen when more then one
        //combined module type tries to register the same type
        match self.type_ids.get(&type_id) {
            Some(id) => return *id,
            None => {}
        }

        let new_id = self.new_type_id();
        self.type_ids.insert(type_id.clone(), new_id);
        self.type_ids_reverse.insert(new_id, type_id);

        new_id
    }

    pub fn lookup_id(&mut self, type_id: String) -> Option<&u64> {
        self.type_ids.get(&type_id)
    }

    pub fn lookup_id_reverse(&mut self, type_id: u64) -> Option<&String> {
        self.type_ids_reverse.get(&type_id)
    }

    pub fn new_message_id(&mut self) -> MessageId {
        MessageId(self.new_id())
    }
    pub fn new_module_id(&mut self) -> ModuleId {
        ModuleId(self.new_id())
    }
    pub fn new_event_id(&mut self) -> EventsId {
        EventsId(self.new_id())
    }
    pub fn new_connection_id(&mut self) -> ConnectionId {
        ConnectionId(self.new_id())
    }

    pub fn lookup_message_id(&mut self, type_id: String) -> Option<MessageTypeId> {
        match self.lookup_id(type_id) {
            Some(id) => Some(MessageTypeId(*id)),
            None => None,
        }
    }
    pub fn lookup_module_id(&mut self, type_id: String) -> Option<ModuleTypeId> {
       match self.lookup_id(type_id) {
            Some(id) => Some(ModuleTypeId(*id)),
            None => None,
        }
    }
    pub fn lookup_event_id(&mut self, type_id: String) -> Option<EventsTypeId> {
        match self.lookup_id(type_id) {
            Some(id) => Some(EventsTypeId(*id)),
            None => None,
        }
    }
    pub fn lookup_connection_id(&mut self, type_id: String) -> Option<ConnectionTypeId> {
        match self.lookup_id(type_id) {
            Some(id) => Some(ConnectionTypeId(*id)),
            None => None,
        }
    }

    pub fn lookup_message_id_reverse(&mut self, type_id: MessageTypeId) -> Option<&String> {
        self.lookup_id_reverse(match type_id{MessageTypeId(id) => id})
    }
    pub fn lookup_module_id_reverse(&mut self, type_id: ModuleTypeId) -> Option<&String> {
       self.lookup_id_reverse(match type_id{ModuleTypeId(id) => id})
    }
    pub fn lookup_event_id_reverse(&mut self, type_id: EventsTypeId) -> Option<&String> {
        self.lookup_id_reverse(match type_id{EventsTypeId(id) => id})
    }
    pub fn lookup_connection_id_reverse(&mut self, type_id: ConnectionTypeId) -> Option<&String> {
        self.lookup_id_reverse(match type_id{ConnectionTypeId(id) => id})
    }
}
