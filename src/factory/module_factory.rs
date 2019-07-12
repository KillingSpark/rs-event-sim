#![allow(dead_code)]

use std::collections::HashMap;
use crate::id_mngmnt::id_types::{GateId, ModuleTypeId};
use crate::modules::module::Module;
use crate::id_mngmnt::id_registrar::IdRegistrar;

use crate::modules::container;


pub fn container_from_params(id_reg: &mut IdRegistrar,  parameters: &HashMap<&str, &str>) -> GeneratorResult {
    let name = (*parameters.get(&"name").unwrap()).to_owned();
    let gates = parameters.get(&"gates").unwrap().split(",").map(|gate_tuple| {
        let gates: Vec<&str> = gate_tuple.split(">").collect();
        let outer: u64 = gates[0].parse().unwrap();
        let inner: u64 = gates[1].parse().unwrap();
        (GateId(outer), GateId(inner))
    }).collect();

    Ok(Box::new(container::new_module_container(id_reg, name, gates)))
}

type GeneratorFunction = fn (id_reg: &mut IdRegistrar, parameters: &HashMap<String, String>) -> GeneratorResult; 
type GeneratorResult = Result<Box<Module>, Box<std::error::Error>>;

pub struct ModuleFactory {
    generators: HashMap<ModuleTypeId, GeneratorFunction>,
}

pub fn new() -> ModuleFactory {
    ModuleFactory {
        generators: HashMap::new(),
    }
}

impl ModuleFactory {
    pub fn generate(&self, id_reg: &mut IdRegistrar, id: ModuleTypeId, parameters: &HashMap<String, String>) -> GeneratorResult {
        let gen = self.generators.get(&id).unwrap();
        gen(id_reg, parameters)
    }
}
