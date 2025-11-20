use rust_logic_graph::{GraphDef, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// YAML configuration for graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub nodes: HashMap<String, NodeConfig>,
    pub edges: Vec<EdgeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub r#type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
}

impl GraphConfig {
    /// Load graph configuration from a YAML file
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read YAML file: {:?}", path.as_ref()))?;
        
        Self::from_yaml_str(&content)
    }
    
    /// Load graph configuration from a YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml)
            .with_context(|| "Failed to parse YAML configuration")
    }
    
    /// Convert to GraphDef for rust-logic-graph
    pub fn to_graph_def(&self) -> Result<GraphDef> {
        let mut nodes = HashMap::new();
        
        for (node_id, node_config) in &self.nodes {
            let node_type = match node_config.r#type.as_str() {
                "DBNode" => NodeType::DBNode,
                "RuleNode" => NodeType::RuleNode,
                "AINode" => NodeType::AINode,
                "APINode" => NodeType::APINode,
                _ => anyhow::bail!("Unknown node type: {}", node_config.r#type),
            };
            nodes.insert(node_id.clone(), node_type);
        }
        
        let edges = self.edges.iter().map(|e| {
            rust_logic_graph::core::Edge {
                from: e.from.clone(),
                to: e.to.clone(),
                rule: e.rule.clone(),
            }
        }).collect();
        
        Ok(GraphDef { nodes, edges })
    }
    
    /// Get the list of node IDs in topological order (useful for initialization)
    pub fn get_node_order(&self) -> Vec<String> {
        // Simple implementation: return nodes based on edges
        let mut visited = std::collections::HashSet::new();
        let mut order = Vec::new();
        
        // Find root nodes (no incoming edges)
        let has_incoming: std::collections::HashSet<_> = self.edges.iter()
            .map(|e| e.to.clone())
            .collect();
        
        for node_id in self.nodes.keys() {
            if !has_incoming.contains(node_id) {
                order.push(node_id.clone());
                visited.insert(node_id.clone());
            }
        }
        
        // Add remaining nodes
        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                order.push(node_id.clone());
            }
        }
        
        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_yaml_config() {
        let yaml = r#"
nodes:
  node1:
    type: DBNode
    description: "Test node"
  node2:
    type: RuleNode
    description: "Rule node"

edges:
  - from: node1
    to: node2
"#;
        
        let config = GraphConfig::from_yaml_str(yaml).unwrap();
        assert_eq!(config.nodes.len(), 2);
        assert_eq!(config.edges.len(), 1);
        
        let graph_def = config.to_graph_def().unwrap();
        assert_eq!(graph_def.nodes.len(), 2);
        assert_eq!(graph_def.edges.len(), 1);
    }
}
