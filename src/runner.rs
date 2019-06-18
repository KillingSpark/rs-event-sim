use crate::clock::Clock;
use crate::connection::connection::Connection;
use crate::connection::mesh::ConnectionMesh;
use crate::event::TimerEvent;
use crate::id_mngmnt::id_registrar::IdRegistrar;
use crate::id_mngmnt::id_types::{GateId, ModuleId, PortId};
use crate::modules::module::{HandleContext, HandleResult, Module};

pub struct Runner {
    pub clock: Clock,

    pub modules: std::collections::HashMap<ModuleId, Box<Module>>,
    pub timer_queue: std::collections::BinaryHeap<TimerEvent>,

    pub connections: ConnectionMesh,
    pub prng: rand::prng::XorShiftRng,
}

impl Runner {
    pub fn add_timer_event(&mut self, tev: TimerEvent) -> Result<(), Box<std::error::Error>> {
        if tev.time < self.clock.now() {
            panic!("Tried to insert event in the past")
        }

        self.timer_queue.push(tev);
        Ok(())
    }

    pub fn connect_modules(
        &mut self,
        conn: Box<Connection>,

        mod_out: ModuleId,
        gate_out: GateId,
        out_port: PortId,

        mod_in: ModuleId,
        gate_in: GateId,
        in_port: PortId,
    ) -> Result<(), Box<std::error::Error>> {
        //check validity of modules
        match self.modules.get(&mod_in) {
            None => panic!(
                "Tried to connect module that does not exist: {}",
                mod_in.raw()
            ),
            Some(_) => {}
        }
        match self.modules.get(&mod_out) {
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
        self.connections
            .connect_modules(conn, mod_out, gate_out, out_port, mod_in, gate_in, in_port)
    }

    pub fn add_module(&mut self, module: Box<Module>) -> Result<(), Box<std::error::Error>> {
        match self.modules.get(&module.module_id()) {
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

        self.modules.insert(module.module_id(), module);

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
                .get_mut(&tmsg.recipient)
                .unwrap()
                .handle_message(tmsg.msg.as_ref(), tmsg.recp_gate, tmsg.recp_port, &mut ctx)
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

            let module = match self.modules.get_mut(&ev.mod_id) {
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

    pub fn run(
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
                    println!("");
                    println!("##################");
                    println!("");
                    println!("No more messages nor events available. This simulation is over.");
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
}
