extern crate sim;

use sim::core::connection::mesh;
use sim::core::connection::simple_connection;
use sim::core::events::text_event;
use sim::core::id_mngmnt::id_registrar::IdRegistrar;
use sim::core::id_mngmnt::id_types::GateId;
use sim::core::id_mngmnt::id_types::ModuleId;
use sim::core::id_mngmnt::id_types::PortId;
use sim::core::messages::text_message;
use sim::core::modules::container;
use sim::core::modules::echo_module;
use sim::core::modules::simple_module;
use sim::core::modules::sink;
use sim::core::runner;
use sim::net::router;

fn register_needed_types(id_reg: &mut IdRegistrar) {
    simple_module::register(id_reg);
    sink::register(id_reg);
    text_event::register(id_reg);
    text_message::register(id_reg);
    simple_connection::register(id_reg);
    echo_module::register(id_reg);
    container::register(id_reg);
    router::router::register(id_reg);
}

fn setup_group(r: &mut runner::Runner, id_reg: &mut IdRegistrar) -> ModuleId {
    let sink = Box::new(sink::new_sink(id_reg, "Sink".to_owned()));
    let echo = Box::new(echo_module::new_echo_module(id_reg, "Echo".to_owned()));
    let group = Box::new(container::new_module_container(
        id_reg,
        "Container".to_owned(),
        vec![(GateId(1), GateId(0))],
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
        mesh::ConnectionKind::Onedirectional,
        group_id,
        GateId(0),
        PortId(0),
        sink_id,
        sink::IN_GATE,
        PortId(0),
    )
    .unwrap();

    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 0, 0, 0)),
        mesh::ConnectionKind::Bidrectional,
        group_id,
        GateId(0),
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

    let num_groups = 1000;
    let mut groups = Vec::new();

    for _ in 0..num_groups {
        groups.push(setup_group(r, id_reg));
    }

    let mut routing = std::collections::HashMap::new();

    //route from echo to echo as a chain until the last one will be dropped by the router
    for idx in 0..groups.len() * 2 {
        routing.insert(PortId(idx as u64), PortId((idx + 2) as u64));
    }
    let (router_id, tree) = router::router::make_router(
        r,
        id_reg,
        num_groups * 2 + 1,
        "CoolRouter".to_owned(),
        routing,
    );

    r.add_to_tree(tree);

    //simplemodule as source to the router
    r.connect_modules(
        Box::new(simple_connection::new_simple_connection(id_reg, 1, 10, 0)),
        mesh::ConnectionKind::Onedirectional,
        smod_id,
        simple_module::OUT_GATE,
        PortId(0),
        router_id,
        router::router::ROUTER_GATE_OUTER,
        PortId(0),
    )
    .unwrap();

    //connect all groups to the router
    let mut idx = 1;
    for group in groups {
        r.connect_modules(
            Box::new(simple_connection::new_simple_connection(id_reg, 1, 10, 0)),
            mesh::ConnectionKind::Onedirectional,
            router_id,
            router::router::ROUTER_GATE_OUTER,
            PortId(idx),
            group,
            GateId(1),
            PortId(0),
        )
        .unwrap();

        r.connect_modules(
            Box::new(simple_connection::new_simple_connection(id_reg, 1, 0, 0)),
            mesh::ConnectionKind::Bidrectional,
            router_id,
            router::router::ROUTER_GATE_OUTER,
            PortId(idx + 1),
            group,
            GateId(1),
            PortId(1),
        )
        .unwrap();

        idx += 2;
    }
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

    //use std::fs::File;
    //let mut f = File::create("graph.dot").unwrap();
    //r.print_as_dot(&mut f);

    r.run(&mut id_reg, 10000).unwrap();
}
