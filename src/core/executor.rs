use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::Result;
use tracing::{info, debug, warn};
use std::time::{Duration, Instant};

use crate::core::{Graph, GraphDef};
use crate::node::{Node, RuleNode as ConcreteRuleNode, DBNode, AINode};
use crate::rule::Rule;
use crate::cache::{CacheManager, CacheKey};

/// Execution statistics for a single node
#[derive(Debug, Clone)]
pub struct NodeExecutionStats {
    pub node_id: String,
    pub duration: Duration,
    pub cache_hit: bool,
    pub success: bool,
}

/// Overall execution metrics
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    pub total_duration: Duration,
    pub nodes_executed: usize,
    pub nodes_skipped: usize,
    pub nodes_failed: usize,
    pub cache_hits: usize,
    pub node_stats: Vec<NodeExecutionStats>,
}

/// Executor for running graph nodes in topological order.
/// 
/// # Thread Safety
/// 
/// The Executor is **NOT thread-safe** for concurrent executions on the same instance.
/// While `execute()` is async and takes `&self` (shared reference), the underlying
/// implementation assumes single-threaded access patterns:
/// 
/// - `self.nodes` is a regular `HashMap` without synchronization
/// - Multiple concurrent calls to `execute()` would have data races when accessing nodes
/// 
/// ## Safe Usage Patterns
/// 
/// 1. **Single execution at a time**: Only call `execute()` once at a time per executor instance
/// 2. **Clone for parallelism**: Create separate executor instances for parallel graph executions
/// 3. **Sequential async**: Use `.await` to ensure executions don't overlap
/// 
/// ## Future Work
/// 
/// For true concurrent execution support, wrap `nodes` in `Arc<RwLock<HashMap>>` or similar.
pub struct Executor {
    nodes: HashMap<String, Box<dyn Node>>,
    cache: Option<CacheManager>,
    metrics: ExecutionMetrics,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            cache: None,
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Create a new executor with caching enabled
    pub fn with_cache(cache: CacheManager) -> Self {
        Self {
            nodes: HashMap::new(),
            cache: Some(cache),
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Enable caching for this executor
    pub fn set_cache(&mut self, cache: CacheManager) {
        self.cache = Some(cache);
    }

    /// Get the cache manager (if enabled)
    pub fn cache(&self) -> Option<&CacheManager> {
        self.cache.as_ref()
    }
    
    /// Get execution metrics from last run
    pub fn metrics(&self) -> &ExecutionMetrics {
        &self.metrics
    }
    
    /// Reset execution metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ExecutionMetrics::default();
    }

    /// Build executor from graph definition
    pub fn from_graph_def(def: &GraphDef) -> Result<Self> {
        let mut executor = Self::new();

        // Create concrete node instances based on NodeConfig
        for (node_id, config) in &def.nodes {
            let node: Box<dyn Node> = match config.node_type {
                crate::node::NodeType::RuleNode => {
                    let condition = config.condition.as_deref().unwrap_or("true");
                    Box::new(ConcreteRuleNode::new(node_id, condition))
                }
                crate::node::NodeType::DBNode => {
                    let query = config.query.clone()
                        .unwrap_or_else(|| format!("SELECT * FROM {}", node_id));
                    Box::new(DBNode::new(node_id, query))
                }
                crate::node::NodeType::AINode => {
                    let prompt = config.prompt.clone()
                        .unwrap_or_else(|| format!("Process data for {}", node_id));
                    Box::new(AINode::new(node_id, prompt))
                }
                crate::node::NodeType::GrpcNode => {
                    // Parse query field as "service_url#method"
                    let query = config.query.clone()
                        .unwrap_or_else(|| format!("http://localhost:50051#{}_method", node_id));
                    let parts: Vec<&str> = query.split('#').collect();
                    let service_url = parts.get(0).unwrap_or(&"http://localhost:50051").to_string();
                    let method = parts.get(1).unwrap_or(&"UnknownMethod").to_string();
                    Box::new(crate::node::GrpcNode::new(node_id, service_url, method))
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

    /// Detect cycles in the graph using DFS
    fn detect_cycles(&self, graph: &Graph) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        // Build adjacency list for cycle detection
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &graph.def.edges {
            adj_list
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());
        }
        
        // DFS to detect cycles
        fn dfs_cycle_check(
            node: &str,
            adj_list: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());
            
            if let Some(neighbors) = adj_list.get(node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        if let Some(cycle) = dfs_cycle_check(neighbor, adj_list, visited, rec_stack, path) {
                            return Some(cycle);
                        }
                    } else if rec_stack.contains(neighbor) {
                        // Found a cycle - return the cycle path
                        let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                        return Some(path[cycle_start..].to_vec());
                    }
                }
            }
            
            path.pop();
            rec_stack.remove(node);
            None
        }
        
        // Check all nodes
        for node_id in graph.def.nodes.keys() {
            if !visited.contains(node_id) {
                let mut path = Vec::new();
                if let Some(cycle) = dfs_cycle_check(node_id, &adj_list, &mut visited, &mut rec_stack, &mut path) {
                    return Err(anyhow::anyhow!(
                        "Cycle detected in graph: {} -> {}",
                        cycle.join(" -> "),
                        cycle.first().unwrap()
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Execute the graph in topological order
    pub async fn execute(&mut self, graph: &mut Graph) -> Result<()> {
        info!("Executor: Starting graph execution");
        let execution_start = Instant::now();
        
        // Reset metrics
        self.metrics = ExecutionMetrics::default();

        // Validate graph structure first
        graph.def.validate()?;
        
        // Warn about disconnected components
        if graph.def.has_disconnected_components() {
            warn!("Graph has disconnected components - some nodes may not be reachable");
        }

        // First, detect cycles in the graph
        self.detect_cycles(graph)?;

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
            return Err(anyhow::anyhow!(
                "No starting nodes found in graph (all nodes have incoming edges). \
                This indicates either a cycle (which should have been caught earlier) \
                or an invalid graph structure. Nodes: {:?}",
                graph.def.nodes.keys().collect::<Vec<_>>()
            ));
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
            // IMPORTANT: Only check edges from nodes that have been executed
            let incoming_edges: Vec<_> = graph
                .def
                .edges
                .iter()
                .filter(|e| e.to == node_id && executed.contains(&e.from))
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
                                    "Skipping node '{}' due to failed rule '{}' from executed node '{}'",
                                    node_id, rule_id, edge.from
                                );
                                self.metrics.nodes_skipped += 1;
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
                    let node_start = Instant::now();
                    let mut cache_hit = false;
                    
                    // Create cache key based on node ID and relevant context only
                    // Only include context keys that this node might depend on
                    let relevant_context: HashMap<String, serde_json::Value> = incoming_edges
                        .iter()
                        .filter_map(|edge| {
                            graph.context.data.get(&format!("{}_result", edge.from))
                                .map(|v| (edge.from.clone(), v.clone()))
                        })
                        .collect();
                    
                    let context_value = serde_json::to_value(&relevant_context)?;
                    let cache_key = CacheKey::new(&node_id, &context_value);

                    // Check cache first
                    let cached_result = if let Some(cache) = &self.cache {
                        cache.get(&cache_key)
                    } else {
                        None
                    };

                    let result = if let Some(cached_value) = cached_result {
                        info!("Node '{}' result retrieved from cache", node_id);
                        cache_hit = true;
                        self.metrics.cache_hits += 1;
                        
                        // Merge cached result into context
                        if let serde_json::Value::Object(cached_obj) = cached_value {
                            for (k, v) in cached_obj {
                                graph.context.data.insert(k, v);
                            }
                        }
                        
                        Ok(serde_json::Value::Null) // Successfully used cache
                    } else {
                        // Execute node and cache result
                        let exec_result = node.run(&mut graph.context).await;
                        
                        // Store result in cache if execution succeeded
                        if exec_result.is_ok() {
                            if let Some(cache) = &self.cache {
                                let context_result = serde_json::to_value(&graph.context.data)?;
                                if let Err(e) = cache.put(cache_key, context_result, None) {
                                    warn!("Failed to cache result for node '{}': {}", node_id, e);
                                }
                            }
                        }
                        
                        exec_result
                    };

                    match result {
                        Ok(_) => {
                            let duration = node_start.elapsed();
                            info!("Node '{}' executed successfully in {:?}", node_id, duration);
                            execution_order.push(node_id.clone());
                            
                            self.metrics.nodes_executed += 1;
                            self.metrics.node_stats.push(NodeExecutionStats {
                                node_id: node_id.clone(),
                                duration,
                                cache_hit,
                                success: true,
                            });
                        }
                        Err(e) => {
                            let duration = node_start.elapsed();
                            warn!("Node '{}' execution failed: {:?}", node_id, e);
                            
                            self.metrics.nodes_failed += 1;
                            self.metrics.node_stats.push(NodeExecutionStats {
                                node_id: node_id.clone(),
                                duration,
                                cache_hit,
                                success: false,
                            });
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

        self.metrics.total_duration = execution_start.elapsed();
        
        info!(
            "Executor: Completed execution in {:?}. Executed: {}, Skipped: {}, Failed: {}, Cache hits: {}",
            self.metrics.total_duration,
            self.metrics.nodes_executed,
            self.metrics.nodes_skipped,
            self.metrics.nodes_failed,
            self.metrics.cache_hits
        );

        // Verify all nodes were executed (should not happen with cycle detection)
        let unexecuted: Vec<_> = graph
            .def
            .nodes
            .keys()
            .filter(|id| !executed.contains(*id))
            .collect();

        if !unexecuted.is_empty() {
            return Err(anyhow::anyhow!(
                "Some nodes were not executed: {:?}. This indicates a bug in the executor logic.",
                unexecuted
            ));
        }

        Ok(())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
