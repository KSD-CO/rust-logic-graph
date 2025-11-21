
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tracing::{info, debug};
use std::sync::Arc;

use crate::core::Context;
use crate::rule::RuleResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    RuleNode,
    DBNode,
    AINode,
    GrpcNode,
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
        info!("RuleNode[{}]: Evaluating condition '{}'", self.id, self.condition);

        // Simple condition evaluation (can be extended with proper parser)
        let result = if self.condition == "true" {
            Value::Bool(true)
        } else if self.condition == "false" {
            Value::Bool(false)
        } else {
            // Try to evaluate based on context
            ctx.data.get(&self.condition).cloned().unwrap_or(Value::Bool(true))
        };

        debug!("RuleNode[{}]: Result = {:?}", self.id, result);
        ctx.data.insert(format!("{}_result", self.id), result.clone());

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
}

impl DBNode {
    pub fn new(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            executor: None,
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
        }
    }
}

impl std::fmt::Debug for DBNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DBNode")
            .field("id", &self.id)
            .field("query", &self.query)
            .field("has_executor", &self.executor.is_some())
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

        let executor = self.executor.as_ref()
            .map(|e| e.clone())
            .unwrap_or_else(|| Arc::new(MockDatabaseExecutor) as Arc<dyn DatabaseExecutor>);
        
        // Get params from context (if any)
        let params: Vec<String> = vec![];  // TODO: Extract from context if needed
        let params_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
        
        let result = executor.execute(&self.query, &params_refs).await
            .map_err(|e| crate::rule::RuleError::Eval(format!("Database error: {}", e)))?;

        debug!("DBNode[{}]: Query result = {:?}", self.id, result);
        ctx.data.insert(format!("{}_result", self.id), result.clone());

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
        ctx.data.insert(format!("{}_result", self.id), mock_response.clone());

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
    pub fn new(id: impl Into<String>, service_url: impl Into<String>, method: impl Into<String>) -> Self {
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
        info!("GrpcNode[{}]: Calling gRPC service '{}' method '{}'", 
              self.id, self.service_url, self.method);

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
        ctx.data.insert(format!("{}_result", self.id), mock_response.clone());

        Ok(mock_response)
    }
}

