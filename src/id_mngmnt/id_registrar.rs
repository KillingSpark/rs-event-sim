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
            None => {},
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
}

