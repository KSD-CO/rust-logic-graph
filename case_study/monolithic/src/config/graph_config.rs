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
    /// Database name for DBNode (optional, defaults to single DB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    /// Full connection string for DBNode (optional, overrides database field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
    /// Context keys to extract as query parameters for DBNode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>,
    /// Condition expression for RuleNode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// AI prompt for AINode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Input mappings: local_name -> context_key
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    /// Field mappings: field_name -> path (e.g., "avg_daily_demand" -> "oms_data.avg_daily_demand")
    #[serde(default)]
    pub field_mappings: HashMap<String, String>,
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
                _ => anyhow::bail!("Unknown node type: {}", node_config.r#type),
            };
            
            // Create proper NodeConfig with query/condition/prompt
            let mut config = match node_type {
                rust_logic_graph::NodeType::DBNode => {
                    let query = node_config.query.clone()
                        .unwrap_or_else(|| format!("SELECT * FROM {}", node_id));
                    
                    // Use db_node_with_params if params are specified
                    if let Some(params) = &node_config.params {
                        rust_logic_graph::NodeConfig::db_node_with_params(query, params.clone())
                    } else {
                        rust_logic_graph::NodeConfig::db_node(query)
                    }
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
                    let query = node_config.query.clone()
                        .unwrap_or_else(|| format!("http://localhost:50051#{}_method", node_id));
                    rust_logic_graph::NodeConfig::grpc_node(&query, &query)
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
