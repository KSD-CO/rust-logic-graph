use rust_logic_graph::GraphDef;
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
    /// SQL query for DBNode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Condition expression for RuleNode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// AI prompt for AINode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
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
                "DBNode" => rust_logic_graph::NodeType::DBNode,
                "RuleNode" => rust_logic_graph::NodeType::RuleNode,
                "AINode" => rust_logic_graph::NodeType::AINode,
                "GrpcNode" => rust_logic_graph::NodeType::GrpcNode,
                "APINode" => rust_logic_graph::NodeType::AINode, // Map APINode to AINode for backward compat
                _ => anyhow::bail!("Unknown node type: {}", node_config.r#type),
            };
            
            // Create proper NodeConfig with query/condition/prompt
            let config = match node_type {
                rust_logic_graph::NodeType::DBNode => {
                    rust_logic_graph::NodeConfig::db_node(
                        node_config.query.clone()
                            .unwrap_or_else(|| format!("SELECT * FROM {}", node_id))
                    )
                }
                rust_logic_graph::NodeType::RuleNode => {
                    rust_logic_graph::NodeConfig::rule_node(
                        node_config.condition.clone().unwrap_or_else(|| "true".to_string())
                    )
                }
                rust_logic_graph::NodeType::AINode => {
                    rust_logic_graph::NodeConfig::ai_node(
                        node_config.prompt.clone()
                            .unwrap_or_else(|| format!("Process data for {}", node_id))
                    )
                }
                rust_logic_graph::NodeType::GrpcNode => {
                    // Parse query field as "service_url#method"
                    let query = node_config.query.as_ref()
                        .ok_or_else(|| anyhow::anyhow!("GrpcNode '{}' missing 'query' field with format 'service_url#method'", node_id))?;
                    
                    // Split into service_url and method
                    let parts: Vec<&str> = query.split('#').collect();
                    if parts.len() != 2 {
                        anyhow::bail!("GrpcNode '{}' query must be in format 'service_url#method', got: {}", node_id, query);
                    }
                    
                    let service_url = parts[0];
                    let method = parts[1];
                    
                    rust_logic_graph::NodeConfig::grpc_node(service_url, method)
                }
            };
            
            nodes.insert(node_id.clone(), config);
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
        // For more complex graphs, you'd want proper topological sort
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

