mod clock;
mod connection;
mod events;
mod id_mngmnt;
mod messages;
mod modules;
mod runner;

use event::TimerEvent;
use id_mngmnt::{id_registrar::IdRegistrar};
use modules::module::Module;
use modules::simple_module;
use modules::sink;

use clock::Clock;
use connection::connection::ConnectionMesh;
use connection::simple_connection;
use messages::text_message;
use events::{event, text_event};

fn setup_modules(r: &mut runner::Runner, id_reg: &mut IdRegistrar) {
    simple_module::register(id_reg);
    sink::register(id_reg);
    text_event::register(id_reg);
    text_message::register(id_reg);
    simple_connection::register(id_reg);

    let smod = Box::new(simple_module::new_simple_module(id_reg));

    let sink1 = Box::new(sink::new_sink(id_reg));
    let sink2 = Box::new(sink::new_sink(id_reg));

    r.add_timer_event(TimerEvent {
        time: 10,
        mod_id: smod.module_id(),
        event: Box::new(text_event::new_text_msg(id_reg, "StarterEvent".to_owned())),
    })
    .unwrap();

    let smod_id = smod.id;
    let s1_id = sink1.id;
    let s2_id = sink2.id;

    r.add_module(smod).unwrap();
    r.add_module(sink1).unwrap();
    r.add_module(sink2).unwrap();

    let sconn1_2 = simple_connection::new_simple_connection(id_reg, 2);
    let sconn1_3 = simple_connection::new_simple_connection(id_reg, 1);
    
    r.connections.add_gate(smod_id, simple_module::OUT_GATE);
    r.connections.add_gate(s1_id, 0);
    r.connections.add_gate(s2_id, 0);

    r.connect_modules(Box::new(sconn1_2), smod_id, simple_module::OUT_GATE, 0, s1_id, 0, 0)
        .unwrap();
    r.connect_modules(Box::new(sconn1_3), smod_id, simple_module::OUT_GATE, 1, s2_id, 0, 0)
        .unwrap();
}

fn main() {
    println!("Hello, world!");
    let mut r = runner::Runner {
        clock: Clock { time: 0 },

        modules: std::collections::HashMap::new(),
        timer_queue: std::collections::BinaryHeap::new(),

        connections: ConnectionMesh {
            connections: std::collections::HashMap::new(),
            gates: std::collections::HashMap::new(),

            messages: std::collections::BinaryHeap::new(),
        },
    };

    let mut id_reg = IdRegistrar {
        last_id: 0,
        last_type_id: 0,
        type_ids: std::collections::HashMap::new(),
        type_ids_reverse: std::collections::HashMap::new(),
    };

    setup_modules(&mut r, &mut id_reg);

    r.run(&mut id_reg, 200).unwrap();
}
