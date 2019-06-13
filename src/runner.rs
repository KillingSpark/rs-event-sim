use crate::clock::Clock;
use crate::connection::connection::{Connection, ConnectionMesh};
use crate::event::TimerEvent;
use crate::id_registrar::IdRegistrar;
use crate::modules::module::{HandleContext, HandleResult, Module};

pub struct Runner {
    pub clock: Clock,

    pub modules: std::collections::HashMap<u64, Box<Module>>,
    pub timer_queue: std::collections::BinaryHeap<TimerEvent>,

    pub connections: ConnectionMesh,
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
        mod_out: u64,
        out_port: u64,
        mod_in: u64,
    ) -> Result<(), Box<std::error::Error>> {
        //check validity of modules
        match self.modules.get(&mod_in) {
            None => panic!("Tried to connect module that does not exist: {}", mod_in),
            Some(_) => {}
        }
        match self.modules.get(&mod_out) {
            None => panic!("Tried to connect module that does not exist: {}", mod_out),
            Some(_) => {}
        }
        match self.connections.connections.get(&conn.connection_id()) {
            Some(_) => panic!(
                "Tried to insert connection: {} that already exists",
                conn.connection_id()
            ),
            None => {}
        }

        //handoff to connection mesh
        self.connections
            .connect_modules(conn, mod_out, out_port, mod_in)
    }

    pub fn add_module(&mut self, module: Box<Module>) -> Result<(), Box<std::error::Error>> {
        match self.modules.get(&module.module_id()) {
            Some(_) => {
                panic!(
                    "Tried to add module with already existing module_id: {}",
                    module.module_id()
                );
            }
            None => {}
        }
        self.connections
            .connections_out
            .insert(module.module_id(), std::collections::HashMap::new());
        self.modules.insert(module.module_id(), module);
        Ok(())
    }

    fn process_result(&mut self, result: HandleResult) {
        //push all new timer events in the queue
        match result.timer_events {
            None => {}
            Some(events) => {
                for ev in events {
                    self.timer_queue.push(ev);
                }
            }
        }
    }

    //returns how many messages were found
    fn process_messages(&mut self, id_reg: &mut IdRegistrar) -> u64 {
        let mut msg_counter = 0;

        let mut collected_msgs: Vec<(u64, Vec<Box<crate::messages::message::Message>>)> = Vec::new();

        for (_, c) in &mut self.connections.connections {
            let msgs = match c.pull() {
                Ok(m) => match m {
                    None => continue,
                    Some(m) => m,
                },
                Err(_) => panic!("Error while pulling messages"),
            };
            collected_msgs.push((c.connection_id(), msgs));
        }

        for (conn_id, msgs) in collected_msgs {
            for msg in msgs {
                let recipient = self
                    .modules
                    .get_mut(self.connections.connections_in.get(&conn_id).unwrap())
                    .unwrap();
                let mut ctx = HandleContext {
                    time: &self.clock,
                    id_reg: id_reg,
                    connections: &mut self.connections,
                };
                let result = recipient.handle_message(msg.as_ref(), &mut ctx);

                match result {
                    Ok(res) => {
                        self.process_result(res);
                    }
                    Err(_) => panic!("Buuhuu"),
                }

                msg_counter += 1;
            }
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
                    ev.mod_id
                ),
            };
            let mut ctx = HandleContext {
                time: &self.clock,
                id_reg: id_reg,
                connections: &mut self.connections,
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

    pub fn run(
        &mut self,
        id_reg: &mut IdRegistrar,
        endtime: i64,
    ) -> Result<(), Box<std::error::Error>> {
        while self.clock.now() <= endtime {
            println!("Time: {}", self.clock.now());
            match self.timer_queue.peek() {
                Some(ev) => {
                    if ev.time > self.clock.now() {
                        self.clock.step(ev.time - self.clock.now());
                    }
                }
                None => {
                    self.clock.step(endtime - self.clock.now() + 1);
                }
            }

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
