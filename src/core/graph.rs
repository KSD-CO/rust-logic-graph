
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

impl Context {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) -> anyhow::Result<()> {
        self.data.insert(key.into(), value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
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
