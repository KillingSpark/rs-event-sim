#![allow(dead_code)]

use std::collections::HashMap;

pub struct Parser {

}

pub struct ModuleDesc {
    params: HashMap<String, String>,
    gates: Vec<String>,
}

type GatePort = (String, u64);

pub struct ContainerDesc {
    gates: Vec<String>,
    connections_down: Vec<(GatePort, (String,GatePort))>,
    connections_up: Vec<((String,GatePort), GatePort)>,
    connections_bidir: Vec<(GatePort, (String,GatePort))>,
    sub_modules: Vec<(String, String)>,
}

impl Parser {
    pub fn parse_module(&mut self) -> Result<HashMap<String, String>, Box<std::error::Error>>{
        //for now always returns stuff for a simplemodule
        let mut params = HashMap::new();
        params.insert("name".to_owned(), "BoomBox123".to_owned());
        
        Ok(params)
    }
}