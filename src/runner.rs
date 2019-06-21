use crate::clock::Clock;
use crate::connection::connection::Connection;
use crate::connection::mesh::ConnectionMesh;
use crate::event::TimerEvent;
use crate::id_mngmnt::id_registrar::IdRegistrar;
use crate::id_mngmnt::id_types::{GateId, ModuleId, PortId};
use crate::modules::module::{FinalizeResult, HandleContext, HandleResult, Module};

use rand::prng::XorShiftRng;
use rand::SeedableRng;

#[derive(Clone)]
pub enum Tree<T> {
    Node(T, Vec<Tree<T>>),
    Leaf(T),
}

struct ModuleMngr {
    modules: std::collections::HashMap<ModuleId, Box<Module>>,
}

impl ModuleMngr {
    fn finalize_modules_rec(
        &mut self,
        tree: &Tree<(String, ModuleId)>,
        ctx: &mut HandleContext,
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

                match self.modules.get_mut(&id).unwrap().finalize(ctx) {
                    Some(mut container_results) => {
                        local_results.results.append(&mut container_results.results);
                    }
                    None => {}
                }
            }
            Tree::Leaf((_, id)) => match self.modules.get_mut(&id).unwrap().finalize(ctx) {
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
    clock: Clock,

    timer_queue: std::collections::BinaryHeap<TimerEvent>,

    connections: ConnectionMesh,
    prng: rand::prng::XorShiftRng,

    module_tree: Tree<(String, ModuleId)>,
    modules: ModuleMngr,
}

pub fn new_runner(seed: [u8; 16]) -> Runner {
    Runner {
        clock: Clock { time: 0 },

        modules: ModuleMngr {
            modules: std::collections::HashMap::new(),
        },
        timer_queue: std::collections::BinaryHeap::new(),

        connections: ConnectionMesh {
            connections: std::collections::HashMap::new(),
            gates: std::collections::HashMap::new(),

            messages: std::collections::BinaryHeap::new(),
        },

        prng: XorShiftRng::from_seed(seed),

        module_tree: Tree::Node(("Top".to_owned(), ModuleId(0)), Vec::new()),
    }
}

impl Runner {
    pub fn add_to_tree(&mut self, tree: Tree<(String, ModuleId)>) {
        let new_top: Tree<(String, ModuleId)> = match &self.module_tree {
            Tree::Node((name, id), children) => {
                let mut new_children = Vec::new();
                for c in children {
                    new_children.push(c.clone());
                }
                new_children.push(tree);

                Tree::Node((name.clone(), *id), new_children)
            }
            Tree::Leaf(_) => {
                panic!("WHUUUUT");
            }
        };
        self.module_tree = new_top;
    }

    pub fn connect_modules(
        &mut self,
        conn: Box<Connection>,

        con_kind: crate::connection::mesh::ConnectionKind,

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

        self.connections
            .gates
            .insert(module.module_id(), std::collections::HashMap::new());

        for g in module.get_gate_ids() {
            self.connections.add_gate(module.module_id(), g);
        }

        self.modules.modules.insert(module.module_id(), module);

        Ok(())
    }

    fn process_result(&mut self, _result: HandleResult) {}

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
            let mut ctx = HandleContext {
                time: &self.clock,
                id_reg: id_reg,
                connections: &mut self.connections,
                timer_queue: &mut self.timer_queue,
                prng: &mut self.prng,
            };
            self.modules
                .modules
                .get_mut(&tmsg.recipient)
                .unwrap()
                .handle_message(tmsg.msg, tmsg.recp_gate, tmsg.recp_port, &mut ctx)
                .unwrap();
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

            let module = match self.modules.modules.get_mut(&ev.mod_id) {
                Some(m) => m,
                None => panic!(
                    "Non existent module-ID found in a timer-event: {}",
                    ev.mod_id.raw(),
                ),
            };
            let mut ctx = HandleContext {
                time: &self.clock,
                id_reg: id_reg,
                connections: &mut self.connections,
                timer_queue: &mut self.timer_queue,
                prng: &mut self.prng,
            };

            let result = module.handle_timer_event(ev.event.as_ref(), &mut ctx);
            match result {
                Err(e) => return Err(e),
                Ok(res) => {
                    self.process_result(res);
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

    fn init_modules(&mut self, id_reg: &mut IdRegistrar) {
        let mut ctx = HandleContext {
            time: &self.clock,
            id_reg: id_reg,
            connections: &mut self.connections,
            timer_queue: &mut self.timer_queue,
            prng: &mut self.prng,
        };

        self.modules.modules.iter_mut().for_each(|(_, module)| {
            module.initialize(&mut ctx);
        });
    }

    fn finalize_modules(&mut self, id_reg: &mut IdRegistrar) {
        let mut global_results = Vec::new();

        let tree = &self.module_tree;

        let mut ctx = HandleContext {
            time: &self.clock,
            id_reg: id_reg,
            connections: &mut self.connections,
            timer_queue: &mut self.timer_queue,
            prng: &mut self.prng,
        };

        match tree {
            Tree::Node(_, children) => {
                for p in children {
                    match self.modules.finalize_modules_rec(p, &mut ctx) {
                        Some(mut results) => {
                            global_results.append(&mut results.results);
                        }
                        None => {}
                    }
                }
            }
            _ => panic!("WHUTTTTT"),
        }

        for (mname, fname, val) in global_results {
            println!("{} {} {}", mname, fname, val);
        }
    }

    fn run_main_loop(
        &mut self,
        id_reg: &mut IdRegistrar,
        endtime: u64,
    ) -> Result<(), Box<std::error::Error>> {
        while self.clock.now() <= endtime {
            match self.get_next_time_to_run() {
                Some(time) => {
                    if time < self.clock.now() {
                        panic!("Nope");
                    }
                    self.clock.step(time - self.clock.now());
                }
                None => {
                    break;
                }
            }
            println!("Time: {}", self.clock.now());

            //process events and messages until no more messages are there and no more events registered for this clock time
            loop {
                let mut x = 0;
                x += self.process_events(id_reg).unwrap();
                x += self.process_messages(id_reg);
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

        let result = self.run_main_loop(id_reg, endtime);

        println!("");
        println!("##################");
        println!("");
        println!("No more messages nor events available. This simulation is over.");

        println!("Finalizing Modules");
        self.finalize_modules(id_reg);

        result
    }
}
