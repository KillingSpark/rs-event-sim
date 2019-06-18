use crate::messages::message::{Message};
use crate::id_mngmnt::id_types::{ConnectionId, ConnectionTypeId};
use crate::connection::connection::HandleContext;
use rand::RngCore;

pub struct SimpleConnection {
    pub buf: Vec<(u64, Box<Message>)>,
    pub id: ConnectionId,
    pub type_id: ConnectionTypeId,

    pub delay: u64,
    pub drop_chance: u64, // chance in percent * 100 (eg dropchance should be 50% ==> 5000)
}

pub static TYPE_STR: &str = "SimpleConnection";
pub fn register(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
}
pub fn new_simple_connection(id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar, delay: u64, drop_chance: u64) -> SimpleConnection {
    SimpleConnection {
        id: id_reg.new_connection_id(),
        type_id: id_reg.lookup_connection_id(TYPE_STR.to_owned()).unwrap(),
        delay: delay,
        drop_chance: drop_chance,
        buf: Vec::new(),
    }
}

impl crate::connection::connection::Connection for SimpleConnection {
    fn handle_message(&mut self, message: Box<Message>, ctx: &mut HandleContext) -> Option<(u64,Box<Message>)> {
        if ctx.prng.next_u64() & 10000 < self.drop_chance {
            return None;
        }

        Some((
            ctx.time.now() + self.delay,
            message,
        ))
    }

    fn connection_id(&self) -> ConnectionId {
        self.id
    }
    fn connection_type_id(&self) -> ConnectionTypeId {
        self.type_id
    }
}