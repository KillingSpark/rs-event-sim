mod clock;
mod connection;
mod events;
mod id_mngmnt;
mod messages;
mod modules;
mod runner;

use event::TimerEvent;
use id_mngmnt::{id_registrar::IdRegistrar};
use modules::{module::Module, sink::Sink};
use modules::simple_module;

use clock::Clock;
use connection::connection::ConnectionMesh;
use connection::simple_connection::SimpleConnection;
use messages::text_message;
use events::{event, text_event::TextEvent};

fn setup_modules(r: &mut runner::Runner, id_reg: &mut IdRegistrar) {
    simple_module::register(id_reg);
    let te_type = id_reg.register_type("TextEvent".to_owned());
    let sc_type = id_reg.register_type("SimpleConnection".to_owned());
    let sink_type = id_reg.register_type("SinkModule".to_owned());
    id_reg.register_type("TextSignal".to_owned());

    let smod = Box::new(simple_module::new_simple_module(id_reg));

    let sink1 = Box::new( Sink{
        id: id_reg.new_id(),
        type_id: sink_type,
    });
    
    let sink2 = Box::new( Sink{
        id: id_reg.new_id(),
        type_id: sink_type,
    });

    r.add_timer_event(TimerEvent {
        time: 10,
        mod_id: smod.module_id(),
        event: Box::new(TextEvent {
            type_id: te_type,
            id: id_reg.new_id(),
            data: "StarterEvent".to_owned(),
        }),
    })
    .unwrap();

    let smod_id = smod.id;
    let s1_id = sink1.id;
    let s2_id = sink2.id;

    r.add_module(smod).unwrap();
    r.add_module(sink1).unwrap();
    r.add_module(sink2).unwrap();

    let sconn1_2 = SimpleConnection {
        buf: Vec::new(),
        delay: 2,

        id: id_reg.new_id(),
        type_id: sc_type,
    };
    let sconn1_3 = SimpleConnection {
        buf: Vec::new(),
        delay: 1,

        id: id_reg.new_id(),
        type_id: sc_type,
    };
    
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
