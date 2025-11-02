use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::Result;
use tracing::{info, debug, warn};

use crate::core::{Graph, GraphDef};
use crate::node::{Node, RuleNode as ConcreteRuleNode, DBNode, AINode};
use crate::rule::Rule;

pub struct Executor {
    nodes: HashMap<String, Box<dyn Node>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Build executor from graph definition
    pub fn from_graph_def(def: &GraphDef) -> Result<Self> {
        let mut executor = Self::new();

        // Create concrete node instances based on NodeType
        for (node_id, node_type) in &def.nodes {
            let node: Box<dyn Node> = match node_type {
                crate::node::NodeType::RuleNode => {
                    Box::new(ConcreteRuleNode::new(node_id, "true"))
                }
                crate::node::NodeType::DBNode => {
                    Box::new(DBNode::new(node_id, format!("SELECT * FROM {}", node_id)))
                }
                crate::node::NodeType::AINode => {
                    Box::new(AINode::new(node_id, format!("Process data for {}", node_id)))
                }
            };

            executor.register_node(node);
        }

        Ok(executor)
    }

    /// Register a node with the executor
    pub fn register_node(&mut self, node: Box<dyn Node>) {
        let id = node.id().to_string();
        self.nodes.insert(id, node);
    }

    /// Execute the graph in topological order
    pub async fn execute(&self, graph: &mut Graph) -> Result<()> {
        info!("Executor: Starting graph execution");

        // Build adjacency list and in-degree map
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize all nodes with 0 in-degree
        for node_id in graph.def.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            adj_list.insert(node_id.clone(), Vec::new());
        }

        // Build the graph structure
        for edge in &graph.def.edges {
            adj_list
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());

            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        // Find all nodes with in-degree 0 (starting nodes)
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        if queue.is_empty() {
            warn!("No starting nodes found (all nodes have incoming edges). Starting with first node.");
            if let Some(first_node) = graph.def.nodes.keys().next() {
                queue.push_back(first_node.clone());
            }
        }

        let mut executed = HashSet::new();
        let mut execution_order = Vec::new();

        // Topological sort & execution
        while let Some(node_id) = queue.pop_front() {
            if executed.contains(&node_id) {
                continue;
            }

            info!("Executor: Processing node '{}'", node_id);

            // Check if all incoming edges have their rules satisfied
            let incoming_edges: Vec<_> = graph
                .def
                .edges
                .iter()
                .filter(|e| e.to == node_id)
                .collect();

            let mut should_execute = true;

            for edge in &incoming_edges {
                if let Some(rule_id) = &edge.rule {
                    let rule = Rule::new(rule_id, "true"); // Default condition

                    match rule.evaluate(&graph.context.data) {
                        Ok(result) => {
                            debug!(
                                "Rule '{}' for edge {} -> {} evaluated to: {:?}",
                                rule_id, edge.from, edge.to, result
                            );

                            if let serde_json::Value::Bool(false) = result {
                                should_execute = false;
                                info!(
                                    "Skipping node '{}' due to failed rule '{}'",
                                    node_id, rule_id
                                );
                                break;
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

            // Execute the node
            if should_execute {
                if let Some(node) = self.nodes.get(&node_id) {
                    match node.run(&mut graph.context).await {
                        Ok(_) => {
                            info!("Node '{}' executed successfully", node_id);
                            execution_order.push(node_id.clone());
                        }
                        Err(e) => {
                            warn!("Node '{}' execution failed: {:?}", node_id, e);
                        }
                    }
                } else {
                    warn!("Node '{}' not found in executor", node_id);
                }
            }

            executed.insert(node_id.clone());

            // Add downstream nodes to queue
            if let Some(neighbors) = adj_list.get(&node_id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 && !executed.contains(neighbor) {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        info!(
            "Executor: Completed execution. Executed nodes: {:?}",
            execution_order
        );

        // Check for unexecuted nodes (possible cycle)
        let unexecuted: Vec<_> = graph
            .def
            .nodes
            .keys()
            .filter(|id| !executed.contains(*id))
            .collect();

        if !unexecuted.is_empty() {
            warn!("Some nodes were not executed (possible cycle): {:?}", unexecuted);
        }

        Ok(())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
