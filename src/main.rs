mod clock;
mod connection;
mod events;
mod id_mngmnt;
mod messages;
mod modules;
mod runner;

use id_mngmnt::id_registrar::IdRegistrar;
use id_mngmnt::id_types::ModuleId;
use id_mngmnt::id_types::PortId;
use modules::container;
use modules::echo_module;
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
    echo_module::register(id_reg);
    container::register(id_reg);
}

fn setup_group(r: &mut runner::Runner, id_reg: &mut IdRegistrar) -> ModuleId {
    let sink = Box::new(sink::new_sink(id_reg, "Sink".to_owned()));
    let echo = Box::new(echo_module::new_echo_module(id_reg, "Echo".to_owned()));
    let group = Box::new(container::new_module_container(
        id_reg,
        "Container".to_owned(),
    ));

    let group_id = group.id;
    let sink_id = sink.id;
    let echo_id = echo.id;

    let tree = runner::Tree::Node(
        ("Group".to_owned(), group_id),
        vec![
            runner::Tree::Leaf(("Echo".to_owned(), echo_id)),
            runner::Tree::Leaf(("Sink".to_owned(), sink_id)),
        ],
    );

    r.add_to_tree(tree);

    r.add_module(sink).unwrap();
    r.add_module(echo).unwrap();
    r.add_module(group).unwrap();

    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0)),
        crate::connection::mesh::ConnectionKind::Onedirectional,
        group_id,
        container::INNER_GATE,
        PortId(0),
        sink_id,
        sink::IN_GATE,
        PortId(0),
    )
    .unwrap();

    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0)),
        crate::connection::mesh::ConnectionKind::Bidrectional,
        group_id,
        container::INNER_GATE,
        PortId(1),
        echo_id,
        echo_module::IN_GATE,
        PortId(0),
    )
    .unwrap();

    group_id
}

fn setup_modules(r: &mut runner::Runner, id_reg: &mut IdRegistrar) {
    register_needed_types(id_reg);

    let smod = Box::new(simple_module::new_simple_module(
        id_reg,
        "Source".to_owned(),
    ));
    let smod_id = smod.id;
    r.add_module(smod).unwrap();
    r.add_to_tree(runner::Tree::Leaf(("Source".to_owned(), smod_id)));

    let group_id = setup_group(r, id_reg);

    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 1, 0, 0)),
        crate::connection::mesh::ConnectionKind::Onedirectional,
        smod_id,
        simple_module::OUT_GATE,
        PortId(0),
        group_id,
        container::OUTER_GATE,
        PortId(0),
    )
    .unwrap();

    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 1, 0, 0)),
        crate::connection::mesh::ConnectionKind::Bidrectional,
        smod_id,
        simple_module::OUT_GATE,
        PortId(1),
        group_id,
        container::OUTER_GATE,
        PortId(1),
    )
    .unwrap();
}

fn main() {
    println!("Starting simulation");
    let seed: [u8; 16] = [
        40, 157, 153, 238, 231, 98, 7, 241, 206, 84, 162, 233, 247, 101, 104, 215,
    ];

    let mut r = runner::new_runner(seed);

    let mut id_reg = IdRegistrar {
        last_id: 0,
        last_type_id: 0,
        type_ids: std::collections::HashMap::new(),
        type_ids_reverse: std::collections::HashMap::new(),
    };

    setup_modules(&mut r, &mut id_reg);

    r.run(&mut id_reg, 2000).unwrap();
}
