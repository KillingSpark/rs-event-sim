mod clock;
mod connection;
mod events;
mod id_registrar;
mod messages;
mod modules;
mod runner;
mod id_types;

use event::TimerEvent;
use id_registrar::IdRegistrar;
use modules::{module::Module, simple_module::SimpleModule};

use clock::Clock;
use connection::connection::ConnectionMesh;
use connection::simple_connection::SimpleConnection;
use messages::text_message;
use events::{event, text_event::TextEvent};

fn setup_modules(r: &mut runner::Runner, id_reg: &mut IdRegistrar) {
    let te_type = id_reg.register_type("TextEvent".to_owned());
    let sm_type = id_reg.register_type("SimpleModule".to_owned());
    let sc_type = id_reg.register_type("SimpleConnection".to_owned());
    id_reg.register_type("TextSignal".to_owned());

    let smod = Box::new(SimpleModule {
        id: id_reg.new_id(),
        type_id: sm_type,
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

    let smod2 = Box::new(SimpleModule {
        id: id_reg.new_id(),
        type_id: sm_type,
    });
    r.add_timer_event(TimerEvent {
        time: 15,
        mod_id: smod2.module_id(),
        event: Box::new(TextEvent {
            type_id: te_type,
            id: id_reg.new_id(),
            data: "StarterEvent2".to_owned(),
        }),
    })
    .unwrap();

    let smod_id = smod.id;
    let smod2_id = smod2.id;

    r.add_module(smod).unwrap();
    r.add_module(smod2).unwrap();

    let sconn = SimpleConnection {
        buf: Vec::new(),

        id: id_reg.new_id(),
        type_id: sc_type,
    };
    r.connect_modules(Box::new(sconn), smod_id, 0, smod2_id)
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
            connections_in: std::collections::HashMap::new(),
            connections_out: std::collections::HashMap::new(),
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
