
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tracing::{info, debug};

use crate::core::Context;
use crate::rule::RuleResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    RuleNode,
    DBNode,
    AINode,
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
// DBNode - Simulates database operations
// ============================================================

#[derive(Debug, Clone)]
pub struct DBNode {
    pub id: String,
    pub query: String,
}

impl DBNode {
    pub fn new(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
        }
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

        // Simulate async DB operation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Mock database result
        let mock_result = serde_json::json!({
            "query": self.query,
            "rows": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false}
            ],
            "count": 2
        });

        debug!("DBNode[{}]: Query result = {:?}", self.id, mock_result);
        ctx.data.insert(format!("{}_result", self.id), mock_result.clone());

        Ok(mock_result)
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
