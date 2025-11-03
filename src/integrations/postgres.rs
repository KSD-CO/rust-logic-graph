//! PostgreSQL integration with connection pooling
//!
//! Provides async database operations using sqlx

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleResult, RuleError};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{PgPool, Row, Column};
use tracing::{info, error};

/// PostgreSQL database node
#[derive(Debug, Clone)]
pub struct PostgresNode {
    pub id: String,
    pub query: String,
    pub pool: Option<PgPool>,
}

impl PostgresNode {
    /// Create a new PostgreSQL node
    pub fn new(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            pool: None,
        }
    }

    /// Initialize with connection pool
    pub async fn with_pool(mut self, database_url: &str) -> Result<Self, RuleError> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to connect to PostgreSQL: {}", e)))?;
        self.pool = Some(pool);
        Ok(self)
    }

    /// Execute query and return results
    async fn execute_query(&self, query: &str, ctx: &Context) -> Result<Vec<Value>, RuleError> {
        let pool = self.pool.as_ref()
            .ok_or_else(|| RuleError::Eval("PostgreSQL pool not initialized".to_string()))?;

        // Replace placeholders from context
        let processed_query = self.process_query(query, ctx);

        info!("PostgresNode[{}]: Executing query: {}", self.id, processed_query);

        let rows = sqlx::query(&processed_query)
            .fetch_all(pool)
            .await
            .map_err(|e| RuleError::Eval(format!("Query execution failed: {}", e)))?;

        // Convert rows to JSON
        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();

            // Extract columns dynamically
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

                // Try to extract value (simplified - you'd want more types)
                if let Ok(val) = row.try_get::<String, _>(i) {
                    obj.insert(column_name.to_string(), Value::String(val));
                } else if let Ok(val) = row.try_get::<i64, _>(i) {
                    obj.insert(column_name.to_string(), Value::Number(val.into()));
                } else if let Ok(val) = row.try_get::<f64, _>(i) {
                    if let Some(num) = serde_json::Number::from_f64(val) {
                        obj.insert(column_name.to_string(), Value::Number(num));
                    }
                } else if let Ok(val) = row.try_get::<bool, _>(i) {
                    obj.insert(column_name.to_string(), Value::Bool(val));
                }
            }

            results.push(Value::Object(obj));
        }

        Ok(results)
    }

    /// Replace query placeholders with context values
    fn process_query(&self, query: &str, ctx: &Context) -> String {
        let mut processed = query.to_string();

        // Replace {{key}} with context values
        for (key, value) in &ctx.data {
            let placeholder = format!("{{{{{}}}}}", key);
            if processed.contains(&placeholder) {
                let replacement = match value {
                    Value::String(s) => format!("'{}'", s),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => continue,
                };
                processed = processed.replace(&placeholder, &replacement);
            }
        }

        processed
    }
}

#[async_trait]
impl Node for PostgresNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("PostgresNode[{}]: Starting execution", self.id);

        match self.execute_query(&self.query, ctx).await {
            Ok(results) => {
                info!("PostgresNode[{}]: Query returned {} rows", self.id, results.len());

                // Store results in context
                ctx.data.insert(format!("{}_result", self.id), Value::Array(results.clone()));
                ctx.data.insert(format!("{}_count", self.id), Value::Number(results.len().into()));

                Ok(Value::Array(results))
            }
            Err(e) => {
                error!("PostgresNode[{}]: Query failed: {}", self.id, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_query_processing() {
        let node = PostgresNode::new("test", "SELECT * FROM users WHERE id = {{user_id}}");
        let mut ctx = Context {
            data: HashMap::new(),
        };
        ctx.data.insert("user_id".to_string(), Value::Number(42.into()));

        let processed = node.process_query(&node.query, &ctx);
        assert_eq!(processed, "SELECT * FROM users WHERE id = 42");
    }
}
