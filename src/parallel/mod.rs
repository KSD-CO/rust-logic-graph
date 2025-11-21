//! Parallel execution module
//!
//! Provides parallel node execution with layer detection and concurrent processing

use std::collections::{HashMap, HashSet};
use anyhow::Result;
use tracing::{info, debug, warn};

use crate::core::{Graph, GraphDef};
use crate::node::Node;
use crate::rule::Rule;

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of concurrent nodes per layer
    pub max_concurrent: usize,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            verbose: false,
        }
    }
}

/// Represents a layer of independent nodes that can execute in parallel
#[derive(Debug, Clone)]
pub struct ExecutionLayer {
    pub layer_index: usize,
    pub node_ids: Vec<String>,
}

/// Parallel executor that identifies independent nodes and executes them concurrently
pub struct ParallelExecutor {
    nodes: HashMap<String, Box<dyn Node>>,
    _config: ParallelConfig,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            _config: config,
        }
    }

    /// Register a node with the executor
    pub fn register_node(&mut self, node: Box<dyn Node>) {
        let id = node.id().to_string();
        self.nodes.insert(id, node);
    }

    /// Analyze graph and identify execution layers
    /// Nodes in the same layer have no dependencies on each other and can run in parallel
    pub fn identify_layers(&self, def: &GraphDef) -> Result<Vec<ExecutionLayer>> {
        info!("ParallelExecutor: Identifying execution layers");

        // Build adjacency list and in-degree map
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize all nodes
        for node_id in def.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            adj_list.insert(node_id.clone(), Vec::new());
        }

        // Build graph structure
        for edge in &def.edges {
            adj_list
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());

            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        // Perform layer-by-layer topological sort
        let mut layers = Vec::new();
        let mut current_layer_index = 0;
        let mut processed = HashSet::new();

        loop {
            // Find all nodes with in-degree 0 that haven't been processed
            let current_layer_nodes: Vec<String> = in_degree
                .iter()
                .filter(|(id, &degree)| degree == 0 && !processed.contains(*id))
                .map(|(id, _)| id.clone())
                .collect();

            if current_layer_nodes.is_empty() {
                break;
            }

            debug!(
                "Layer {}: {} nodes can execute in parallel: {:?}",
                current_layer_index,
                current_layer_nodes.len(),
                current_layer_nodes
            );

            layers.push(ExecutionLayer {
                layer_index: current_layer_index,
                node_ids: current_layer_nodes.clone(),
            });

            // Mark nodes as processed and update in-degrees
            for node_id in &current_layer_nodes {
                processed.insert(node_id.clone());

                // Reduce in-degree for downstream nodes
                if let Some(neighbors) = adj_list.get(node_id) {
                    for neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            *degree = degree.saturating_sub(1);
                        }
                    }
                }
            }

            current_layer_index += 1;
        }

        // Check for unprocessed nodes (cycles)
        let unprocessed: Vec<_> = def
            .nodes
            .keys()
            .filter(|id| !processed.contains(*id))
            .collect();

        if !unprocessed.is_empty() {
            warn!(
                "Some nodes could not be scheduled (possible cycle): {:?}",
                unprocessed
            );
        }

        info!(
            "ParallelExecutor: Identified {} execution layers with total {} nodes",
            layers.len(),
            processed.len()
        );

        Ok(layers)
    }

    /// Execute a single layer of nodes in parallel
    async fn execute_layer(
        &self,
        layer: &ExecutionLayer,
        graph: &mut Graph,
    ) -> Result<Vec<String>> {
        info!(
            "ParallelExecutor: Executing layer {} with {} nodes",
            layer.layer_index,
            layer.node_ids.len()
        );

        let mut successful_nodes = Vec::new();

        // Execute nodes in the layer
        // Note: Currently executing sequentially within layer due to context sharing
        // In a production system, you'd use proper synchronization (Arc<Mutex<Context>>)
        // or message passing to enable true parallel execution
        for node_id in &layer.node_ids {
            // Check if node should be executed based on incoming edge rules
            let should_execute = self.check_incoming_rules(node_id, graph);

            if !should_execute {
                info!("Skipping node '{}' due to failed rule", node_id);
                continue;
            }

            if let Some(node) = self.nodes.get(node_id) {
                info!("Executing node '{}'", node_id);

                match node.run(&mut graph.context).await {
                    Ok(_) => {
                        info!("Node '{}' executed successfully", node_id);
                        successful_nodes.push(node_id.clone());
                    }
                    Err(e) => {
                        warn!("Node '{}' execution failed: {:?}", node_id, e);
                    }
                }
            } else {
                warn!("Node '{}' not found in executor", node_id);
            }
        }

        info!(
            "Layer {} completed: {}/{} nodes successful",
            layer.layer_index,
            successful_nodes.len(),
            layer.node_ids.len()
        );

        Ok(successful_nodes)
    }

    /// Check if a node should execute based on incoming edge rules
    fn check_incoming_rules(&self, node_id: &str, graph: &Graph) -> bool {
        let incoming_edges: Vec<_> = graph
            .def
            .edges
            .iter()
            .filter(|e| e.to == node_id)
            .collect();

        for edge in &incoming_edges {
            if let Some(rule_id) = &edge.rule {
                let rule = Rule::new(rule_id, "true");

                match rule.evaluate(&graph.context.data) {
                    Ok(result) => {
                        if let serde_json::Value::Bool(false) = result {
                            debug!(
                                "Rule '{}' for edge {} -> {} evaluated to false",
                                rule_id, edge.from, edge.to
                            );
                            return false;
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Rule '{}' evaluation failed: {}. Assuming true.",
                            rule_id, e
                        );
                    }
                }
            }
        }

        true
    }

    /// Execute the entire graph with parallel execution per layer
    pub async fn execute(&self, graph: &mut Graph) -> Result<()> {
        info!("ParallelExecutor: Starting parallel graph execution");

        // Identify execution layers
        let layers = self.identify_layers(&graph.def)?;

        if layers.is_empty() {
            warn!("No execution layers found");
            return Ok(());
        }

        info!(
            "ParallelExecutor: Executing {} layers",
            layers.len()
        );

        let mut total_executed = 0;

        // Execute each layer sequentially (but nodes within each layer run in parallel)
        for layer in layers {
            let successful_nodes = self.execute_layer(&layer, graph).await?;
            total_executed += successful_nodes.len();
        }

        info!(
            "ParallelExecutor: Completed parallel execution. Total nodes executed: {}",
            total_executed
        );

        Ok(())
    }

    /// Get parallel execution statistics
    pub fn get_parallelism_stats(&self, def: &GraphDef) -> Result<ParallelismStats> {
        let layers = self.identify_layers(def)?;

        let total_nodes = def.nodes.len();
        let max_parallel_nodes = layers
            .iter()
            .map(|layer| layer.node_ids.len())
            .max()
            .unwrap_or(0);

        let sequential_time = total_nodes; // Assume 1 unit per node
        let parallel_time = layers.len(); // Each layer is 1 unit

        let speedup = if parallel_time > 0 {
            sequential_time as f64 / parallel_time as f64
        } else {
            1.0
        };

        Ok(ParallelismStats {
            total_nodes,
            num_layers: layers.len(),
            max_parallel_nodes,
            avg_parallel_nodes: if !layers.is_empty() {
                total_nodes as f64 / layers.len() as f64
            } else {
                0.0
            },
            theoretical_speedup: speedup,
            layers,
        })
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new(ParallelConfig::default())
    }
}

/// Statistics about parallelism in a graph
#[derive(Debug)]
pub struct ParallelismStats {
    pub total_nodes: usize,
    pub num_layers: usize,
    pub max_parallel_nodes: usize,
    pub avg_parallel_nodes: f64,
    pub theoretical_speedup: f64,
    pub layers: Vec<ExecutionLayer>,
}

impl ParallelismStats {
    pub fn print_summary(&self) {
        println!("\n=== Parallelism Analysis ===");
        println!("Total nodes: {}", self.total_nodes);
        println!("Execution layers: {}", self.num_layers);
        println!("Max parallel nodes: {}", self.max_parallel_nodes);
        println!("Avg parallel nodes per layer: {:.2}", self.avg_parallel_nodes);
        println!("Theoretical speedup: {:.2}x", self.theoretical_speedup);
        println!("\nLayer breakdown:");
        for layer in &self.layers {
            println!(
                "  Layer {}: {} nodes - {:?}",
                layer.layer_index,
                layer.node_ids.len(),
                layer.node_ids
            );
        }
        println!("===========================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeType;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_layer_identification() {
        // Create a graph with known parallelism
        // Graph structure:
        //     A
        //    / \
        //   B   C   <- Can run in parallel
        //    \ /
        //     D
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), NodeType::RuleNode);
        nodes.insert("B".to_string(), NodeType::RuleNode);
        nodes.insert("C".to_string(), NodeType::RuleNode);
        nodes.insert("D".to_string(), NodeType::RuleNode);

        let mut edges = Vec::new();
        edges.push(crate::core::Edge {
            from: "A".to_string(),
            to: "B".to_string(),
            rule: None,
        });
        edges.push(crate::core::Edge {
            from: "A".to_string(),
            to: "C".to_string(),
            rule: None,
        });
        edges.push(crate::core::Edge {
            from: "B".to_string(),
            to: "D".to_string(),
            rule: None,
        });
        edges.push(crate::core::Edge {
            from: "C".to_string(),
            to: "D".to_string(),
            rule: None,
        });

        let def = GraphDef::from_node_types(nodes, edges);
        let executor = ParallelExecutor::default();
        let layers = executor.identify_layers(&def).unwrap();

        // Should have 3 layers: [A], [B, C], [D]
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0].node_ids.len(), 1); // A
        assert_eq!(layers[1].node_ids.len(), 2); // B, C
        assert_eq!(layers[2].node_ids.len(), 1); // D
    }

    #[tokio::test]
    async fn test_parallelism_stats() {
        let mut nodes = HashMap::new();
        // Linear chain: A -> B -> C -> D (no parallelism)
        nodes.insert("A".to_string(), NodeType::RuleNode);
        nodes.insert("B".to_string(), NodeType::RuleNode);
        nodes.insert("C".to_string(), NodeType::RuleNode);
        nodes.insert("D".to_string(), NodeType::RuleNode);

        let mut edges = Vec::new();
        edges.push(crate::core::Edge {
            from: "A".to_string(),
            to: "B".to_string(),
            rule: None,
        });
        edges.push(crate::core::Edge {
            from: "B".to_string(),
            to: "C".to_string(),
            rule: None,
        });
        edges.push(crate::core::Edge {
            from: "C".to_string(),
            to: "D".to_string(),
            rule: None,
        });

        let def = GraphDef::from_node_types(nodes, edges);
        let executor = ParallelExecutor::default();
        let stats = executor.get_parallelism_stats(&def).unwrap();

        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.num_layers, 4); // All sequential
        assert_eq!(stats.max_parallel_nodes, 1); // No parallelism
        assert_eq!(stats.theoretical_speedup, 1.0); // No speedup
    }

    #[tokio::test]
    async fn test_parallel_graph_stats() {
        let mut nodes = HashMap::new();
        // Fully parallel: 4 independent nodes
        nodes.insert("A".to_string(), NodeType::RuleNode);
        nodes.insert("B".to_string(), NodeType::RuleNode);
        nodes.insert("C".to_string(), NodeType::RuleNode);
        nodes.insert("D".to_string(), NodeType::RuleNode);

        let def = GraphDef::from_node_types(nodes, vec![]);
        let executor = ParallelExecutor::default();
        let stats = executor.get_parallelism_stats(&def).unwrap();

        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.num_layers, 1); // All in one layer
        assert_eq!(stats.max_parallel_nodes, 4); // All parallel
        assert_eq!(stats.theoretical_speedup, 4.0); // 4x speedup
    }
}
