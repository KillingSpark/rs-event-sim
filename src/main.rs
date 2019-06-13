mod clock;
mod connection;
mod event;
mod id_registrar;
mod message;
mod module;
mod runner;
mod simple_connection;
mod simple_module;
mod text_event;
mod text_message;

use event::TimerEvent;
use id_registrar::IdRegistrar;
use module::Module;

use clock::Clock;
use simple_module::SimpleModule;
use text_event::TextEvent;

fn setup_modules(r: &mut runner::Runner, id_reg: &mut IdRegistrar) {
    let te_type = id_reg.register_type("TextEvent".to_owned());
    let sm_type = id_reg.register_type("SimpleModule".to_owned());
    let sc_type = id_reg.register_type("SimpleConnection".to_owned());
    id_reg.register_type("TextSignal".to_owned());

    let smod = Box::new(SimpleModule {
        mod_id: id_reg.new_id(),
        type_id: sm_type,
    });
    r.add_timer_event(TimerEvent {
        time: 10,
        mod_id: smod.module_id(),
        event: Box::new(TextEvent {
            type_id: te_type,
            ev_id: id_reg.new_id(),
            data: "StarterEvent".to_owned(),
        }),
    })
    .unwrap();

    let smod2 = Box::new(SimpleModule {
        mod_id: id_reg.new_id(),
        type_id: sm_type,
    });
    r.add_timer_event(TimerEvent {
        time: 15,
        mod_id: smod2.module_id(),
        event: Box::new(TextEvent {
            type_id: te_type,
            ev_id: id_reg.new_id(),
            data: "StarterEvent2".to_owned(),
        }),
    })
    .unwrap();

    let smod_id = smod.mod_id;
    let smod2_id = smod2.mod_id;

    r.add_module(smod).unwrap();
    r.add_module(smod2).unwrap();

    let sconn = simple_connection::SimpleConnection {
        buf: Vec::new(),

        conn_id: id_reg.new_id(),
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

        connections: crate::connection::ConnectionMesh {
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
