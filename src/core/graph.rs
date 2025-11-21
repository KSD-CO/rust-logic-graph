
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::node::NodeType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub rule: Option<String>,
}

/// Configuration for a node in the graph
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    pub node_type: NodeType,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
}

impl NodeConfig {
    pub fn rule_node(condition: impl Into<String>) -> Self {
        Self {
            node_type: NodeType::RuleNode,
            condition: Some(condition.into()),
            query: None,
            prompt: None,
        }
    }

    pub fn db_node(query: impl Into<String>) -> Self {
        Self {
            node_type: NodeType::DBNode,
            condition: None,
            query: Some(query.into()),
            prompt: None,
        }
    }

    pub fn ai_node(prompt: impl Into<String>) -> Self {
        Self {
            node_type: NodeType::AINode,
            condition: None,
            query: None,
            prompt: Some(prompt.into()),
        }
    }
    
    /// Create a GrpcNode configuration
    pub fn grpc_node(service_url: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            node_type: NodeType::GrpcNode,
            query: Some(format!("{}#{}", service_url.into(), method.into())),
            condition: None,
            prompt: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphDef {
    pub nodes: HashMap<String, NodeConfig>,
    pub edges: Vec<Edge>,
}

impl GraphDef {
    /// Create a GraphDef from simple node types (backward compatibility helper)
    pub fn from_node_types(
        nodes: HashMap<String, NodeType>,
        edges: Vec<Edge>,
    ) -> Self {
        let nodes = nodes
            .into_iter()
            .map(|(id, node_type)| {
                let config = match node_type {
                    NodeType::RuleNode => NodeConfig::rule_node("true"),
                    NodeType::DBNode => NodeConfig::db_node(format!("SELECT * FROM {}", id)),
                    NodeType::AINode => NodeConfig::ai_node(format!("Process data for {}", id)),
                    NodeType::GrpcNode => NodeConfig::grpc_node(
                        format!("http://localhost:50051"),
                        format!("{}_method", id)
                    ),
                };
                (id, config)
            })
            .collect();
        
        Self { nodes, edges }
    }
    
    /// Validate graph structure
    pub fn validate(&self) -> anyhow::Result<()> {
        // Check for empty graph
        if self.nodes.is_empty() {
            return Err(anyhow::anyhow!("Graph has no nodes"));
        }
        
        // Check for invalid edge references
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) {
                return Err(anyhow::anyhow!(
                    "Edge references non-existent source node: '{}'",
                    edge.from
                ));
            }
            if !self.nodes.contains_key(&edge.to) {
                return Err(anyhow::anyhow!(
                    "Edge references non-existent target node: '{}'",
                    edge.to
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if graph has disconnected components
    pub fn has_disconnected_components(&self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        
        use std::collections::HashSet;
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        // Start from first node
        if let Some(first_node) = self.nodes.keys().next() {
            stack.push(first_node.clone());
        }
        
        // DFS traversal (undirected)
        while let Some(node) = stack.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            
            // Add neighbors (both directions)
            for edge in &self.edges {
                if edge.from == node && !visited.contains(&edge.to) {
                    stack.push(edge.to.clone());
                }
                if edge.to == node && !visited.contains(&edge.from) {
                    stack.push(edge.from.clone());
                }
            }
        }
        
        visited.len() < self.nodes.len()
    }
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

    /// Set a value in the context
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.data.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
    
    /// Check if key exists in context
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    /// Remove a value from context
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }
    
    /// Clear all context data
    pub fn clear(&mut self) {
        self.data.clear();
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
