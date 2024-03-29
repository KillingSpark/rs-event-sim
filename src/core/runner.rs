use crate::core::clock;
use crate::core::connection::connection::Connection;
use crate::core::connection::connection::PortKind;
use crate::core::connection::mesh;
use crate::core::connection::mesh::ConnectionMesh;
use crate::core::contexts::{EventHandleContext, SimulationContext};
use crate::core::events::event::TimerEvent;
use crate::core::id_mngmnt::id_registrar::IdRegistrar;
use crate::core::id_mngmnt::id_types::{GateId, ModuleId, PortId};
use crate::core::messages::message::Message;
use crate::core::modules::module::{FinalizeResult, Module};

use rand::prng::XorShiftRng;
use rand::SeedableRng;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::string::String;

#[derive(Clone)]
pub enum Tree<T> {
    Node(T, Vec<Tree<T>>),
    Leaf(T),
}

struct ModuleMngr {
    modules: std::collections::HashMap<ModuleId, Rc<RefCell<Box<Module>>>>,
}

impl ModuleMngr {
    fn finalize_modules(&mut self, tree: &Tree<(String, ModuleId)>, ctx: &mut EventHandleContext) {
        let mut global_results = Vec::new();

        match tree {
            Tree::Node(_, children) => {
                for p in children {
                    match self.finalize_modules_rec(p, ctx) {
                        Some(mut results) => {
                            global_results.append(&mut results.results);
                        }
                        None => {}
                    }
                }
            }
            _ => panic!("WHUTTTTT"),
        }

        //for (mname, fname, val) in global_results {
        //    println!("{} {} {}", mname, fname, val);
        //}
    }

    fn finalize_modules_rec(
        &mut self,
        tree: &Tree<(String, ModuleId)>,
        ctx: &mut EventHandleContext,
    ) -> Option<FinalizeResult> {
        let mut local_results = FinalizeResult {
            results: Vec::new(),
        };

        match tree {
            Tree::Node((name, id), children) => {
                //descend then finalize
                for c in children {
                    match self.finalize_modules_rec(c, ctx) {
                        Some(res) => {
                            let mut renamed = res
                                .results
                                .iter()
                                .map(|(mname, fname, val)| {
                                    let mut new_name = name.clone();
                                    new_name.push_str(mname);
                                    (new_name, fname.clone(), val.clone())
                                })
                                .collect();
                            local_results.results.append(&mut renamed);
                        }
                        None => {}
                    }
                }

                match self
                    .modules
                    .get_mut(&id)
                    .unwrap()
                    .borrow_mut()
                    .finalize(ctx)
                {
                    Some(mut container_results) => {
                        local_results.results.append(&mut container_results.results);
                    }
                    None => {}
                }
            }
            Tree::Leaf((_, id)) => match self
                .modules
                .get_mut(&id)
                .unwrap()
                .borrow_mut()
                .finalize(ctx)
            {
                Some(mut r) => {
                    local_results.results.append(&mut r.results);
                }
                None => {}
            },
        }

        if local_results.results.len() > 0 {
            Some(local_results)
        } else {
            None
        }
    }
}

pub struct Runner {
    clock: clock::Clock,

    timer_queue: std::collections::BinaryHeap<TimerEvent>,
    msg_buffer: std::collections::VecDeque<(Box<Message>, GateId, PortId)>,

    pub connections: ConnectionMesh,
    prng: rand::prng::XorShiftRng,

    pub module_forest: Vec<Tree<(String, ModuleId)>>,
    modules: ModuleMngr,
}

pub fn new_runner(seed: [u8; 16]) -> Runner {
    Runner {
        clock: clock::new(),

        modules: ModuleMngr {
            modules: std::collections::HashMap::new(),
        },
        timer_queue: std::collections::BinaryHeap::new(),
        msg_buffer: std::collections::VecDeque::new(),

        connections: ConnectionMesh {
            connections: std::collections::HashMap::new(),
            gates: std::collections::HashMap::new(),

            messages: std::collections::BinaryHeap::new(),
            messages_now: std::collections::VecDeque::new(),
        },

        prng: XorShiftRng::from_seed(seed),

        module_forest: Vec::new(),
    }
}

impl Runner {
    pub fn init_modules(&mut self, id_reg: &mut IdRegistrar) {
        let mut ctx = EventHandleContext {
            msgs_to_send: &mut self.msg_buffer,
            timer_queue: &mut self.timer_queue,

            mctx: SimulationContext {
                prng: &mut self.prng,
                id_reg: id_reg,
                time: &self.clock,
            },
        };

        for (_, module) in &mut self.modules.modules {
            let mut module = module.borrow_mut();
            let mod_id = (*module).module_id();

            let mut gate_map = std::collections::HashMap::new();
            for ((m, g, p), port) in &self.connections.gates {
                if *m == mod_id {
                    if gate_map.get(g).is_none() {
                        gate_map.insert(*g, std::collections::HashMap::new());
                    }
                    gate_map.get_mut(g).unwrap().insert(*p, *port);
                }
            }

            (*module).initialize(&gate_map, &mut ctx);
            while ctx.msgs_to_send.len() > 0 {
                let (msg, gate, port) = ctx.msgs_to_send.pop_front().unwrap();
                self.connections
                    .send_message(msg, module.module_id(), gate, port, &mut ctx.mctx);
            }
        }
    }

    pub fn finalize_modules(&mut self, id_reg: &mut IdRegistrar) {
        let mut ctx = EventHandleContext {
            msgs_to_send: &mut self.msg_buffer,
            timer_queue: &mut self.timer_queue,

            mctx: SimulationContext {
                prng: &mut self.prng,
                id_reg: id_reg,
                time: &self.clock,
            },
        };

        self.modules.finalize_modules(
            &Tree::Node(("Top".to_owned(), ModuleId(0)), self.module_forest.clone()),
            &mut ctx,
        );
    }

    pub fn add_to_tree(&mut self, tree: Tree<(String, ModuleId)>) {
        self.module_forest.push(tree);
    }

    pub fn connect_modules(
        &mut self,
        conn: Box<Connection>,

        con_kind: mesh::ConnectionKind,

        mod_out: ModuleId,
        gate_out: GateId,
        out_port: PortId,

        mod_in: ModuleId,
        gate_in: GateId,
        in_port: PortId,
    ) -> Result<(), Box<std::error::Error>> {
        //check validity of modules
        match self.modules.modules.get(&mod_in) {
            None => panic!(
                "Tried to connect module that does not exist: {}",
                mod_in.raw()
            ),
            Some(_) => {}
        }
        match self.modules.modules.get(&mod_out) {
            None => panic!(
                "Tried to connect module that does not exist: {}",
                mod_out.raw(),
            ),
            Some(_) => {}
        }
        match self.connections.connections.get(&conn.connection_id()) {
            Some(_) => panic!(
                "Tried to insert connection: {} that already exists",
                conn.connection_id().raw()
            ),
            None => {}
        }

        //handoff to connection mesh
        self.connections.connect_modules(
            conn, con_kind, mod_out, gate_out, out_port, mod_in, gate_in, in_port,
        )
    }

    pub fn add_module(&mut self, module: Box<Module>) -> Result<(), Box<std::error::Error>> {
        match self.modules.modules.get(&module.module_id()) {
            Some(_) => {
                panic!(
                    "Tried to add module with already existing module_id: {}",
                    module.module_id().raw()
                );
            }
            None => {}
        }

        self.modules
            .modules
            .insert(module.module_id(), Rc::new(RefCell::new(module)));

        Ok(())
    }

    //returns how many messages were found
    fn process_messages(&mut self, id_reg: &mut IdRegistrar) -> u64 {
        let mut msg_counter = 0;
        loop {
            match self.connections.messages.peek() {
                Some(tmsg) => {
                    if tmsg.time > self.clock.now() {
                        break;
                    }
                }
                None => {
                    break;
                }
            }

            let tmsg = self.connections.messages.pop().unwrap();
            let mut ctx = EventHandleContext {
                msgs_to_send: &mut self.msg_buffer,
                timer_queue: &mut self.timer_queue,

                mctx: SimulationContext {
                    prng: &mut self.prng,
                    id_reg: id_reg,
                    time: &self.clock,
                },
            };

            self.modules
                .modules
                .get_mut(&tmsg.recipient)
                .unwrap()
                .borrow_mut()
                .handle_message(tmsg.msg, tmsg.recp_gate, tmsg.recp_port, &mut ctx)
                .unwrap();

            while ctx.msgs_to_send.len() > 0 {
                let (msg, gate, port) = ctx.msgs_to_send.pop_front().unwrap();
                self.connections
                    .send_message(msg, tmsg.recipient, gate, port, &mut ctx.mctx);
            }

            msg_counter += 1;
        }

        loop {
            if self.connections.messages_now.len() == 0 {
                break;
            }

            let tmsg = self.connections.messages_now.pop_front().unwrap();
            let mut ctx = EventHandleContext {
                msgs_to_send: &mut self.msg_buffer,
                timer_queue: &mut self.timer_queue,

                mctx: SimulationContext {
                    prng: &mut self.prng,
                    id_reg: id_reg,
                    time: &self.clock,
                },
            };
            self.modules
                .modules
                .get_mut(&tmsg.recipient)
                .unwrap()
                .borrow_mut()
                .handle_message(tmsg.msg, tmsg.recp_gate, tmsg.recp_port, &mut ctx)
                .unwrap();

            while ctx.msgs_to_send.len() > 0 {
                let (msg, gate, port) = ctx.msgs_to_send.pop_front().unwrap();
                self.connections
                    .send_message(msg, tmsg.recipient, gate, port, &mut ctx.mctx);
            }
            msg_counter += 1;
        }

        msg_counter
    }

    fn process_events(&mut self, id_reg: &mut IdRegistrar) -> Result<u64, Box<std::error::Error>> {
        let mut events_counter = 0;
        loop {
            match self.timer_queue.peek() {
                Some(ev) => {
                    if ev.time > self.clock.now() {
                        break;
                    }
                }
                None => {
                    break;
                }
            }

            let ev = self.timer_queue.pop().unwrap();

            let mut module = match self.modules.modules.get_mut(&ev.mod_id) {
                Some(m) => m.borrow_mut(),
                None => panic!(
                    "Non existent module-ID found in a timer-event: {}",
                    ev.mod_id.raw(),
                ),
            };
            let mut ctx = EventHandleContext {
                msgs_to_send: &mut self.msg_buffer,
                timer_queue: &mut self.timer_queue,

                mctx: SimulationContext {
                    prng: &mut self.prng,
                    id_reg: id_reg,
                    time: &self.clock,
                },
            };

            let result = module.handle_timer_event(ev.event.as_ref(), &mut ctx);
            match result {
                Err(e) => return Err(e),
                Ok(_) => {
                    while ctx.msgs_to_send.len() > 0 {
                        let (msg, gate, port) = ctx.msgs_to_send.pop_front().unwrap();

                        self.connections.send_message(
                            msg,
                            module.module_id(),
                            gate,
                            port,
                            &mut ctx.mctx,
                        );
                    }
                }
            }

            events_counter += 1;
        }
        Ok(events_counter)
    }

    fn get_next_time_to_run(&self) -> Option<u64> {
        let mut min = u64::max_value();
        let mut at_least_one = false;
        match self.timer_queue.peek() {
            Some(ev) => {
                if ev.time <= min {
                    min = ev.time;
                    at_least_one = true;
                }
            }
            None => {}
        }

        match self.connections.messages.peek() {
            Some(msg) => {
                if msg.time <= min {
                    min = msg.time;
                    at_least_one = true;
                }
            }
            None => {}
        }
        if at_least_one {
            Some(min)
        } else {
            None
        }
    }

    fn run_main_loop(
        &mut self,
        id_reg: &mut IdRegistrar,
        endtime: u64,
    ) -> Result<(), Box<std::error::Error>> {
        let mut percentage_time_passed = 0;
        let mut msgs_processed_total = 0;
        let mut msgs_processed = 0;
        let mut events_processed_total = 0;
        let mut events_processed = 0;
        let mut time = std::time::Instant::now();

        while self.clock.now() <= endtime {
            match self.get_next_time_to_run() {
                Some(time) => {
                    if time < self.clock.now() {
                        panic!("Nope");
                    }
                    self.clock.set(time).unwrap();
                }
                None => {
                    break;
                }
            }
            if 100 * self.clock.now() / endtime > percentage_time_passed {
                percentage_time_passed = 100 * self.clock.now() / endtime;
                println!("Time: {}, {}%", self.clock.now(), percentage_time_passed);
                println!(
                    "Msgs: {}, Events: {}",
                    msgs_processed_total, events_processed_total
                );

                let new_time = std::time::Instant::now();
                let secs = new_time.duration_since(time).as_secs();
                println!("Real seconds passed: {}", secs);
                time = new_time;
                if secs > 0 {
                    println!(
                        "Msgs/s: {}, Events/s: {}",
                        msgs_processed / secs,
                        events_processed / secs
                    );
                    msgs_processed = 0;
                    events_processed = 0;
                }

                println!(
                    "Msgs in queue: {}, Events in queue: {}",
                    self.connections.messages.len(),
                    self.timer_queue.len()
                );

                println!("     ");
            }

            //process events and messages until no more messages are there and no more events registered for this clock time

            loop {
                let evs = self.process_events(id_reg).unwrap();
                let msgs = self.process_messages(id_reg);

                let x = evs + msgs;
                events_processed_total += evs;
                events_processed += evs;
                msgs_processed_total += msgs;
                msgs_processed += msgs;
                if x == 0 {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn run(
        &mut self,
        id_reg: &mut IdRegistrar,
        endtime: u64,
    ) -> Result<(), Box<std::error::Error>> {
        println!("Initializing Modules");
        self.init_modules(id_reg);

        println!("Running main loop");
        println!("");
        println!("##################");
        println!("");

        let start = std::time::Instant::now();

        let result = self.run_main_loop(id_reg, endtime);

        let end = std::time::Instant::now();

        let duration = end.duration_since(start);

        println!("");
        println!("##################");
        println!("");
        println!("No more messages nor events available. This simulation is over.");
        println!("Running took: {:?}", duration);

        println!("Finalizing Modules");
        self.finalize_modules(id_reg);

        result
    }

    #[allow(dead_code)]
    pub fn print_as_dot(&self, id: Option<ModuleId>, target: &mut Write) {
        target.write("digraph {\n".as_bytes()).unwrap();
        let mut modules = Vec::new();

        match id {
            Some(id) => {
                for m in &self.module_forest {
                    match find_node(m, id) {
                        Some(node) => {
                            print_parent_as_dot("\t", node, target, &mut modules, 0, 2);
                            break;
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }
            None => {
                for m in &self.module_forest {
                    print_parent_as_dot("\t", m, target, &mut modules, 0, 1);
                }
            }
        }

        for ((mod_id, gate_id, port_id), port) in &self.connections.gates {
            if !(modules.contains(mod_id) && modules.contains(&port.rcv_mod)) {
                continue;
            }
            match port.kind {
                PortKind::In => { /*ignore */ }
                PortKind::Out => {
                    target
                        .write(
                            format!(
                                "\t{} -> {}[label=\"M{}G{}P{}, M{}G{}P{}\"];\n",
                                mod_id.raw(),
                                port.rcv_mod.raw(),
                                mod_id.raw(),
                                gate_id.0,
                                port_id.0,
                                port.rcv_mod.raw(),
                                port.rcv_gate.0,
                                port.rcv_port.0,
                            )
                            .as_bytes(),
                        )
                        .unwrap();
                }
                PortKind::InOut => {
                    if *mod_id < port.rcv_mod {
                        target
                            .write(
                                format!(
                                    "\t{} -> {}[dir=\"both\",label=\"M{}G{}P{}, M{}G{}P{}\"];\n",
                                    mod_id.raw(),
                                    port.rcv_mod.raw(),
                                    mod_id.raw(),
                                    gate_id.0,
                                    port_id.0,
                                    port.rcv_mod.raw(),
                                    port.rcv_gate.0,
                                    port.rcv_port.0,
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                    }
                }
            }
        }
        target.write("}\n".as_bytes()).unwrap();
    }
}

fn find_node(
    tree: &Tree<(String, ModuleId)>,
    find_id: ModuleId,
) -> Option<&Tree<(String, ModuleId)>> {
    match tree {
        Tree::Node((_, id), children) => {
            if find_id == *id {
                return Some(tree);
            }
            for c in children {
                match find_node(c, find_id) {
                    Some(t) => return Some(t),
                    None => continue,
                }
            }
            return None;
        }
        Tree::Leaf((_, id)) => {
            if find_id == *id {
                return Some(tree);
            }
            return None;
        }
    };
}

#[allow(dead_code)]
pub fn print_parent_as_dot(
    prefix: &str,
    tree: &Tree<(String, ModuleId)>,
    target: &mut Write,
    modules: &mut Vec<ModuleId>,
    level: u64,
    max_level: u64,
) {
    if level == max_level {
        return;
    }
    match tree {
        Tree::Node((name, id), children) => {
            modules.push(*id);
            target
                .write(
                    format!(
                        "{}subgraph cluster_{} {{ \n{}label=\"{}{}\";\n",
                        prefix,
                        id.raw(),
                        prefix,
                        id.raw(),
                        name
                    )
                    .as_bytes(),
                )
                .unwrap();
            target
                .write(format!("{}{}[label={}{}];\n", prefix, id.raw(), name, id.raw()).as_bytes())
                .unwrap();

            for c in children {
                let mut new_prefix = "\t".to_owned();
                new_prefix.push_str(prefix);
                print_parent_as_dot(
                    new_prefix.as_str(),
                    c,
                    target,
                    modules,
                    level + 1,
                    max_level,
                );
            }

            target.write(format!("{} }}\n", prefix).as_bytes()).unwrap();
        }
        Tree::Leaf((name, id)) => {
            modules.push(*id);
            target
                .write(format!("{}{}[label={}{}];\n", prefix, id.raw(), name, id.raw()).as_bytes())
                .unwrap();
        }
    }
}
