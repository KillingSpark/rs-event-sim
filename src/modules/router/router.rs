use crate::event::Event;
use crate::id_mngmnt::id_registrar::IdRegistrar;
use crate::id_mngmnt::id_types::{GateId, ModuleId, ModuleTypeId, PortId};
use crate::messages::message::Message;
use crate::modules::module::{FinalizeResult, HandleContext, HandleResult, Module};


use crate::connection::mesh::ConnectionKind;
use crate::connection::simple_connection;
use crate::modules::container;
use crate::modules::queue;
use crate::modules::router::rate_puller;
use crate::modules::splitter;
use crate::runner::Runner;
use crate::runner;

pub struct Router {
    type_id: ModuleTypeId,
    id: ModuleId,
    name: String,

    routing_table: std::collections::HashMap<PortId, PortId>,
}

//messages get sent out here (and buffered)
pub const OUT_GATE: GateId = GateId(0);

//messages are received here, processed and sent to the out-buffer on the respective port
pub const IN_GATE: GateId = GateId(1);

pub static TYPE_STR: &str = "RouterModule";

pub fn register(id_reg: &mut IdRegistrar) {
    id_reg.register_type(TYPE_STR.to_owned());
    queue::queue::register(id_reg);
    container::register(id_reg);
    rate_puller::register(id_reg);
    splitter::register(id_reg);
    simple_connection::register(id_reg);
}

fn new(
    id_reg: &mut crate::id_mngmnt::id_registrar::IdRegistrar,
    name: String,
    routing_table: std::collections::HashMap<PortId, PortId>,
) -> Router {
    Router {
        id: id_reg.new_module_id(),
        type_id: id_reg.lookup_module_id(TYPE_STR.to_owned()).unwrap(),
        name: name,

        routing_table: routing_table,
    }
}

pub const ROUTER_GATE_OUTER: GateId = GateId(0);
pub const ROUTER_GATE_INNER: GateId = GateId(1);

pub fn make_router(
    r: &mut Runner,
    id_reg: &mut IdRegistrar,
    port_count: u64,
    name: String,
    routing_table: std::collections::HashMap<PortId, PortId>,
) -> (ModuleId, runner::Tree<(String, ModuleId)>) {
    let container = Box::new(container::new_module_container(id_reg, name.clone(), vec![(ROUTER_GATE_INNER, ROUTER_GATE_OUTER)]));
    let container_id = container.module_id();

    let router = Box::new(new(id_reg, "RouterCore".to_owned(), routing_table));
    let router_id = router.id;

    let split = Box::new(splitter::new(id_reg, "Splitter".to_owned()));
    let split_id = split.module_id();

    let mut children = vec![
        runner::Tree::Leaf((router.name(), router_id)),
        runner::Tree::Leaf((split.name(), split_id)),
    ];

    r.add_module(container).unwrap();
    r.add_module(router).unwrap();
    r.add_module(split).unwrap();

    for idx in 0..port_count {
        let q = Box::new(queue::queue::new(id_reg, "Buffer".to_owned()));
        let queue_id = q.module_id();

        let rate = Box::new(rate_puller::new(id_reg, "RateLimiter".to_owned(), 1));
        let rate_id = rate.module_id();

        children.push(runner::Tree::Leaf((q.name(), queue_id)));
        children.push(runner::Tree::Leaf((rate.name(), rate_id)));

        r.add_module(rate).unwrap();
        r.add_module(q).unwrap();

        //provides interfaces to the outer gate of the enclosing container
        //splits into two ways
        let split_outer_con = Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0));

        r.connect_modules(
            split_outer_con,
            ConnectionKind::Bidrectional,
            split_id,
            splitter::IN_OUT_GATE,
            PortId(idx),
            container_id,
            ROUTER_GATE_INNER,
            PortId(idx),
        )
        .unwrap();

        // 1) from outside into router directly
        let split_router_con = Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0));

        r.connect_modules(
            split_router_con,
            ConnectionKind::Onedirectional,
            split_id,
            splitter::SPLIT_OUT_GATE,
            PortId(idx),
            router_id,
            IN_GATE,
            PortId(idx),
        )
        .unwrap();

        // 2) from router to outside through a buffer and a rate-limited puller
        let router_queue_con = Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0));
        let queue_trig_con = Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0));
        let queue_rate_con = Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0));
        let rate_split_con = Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0));

        r.connect_modules(
            router_queue_con,
            ConnectionKind::Onedirectional,
            router_id,
            OUT_GATE,
            PortId(idx),
            queue_id,
            queue::queue::IN_GATE,
            PortId(0),
        )
        .unwrap();

        r.connect_modules(
            queue_trig_con,
            ConnectionKind::Onedirectional,
            rate_id,
            rate_puller::TRIG_GATE,
            PortId(0),
            queue_id,
            queue::queue::TRIGG_GATE,
            PortId(0),
        )
        .unwrap();

        r.connect_modules(
            queue_rate_con,
            ConnectionKind::Onedirectional,
            queue_id,
            queue::queue::OUT_GATE,
            PortId(0),
            rate_id,
            splitter::SPLIT_IN_GATE,
            PortId(0),
        )
        .unwrap();

        r.connect_modules(
            rate_split_con,
            ConnectionKind::Onedirectional,
            rate_id,
            rate_puller::OUT_GATE,
            PortId(0),
            split_id,
            splitter::SPLIT_IN_GATE,
            PortId(idx),
        )
        .unwrap();
    }

    (container_id, runner::Tree::Node((name, container_id), children))
}

impl Module for Router {
    fn get_gate_ids(&self) -> Vec<GateId> {
        vec![OUT_GATE, IN_GATE]
    }

    fn handle_message(
        &mut self,
        msg: Box<Message>,
        gate: GateId,
        port: PortId,
        ctx: &mut HandleContext,
    ) -> Result<HandleResult, Box<std::error::Error>> {
        match gate {
            IN_GATE => match self.routing_table.get(&port) {
                Some(out_port) => {
                    ctx.connections
                        .send_message(msg, self.id, OUT_GATE, *out_port, &mut ctx.mctx);
                }
                None => {
                    //println!(
                    //    "Router: {} Didnt know where to send message on port: {}",
                    //    &self.name, port.0
                    //);
                }
            },
            OUT_GATE => panic!("Should never receive message on OUT_GATE"),
            _ => panic!("Should never receive message on other gate"),
        }

        Ok(HandleResult {})
    }

    fn handle_timer_event(
        &mut self,
        _ev: &Event,
        _ctx: &mut HandleContext,
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

    fn initialize(&mut self, _ctx: &mut HandleContext) {}

    fn finalize(&mut self, _ctx: &mut HandleContext) -> Option<FinalizeResult> {
        println!("Finalize Queue: {}", &self.name);
        None
    }
}
