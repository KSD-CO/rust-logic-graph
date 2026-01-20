use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::RuleResult;

/// A simple node that simulates an expensive computation by sleeping.
pub struct ExpensiveComputeNode {
    pub id: String,
    pub work_ms: u64,
}

impl ExpensiveComputeNode {
    pub fn new(id: &str, work_ms: u64) -> Self {
        Self {
            id: id.to_string(),
            work_ms,
        }
    }
}

#[async_trait]
impl Node for ExpensiveComputeNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        // Treat as a RuleNode-style compute node for benchmarking
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        // Simulate work
        tokio::time::sleep(Duration::from_millis(self.work_ms)).await;
        // Put a value into context and also return it
        let v = json!({ "node": self.id.clone(), "work_ms": self.work_ms });
        ctx.set(&format!("{}_result", self.id), v.clone());
        Ok(v)
    }
}

/// Helper to create a simple graph/executor pair used by benches.
pub fn make_simple_expensive_node(id: &str, ms: u64) -> ExpensiveComputeNode {
    ExpensiveComputeNode::new(id, ms)
}
