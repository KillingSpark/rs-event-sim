use crate::id_mngmnt::id_types::{ConnectionId, ConnectionTypeId, GateId, ModuleId, PortId};
use crate::messages::message::{Message};
use crate::contexts::SimulationContext;

pub trait Connection {
    fn handle_message(
        &mut self,
        message: Box<Message>,
        ctx: &mut SimulationContext,
    ) -> Option<(u64, Box<Message>)>;

    fn connection_id(&self) -> ConnectionId;
    fn connection_type_id(&self) -> ConnectionTypeId;
}

#[derive(Copy, Clone)]
pub enum PortKind {
    In,
    Out,
    InOut,
}

#[derive(Copy, Clone)]
pub struct Port {
    pub id: PortId,
    pub kind: PortKind,
    pub conn_id: ConnectionId,

    pub rcv_mod: ModuleId,
    pub rcv_gate: GateId,
    pub rcv_port: PortId,
}
