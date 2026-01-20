use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

use crate::core::Context;
use crate::rule::RuleResult;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum NodeType {
    #[default]
    RuleNode,
    DBNode,
    AINode,
    GrpcNode,
    SubgraphNode,
    ConditionalNode,
    LoopNode,
    TryCatchNode,
    RetryNode,
    CircuitBreakerNode,
}

#[async_trait]
pub trait Node: Send + Sync {
    fn id(&self) -> &str;
    fn node_type(&self) -> NodeType;
    async fn run(&self, ctx: &mut Context) -> RuleResult;
}

// ============================================================
// RuleNode - Evaluates conditions and transforms data
// ============================================================

#[derive(Debug, Clone)]
pub struct RuleNode {
    pub id: String,
    pub condition: String,
}

impl RuleNode {
    pub fn new(id: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            condition: condition.into(),
        }
    }
}

#[async_trait]
impl Node for RuleNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!(
            "RuleNode[{}]: Evaluating condition '{}'",
            self.id, self.condition
        );

        // Simple condition evaluation (can be extended with proper parser)
        let result = if self.condition == "true" {
            Value::Bool(true)
        } else if self.condition == "false" {
            Value::Bool(false)
        } else {
            // Try to evaluate based on context
            ctx.data
                .get(&self.condition)
                .cloned()
                .unwrap_or(Value::Bool(true))
        };

        debug!("RuleNode[{}]: Result = {:?}", self.id, result);
        ctx.data
            .insert(format!("{}_result", self.id), result.clone());

        Ok(result)
    }
}

// ============================================================
// DBNode - Database operations with pluggable executor
// ============================================================

/// Trait for database executors - implement this for MySQL, Postgres, etc.
#[async_trait]
pub trait DatabaseExecutor: Send + Sync {
    /// Execute a query and return JSON result
    async fn execute(&self, query: &str, params: &[&str]) -> Result<Value, String>;
}

/// Mock database executor (default for examples/testing)
#[derive(Debug, Clone)]
pub struct MockDatabaseExecutor;

#[async_trait]
impl DatabaseExecutor for MockDatabaseExecutor {
    async fn execute(&self, query: &str, _params: &[&str]) -> Result<Value, String> {
        // Simulate async DB operation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(serde_json::json!({
            "query": query,
            "rows": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false}
            ],
            "count": 2
        }))
    }
}

#[derive(Clone)]
pub struct DBNode {
    pub id: String,
    pub query: String,
    executor: Option<Arc<dyn DatabaseExecutor>>,
    /// Context keys to extract as query parameters
    param_keys: Option<Vec<String>>,
}

impl DBNode {
    pub fn new(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            executor: None,
            param_keys: None,
        }
    }

    /// Create DBNode with parameter keys to extract from context
    pub fn with_params(
        id: impl Into<String>,
        query: impl Into<String>,
        param_keys: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            executor: None,
            param_keys: Some(param_keys),
        }
    }

    /// Create DBNode with custom executor (MySQL, Postgres, etc.)
    pub fn with_executor(
        id: impl Into<String>,
        query: impl Into<String>,
        executor: Arc<dyn DatabaseExecutor>,
    ) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            executor: Some(executor),
            param_keys: None,
        }
    }

    /// Create DBNode with custom executor and parameter keys
    pub fn with_executor_and_params(
        id: impl Into<String>,
        query: impl Into<String>,
        executor: Arc<dyn DatabaseExecutor>,
        param_keys: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            executor: Some(executor),
            param_keys: Some(param_keys),
        }
    }
}

impl std::fmt::Debug for DBNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DBNode")
            .field("id", &self.id)
            .field("query", &self.query)
            .field("has_executor", &self.executor.is_some())
            .field("param_keys", &self.param_keys)
            .finish()
    }
}

#[async_trait]
impl Node for DBNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("DBNode[{}]: Executing query '{}'", self.id, self.query);

        let executor = self
            .executor
            .as_ref()
            .map(|e| e.clone())
            .unwrap_or_else(|| Arc::new(MockDatabaseExecutor) as Arc<dyn DatabaseExecutor>);

        // Extract params from context based on param_keys
        let params: Vec<String> = if let Some(keys) = &self.param_keys {
            keys.iter()
                .filter_map(|key| {
                    ctx.get(key).map(|value| {
                        // Convert JSON value to string for SQL binding
                        match value {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "null".to_string(),
                            _ => value.to_string(),
                        }
                    })
                })
                .collect()
        } else {
            vec![]
        };

        if !params.is_empty() {
            debug!(
                "DBNode[{}]: Using {} parameter(s) from context: {:?}",
                self.id,
                params.len(),
                params
            );
        }

        let params_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();

        let result = executor
            .execute(&self.query, &params_refs)
            .await
            .map_err(|e| crate::rule::RuleError::Eval(format!("Database error: {}", e)))?;

        debug!("DBNode[{}]: Query result = {:?}", self.id, result);
        ctx.data
            .insert(format!("{}_result", self.id), result.clone());

        Ok(result)
    }
}

// ============================================================
// AINode - Simulates AI/LLM operations
// ============================================================

#[derive(Debug, Clone)]
pub struct AINode {
    pub id: String,
    pub prompt: String,
}

impl AINode {
    pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
        }
    }
}

#[async_trait]
impl Node for AINode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("AINode[{}]: Processing prompt '{}'", self.id, self.prompt);

        // Simulate async AI API call
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Mock AI response based on context
        let context_summary: Vec<String> = ctx.data.keys().cloned().collect();
        let mock_response = serde_json::json!({
            "prompt": self.prompt,
            "response": format!("AI processed: {} with context keys: {:?}", self.prompt, context_summary),
            "confidence": 0.95,
            "model": "mock-gpt-4"
        });

        debug!("AINode[{}]: AI response = {:?}", self.id, mock_response);
        ctx.data
            .insert(format!("{}_result", self.id), mock_response.clone());

        Ok(mock_response)
    }
}

// ============================================================
// GrpcNode - Calls gRPC services
// ============================================================

#[derive(Debug, Clone)]
pub struct GrpcNode {
    pub id: String,
    pub service_url: String,
    pub method: String,
}

impl GrpcNode {
    pub fn new(
        id: impl Into<String>,
        service_url: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            service_url: service_url.into(),
            method: method.into(),
        }
    }
}

#[async_trait]
impl Node for GrpcNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::GrpcNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!(
            "GrpcNode[{}]: Calling gRPC service '{}' method '{}'",
            self.id, self.service_url, self.method
        );

        // Simulate async gRPC call
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Mock gRPC response
        let mock_response = serde_json::json!({
            "service": self.service_url,
            "method": self.method,
            "status": "OK",
            "response": format!("gRPC call to {} completed", self.method),
            "latency_ms": 100
        });

        debug!("GrpcNode[{}]: gRPC response = {:?}", self.id, mock_response);
        ctx.data
            .insert(format!("{}_result", self.id), mock_response.clone());

        Ok(mock_response)
    }
}

// ============================================================
// SubgraphNode - Nested graph execution for reusable components
// ============================================================

use crate::core::executor::Executor;
use crate::core::{Graph, GraphDef};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SubgraphNode {
    pub id: String,
    pub graph_def: GraphDef,
    pub input_mapping: HashMap<String, String>, // parent_key -> child_key
    pub output_mapping: HashMap<String, String>, // child_key -> parent_key
}

impl SubgraphNode {
    pub fn new(id: impl Into<String>, graph_def: GraphDef) -> Self {
        Self {
            id: id.into(),
            graph_def,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
        }
    }

    pub fn with_input_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.input_mapping = mapping;
        self
    }

    pub fn with_output_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.output_mapping = mapping;
        self
    }
}

#[async_trait]
impl Node for SubgraphNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::SubgraphNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!(
            "🔷 SubgraphNode[{}]: Executing nested graph with {} nodes",
            self.id,
            self.graph_def.nodes.len()
        );

        // Create child graph with mapped inputs
        let mut child_graph = Graph::new(self.graph_def.clone());

        // Map inputs from parent context to child context
        for (parent_key, child_key) in &self.input_mapping {
            if let Some(value) = ctx.data.get(parent_key) {
                debug!(
                    "SubgraphNode[{}]: Mapping {} -> {}",
                    self.id, parent_key, child_key
                );
                child_graph
                    .context
                    .data
                    .insert(child_key.clone(), value.clone());
            }
        }

        // Create executor and register nodes from subgraph definition
        let mut executor = Executor::new();

        // TODO: Need to register nodes from graph_def
        // This requires access to node constructors, which should be handled
        // by a NodeFactory or similar pattern

        // Execute child graph
        executor.execute(&mut child_graph).await.map_err(|e| {
            crate::rule::RuleError::Eval(format!("Subgraph execution failed: {}", e))
        })?;

        // Map outputs from child context back to parent context
        for (child_key, parent_key) in &self.output_mapping {
            if let Some(value) = child_graph.context.data.get(child_key) {
                debug!(
                    "SubgraphNode[{}]: Mapping output {} -> {}",
                    self.id, child_key, parent_key
                );
                ctx.data.insert(parent_key.clone(), value.clone());
            }
        }

        // Store subgraph result
        let result = serde_json::json!({
            "status": "completed",
            "nodes_executed": self.graph_def.nodes.len()
        });

        ctx.data
            .insert(format!("{}_result", self.id), result.clone());

        info!("✅ SubgraphNode[{}]: Completed successfully", self.id);
        Ok(result)
    }
}

// ============================================================
// ConditionalNode - If/else branching with dynamic routing
// ============================================================

#[derive(Debug, Clone)]
pub struct ConditionalNode {
    pub id: String,
    pub condition: String,
    pub true_branch: Option<String>,  // Node ID to route to if true
    pub false_branch: Option<String>, // Node ID to route to if false
}

impl ConditionalNode {
    pub fn new(id: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            condition: condition.into(),
            true_branch: None,
            false_branch: None,
        }
    }

    pub fn with_branches(
        mut self,
        true_branch: impl Into<String>,
        false_branch: impl Into<String>,
    ) -> Self {
        self.true_branch = Some(true_branch.into());
        self.false_branch = Some(false_branch.into());
        self
    }
}

#[async_trait]
impl Node for ConditionalNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::ConditionalNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!(
            "🔀 ConditionalNode[{}]: Evaluating condition: {}",
            self.id, self.condition
        );

        // Use existing rule engine to evaluate condition
        use crate::rule::Rule;
        let rule = Rule::new(&self.id, &self.condition);
        let eval_result = rule.evaluate(&ctx.data)?;

        let condition_met = match eval_result {
            Value::Bool(b) => b,
            _ => false,
        };

        let selected_branch = if condition_met {
            self.true_branch.as_ref()
        } else {
            self.false_branch.as_ref()
        };

        let result = serde_json::json!({
            "condition_met": condition_met,
            "selected_branch": selected_branch,
            "condition": self.condition
        });

        ctx.data
            .insert(format!("{}_result", self.id), result.clone());
        ctx.data.insert(
            "_branch_taken".to_string(),
            Value::String(
                selected_branch
                    .cloned()
                    .unwrap_or_else(|| "none".to_string()),
            ),
        );

        info!(
            "✅ ConditionalNode[{}]: Branch selected: {:?}",
            self.id, selected_branch
        );

        Ok(result)
    }
}

// ============================================================
// LoopNode - While loops and collection iteration
// ============================================================

#[derive(Debug, Clone)]
pub struct LoopNode {
    pub id: String,
    pub condition: String,              // Loop while this is true
    pub max_iterations: usize,          // Safety limit
    pub body_node_id: Option<String>,   // Node to execute in loop body
    pub collection_key: Option<String>, // For iterating over arrays
}

impl LoopNode {
    pub fn new_while(
        id: impl Into<String>,
        condition: impl Into<String>,
        max_iterations: usize,
    ) -> Self {
        Self {
            id: id.into(),
            condition: condition.into(),
            max_iterations,
            body_node_id: None,
            collection_key: None,
        }
    }

    pub fn new_foreach(id: impl Into<String>, collection_key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            condition: "true".to_string(),
            max_iterations: 10000,
            body_node_id: None,
            collection_key: Some(collection_key.into()),
        }
    }

    pub fn with_body_node(mut self, body_node_id: impl Into<String>) -> Self {
        self.body_node_id = Some(body_node_id.into());
        self
    }
}

#[async_trait]
impl Node for LoopNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::LoopNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("🔁 LoopNode[{}]: Starting loop execution", self.id);

        let mut iterations = 0;
        let mut loop_results = Vec::new();

        // Check if this is a collection iteration
        if let Some(collection_key) = &self.collection_key {
            // Clone collection first to avoid borrow checker issues
            let collection_clone = ctx.data.get(collection_key).cloned();

            if let Some(collection) = collection_clone {
                if let Some(array) = collection.as_array() {
                    info!(
                        "LoopNode[{}]: Iterating over collection with {} items",
                        self.id,
                        array.len()
                    );

                    for (index, item) in array.iter().enumerate() {
                        if iterations >= self.max_iterations {
                            info!(
                                "⚠️  LoopNode[{}]: Max iterations ({}) reached",
                                self.id, self.max_iterations
                            );
                            break;
                        }

                        ctx.data
                            .insert("_loop_index".to_string(), Value::from(index));
                        ctx.data.insert("_loop_item".to_string(), item.clone());

                        // TODO: Execute body node if specified
                        // This requires access to executor or node registry

                        loop_results.push(serde_json::json!({
                            "iteration": index,
                            "item": item
                        }));

                        iterations += 1;
                    }
                }
            }
        } else {
            // While loop
            use crate::rule::Rule;
            let rule = Rule::new(&self.id, &self.condition);

            while iterations < self.max_iterations {
                let eval_result = rule.evaluate(&ctx.data)?;
                let should_continue = match eval_result {
                    Value::Bool(b) => b,
                    _ => false,
                };

                if !should_continue {
                    break;
                }

                ctx.data
                    .insert("_loop_iteration".to_string(), Value::from(iterations));

                // TODO: Execute body node if specified

                loop_results.push(serde_json::json!({
                    "iteration": iterations
                }));

                iterations += 1;
            }
        }

        let result = serde_json::json!({
            "iterations": iterations,
            "completed": iterations < self.max_iterations,
            "results": loop_results
        });

        ctx.data
            .insert(format!("{}_result", self.id), result.clone());

        info!(
            "✅ LoopNode[{}]: Completed {} iterations",
            self.id, iterations
        );
        Ok(result)
    }
}

// ============================================================
// TryCatchNode - Error handling with fallback
// ============================================================

#[derive(Debug, Clone)]
pub struct TryCatchNode {
    pub id: String,
    pub try_node_id: String,
    pub catch_node_id: Option<String>,
    pub finally_node_id: Option<String>,
}

impl TryCatchNode {
    pub fn new(id: impl Into<String>, try_node_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            try_node_id: try_node_id.into(),
            catch_node_id: None,
            finally_node_id: None,
        }
    }

    pub fn with_catch(mut self, catch_node_id: impl Into<String>) -> Self {
        self.catch_node_id = Some(catch_node_id.into());
        self
    }

    pub fn with_finally(mut self, finally_node_id: impl Into<String>) -> Self {
        self.finally_node_id = Some(finally_node_id.into());
        self
    }
}

#[async_trait]
impl Node for TryCatchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::TryCatchNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("🛡️  TryCatchNode[{}]: Executing try block", self.id);

        // TODO: Execute try_node_id
        // For now, simulate success/failure based on context
        let error_occurred = ctx
            .data
            .get("_simulate_error")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let result = if error_occurred {
            info!(
                "⚠️  TryCatchNode[{}]: Error occurred, executing catch block",
                self.id
            );
            ctx.data.insert(
                "_error".to_string(),
                Value::String("Simulated error".to_string()),
            );

            serde_json::json!({
                "status": "error_handled",
                "try_node": self.try_node_id,
                "catch_node": self.catch_node_id
            })
        } else {
            serde_json::json!({
                "status": "success",
                "try_node": self.try_node_id
            })
        };

        // TODO: Execute finally block if specified

        ctx.data
            .insert(format!("{}_result", self.id), result.clone());
        Ok(result)
    }
}

// ============================================================
// RetryNode - Automatic retry with exponential backoff
// ============================================================

use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct RetryNode {
    pub id: String,
    pub target_node_id: String,
    pub max_retries: usize,
    pub initial_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl RetryNode {
    pub fn new(
        id: impl Into<String>,
        target_node_id: impl Into<String>,
        max_retries: usize,
    ) -> Self {
        Self {
            id: id.into(),
            target_node_id: target_node_id.into(),
            max_retries,
            initial_delay_ms: 100,
            backoff_multiplier: 2.0,
        }
    }

    pub fn with_backoff(mut self, initial_delay_ms: u64, multiplier: f64) -> Self {
        self.initial_delay_ms = initial_delay_ms;
        self.backoff_multiplier = multiplier;
        self
    }
}

#[async_trait]
impl Node for RetryNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::RetryNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!(
            "🔄 RetryNode[{}]: Starting with max {} retries",
            self.id, self.max_retries
        );

        let mut attempt = 0;
        let mut delay_ms = self.initial_delay_ms;

        while attempt <= self.max_retries {
            // TODO: Execute target_node_id
            // For now, simulate retry logic

            let should_retry = ctx
                .data
                .get("_simulate_failure")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                && attempt < self.max_retries;

            if !should_retry {
                let result = serde_json::json!({
                    "status": "success",
                    "attempts": attempt + 1,
                    "target_node": self.target_node_id
                });
                ctx.data
                    .insert(format!("{}_result", self.id), result.clone());
                info!(
                    "✅ RetryNode[{}]: Succeeded after {} attempts",
                    self.id,
                    attempt + 1
                );
                return Ok(result);
            }

            info!(
                "⚠️  RetryNode[{}]: Attempt {} failed, retrying in {}ms",
                self.id,
                attempt + 1,
                delay_ms
            );

            sleep(Duration::from_millis(delay_ms)).await;
            delay_ms = (delay_ms as f64 * self.backoff_multiplier) as u64;
            attempt += 1;
        }

        let error_result = serde_json::json!({
            "status": "failed",
            "attempts": attempt,
            "target_node": self.target_node_id
        });

        ctx.data
            .insert(format!("{}_result", self.id), error_result.clone());
        info!(
            "❌ RetryNode[{}]: Failed after {} attempts",
            self.id, attempt
        );

        Ok(error_result)
    }
}

// ============================================================
// CircuitBreakerNode - Prevent cascading failures
// ============================================================

#[derive(Debug, Clone)]
pub struct CircuitBreakerNode {
    pub id: String,
    pub target_node_id: String,
    pub failure_threshold: usize,
    pub timeout_ms: u64,
    pub half_open_timeout_ms: u64,
}

impl CircuitBreakerNode {
    pub fn new(
        id: impl Into<String>,
        target_node_id: impl Into<String>,
        failure_threshold: usize,
    ) -> Self {
        Self {
            id: id.into(),
            target_node_id: target_node_id.into(),
            failure_threshold,
            timeout_ms: 5000,
            half_open_timeout_ms: 30000,
        }
    }
}

#[async_trait]
impl Node for CircuitBreakerNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::CircuitBreakerNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("⚡ CircuitBreakerNode[{}]: Checking circuit state", self.id);

        // TODO: Implement proper circuit breaker state machine
        // States: Closed, Open, HalfOpen
        // For now, simple implementation

        let is_circuit_open = ctx
            .data
            .get("_circuit_open")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if is_circuit_open {
            info!(
                "🚫 CircuitBreakerNode[{}]: Circuit is OPEN, fast-failing",
                self.id
            );
            let result = serde_json::json!({
                "status": "circuit_open",
                "message": "Circuit breaker is open, request rejected"
            });
            ctx.data
                .insert(format!("{}_result", self.id), result.clone());
            return Ok(result);
        }

        // Circuit is closed, execute target node
        // TODO: Execute target_node_id and track failures

        let result = serde_json::json!({
            "status": "success",
            "circuit_state": "closed",
            "target_node": self.target_node_id
        });

        ctx.data
            .insert(format!("{}_result", self.id), result.clone());
        info!("✅ CircuitBreakerNode[{}]: Request completed", self.id);

        Ok(result)
    }
}
