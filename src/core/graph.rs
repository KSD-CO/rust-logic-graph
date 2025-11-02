
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::node::NodeType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub rule: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphDef {
    pub nodes: HashMap<String, NodeType>,
    pub edges: Vec<Edge>,
}

#[derive(Default)]
pub struct Context {
    pub data: HashMap<String, serde_json::Value>,
}

pub struct Graph {
    pub def: GraphDef,
    pub context: Context,
}

impl Graph {
    pub fn new(def: GraphDef) -> Self {
        Self {
            def,
            context: Context::default(),
        }
    }
}
