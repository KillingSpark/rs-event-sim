mod clock;
mod connection;
mod events;
mod id_mngmnt;
mod messages;
mod modules;
mod runner;

use id_mngmnt::id_registrar::IdRegistrar;
use id_mngmnt::id_types::PortId;
use modules::simple_module;
use modules::sink;

use connection::simple_connection;
use events::{event, text_event};
use messages::text_message;


fn register_needed_types(id_reg: &mut IdRegistrar) {
    simple_module::register(id_reg);
    sink::register(id_reg);
    text_event::register(id_reg);
    text_message::register(id_reg);
    simple_connection::register(id_reg);
}

fn setup_modules(r: &mut runner::Runner, id_reg: &mut IdRegistrar) {
    register_needed_types(id_reg);

    let smod = Box::new(simple_module::new_simple_module(id_reg));
    let sink1 = Box::new(sink::new_sink(id_reg));
    let sink2 = Box::new(sink::new_sink(id_reg));

    

    let smod_id = smod.id;
    let s1_id = sink1.id;
    let s2_id = sink2.id;

    r.add_module(smod).unwrap();
    r.add_module(sink1).unwrap();
    r.add_module(sink2).unwrap();

    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 1, 10, 0)),
        smod_id,
        simple_module::OUT_GATE,
        PortId(0),
        s1_id,
        sink::IN_GATE,
        PortId(0),
    )
    .unwrap();
    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 1, 0, 0)),
        smod_id,
        simple_module::OUT_GATE,
        PortId(1),
        s2_id,
        sink::IN_GATE,
        PortId(0),
    )
    .unwrap();
}

fn main() {
    println!("Starting simulation");
    let seed:[u8; 16] = [40,157,153,238,231,98,7,241,206,84,162,233,247,101,104,215];

    let mut r = runner::new_runner(seed);

    let mut id_reg = IdRegistrar {
        last_id: 0,
        last_type_id: 0,
        type_ids: std::collections::HashMap::new(),
        type_ids_reverse: std::collections::HashMap::new(),
    };

    setup_modules(&mut r, &mut id_reg);

    r.run(&mut id_reg, 200).unwrap();
}
