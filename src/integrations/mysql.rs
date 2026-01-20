//! MySQL integration with connection pooling
//!
//! Provides async database operations using sqlx

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleError, RuleResult};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{Column, MySqlPool, Row};
use tracing::{error, info};

/// MySQL database node
#[derive(Debug, Clone)]
pub struct MySqlNode {
    pub id: String,
    pub query: String,
    pub pool: Option<MySqlPool>,
}

impl MySqlNode {
    /// Create a new MySQL node
    pub fn new(id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            pool: None,
        }
    }

    /// Initialize with connection pool
    pub async fn with_pool(mut self, database_url: &str) -> Result<Self, RuleError> {
        let pool = MySqlPool::connect(database_url)
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to connect to MySQL: {}", e)))?;
        self.pool = Some(pool);
        Ok(self)
    }

    /// Execute query and return results
    async fn execute_query(&self, query: &str, ctx: &Context) -> Result<Vec<Value>, RuleError> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| RuleError::Eval("MySQL pool not initialized".to_string()))?;

        let processed_query = self.process_query(query, ctx);
        info!(
            "MySqlNode[{}]: Executing query: {}",
            self.id, processed_query
        );

        let rows = sqlx::query(&processed_query)
            .fetch_all(pool)
            .await
            .map_err(|e| RuleError::Eval(format!("Query execution failed: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();

            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

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

    fn process_query(&self, query: &str, ctx: &Context) -> String {
        let mut processed = query.to_string();

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
impl Node for MySqlNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("MySqlNode[{}]: Starting execution", self.id);

        match self.execute_query(&self.query, ctx).await {
            Ok(results) => {
                info!(
                    "MySqlNode[{}]: Query returned {} rows",
                    self.id,
                    results.len()
                );
                ctx.data
                    .insert(format!("{}_result", self.id), Value::Array(results.clone()));
                ctx.data.insert(
                    format!("{}_count", self.id),
                    Value::Number(results.len().into()),
                );
                Ok(Value::Array(results))
            }
            Err(e) => {
                error!("MySqlNode[{}]: Query failed: {}", self.id, e);
                Err(e)
            }
        }
    }
}
