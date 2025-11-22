use rust_logic_graph::{NodeType, Context};
use rust_logic_graph::node::Node;
use rust_logic_graph::rule::RuleResult;
use rust_logic_graph::error::{RustLogicGraphError, ErrorContext};
use serde_json::Value;
use async_trait::async_trait;

use super::types::DatabasePool;
use super::parser::{parse_mysql_row, parse_postgres_row};

/// Dynamic DB node that executes queries from YAML config
pub struct DynamicDBNode {
    id: String,
    query: String,
    pool: DatabasePool,
    param_keys: Vec<String>,
}

impl DynamicDBNode {
    pub fn new(id: String, query: String, pool: DatabasePool, param_keys: Vec<String>) -> Self {
        Self { id, query, pool, param_keys }
    }
}

#[async_trait]
impl Node for DynamicDBNode {
    fn id(&self) -> &str { &self.id }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("🗄️  DBNode[{}]: Executing query from YAML config", self.id);
        tracing::debug!("Query: {}", self.query);
        
        // Extract params from context
        let params: Vec<String> = self.param_keys.iter()
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
            .collect();
        
        if !params.is_empty() {
            tracing::debug!("Params from context: {:?}", params);
        }
        
        // Execute SQL query based on database type
        let result = match &self.pool {
            DatabasePool::MySql(pool) => {
                let mut query_builder = sqlx::query(&self.query);
                for param in &params {
                    query_builder = query_builder.bind(param);
                }
                
                let row = query_builder
                    .fetch_one(&**pool)
                    .await
                    .map_err(|e| {
                        rust_logic_graph::rule::RuleError::Eval(
                            RustLogicGraphError::database_connection_error(
                                format!("MySQL query failed: {}", e)
                            )
                            .with_context(
                                ErrorContext::new()
                                    .with_node(&self.id)
                                    .with_graph("purchasing_flow")
                                    .with_step("database_query")
                                    .add_metadata("database_type", "MySQL")
                                    .add_metadata("query", &self.query)
                            )
                            .to_string()
                        )
                    })?;
                
                parse_mysql_row(&self.id, row)?
            }
            DatabasePool::Postgres(pool) => {
                // Convert MySQL placeholder ? to Postgres $1, $2, etc.
                let mut pg_query = self.query.clone();
                for (i, _) in params.iter().enumerate() {
                    pg_query = pg_query.replacen("?", &format!("${}", i + 1), 1);
                }
                
                tracing::debug!("Postgres query: {}", pg_query);
                
                let mut query_builder = sqlx::query(&pg_query);
                for param in &params {
                    query_builder = query_builder.bind(param);
                }
                
                let row = query_builder
                    .fetch_one(&**pool)
                    .await
                    .map_err(|e| {
                        rust_logic_graph::rule::RuleError::Eval(
                            RustLogicGraphError::database_connection_error(
                                format!("PostgreSQL query failed: {}", e)
                            )
                            .with_context(
                                ErrorContext::new()
                                    .with_node(&self.id)
                                    .with_graph("purchasing_flow")
                                    .with_step("database_query")
                                    .add_metadata("database_type", "PostgreSQL")
                                    .add_metadata("query", &pg_query)
                            )
                            .to_string()
                        )
                    })?;
                
                parse_postgres_row(&self.id, row)?
            }
        };
        
        tracing::info!("✅ DBNode[{}]: Query result: {}", self.id, result);
        ctx.data.insert(self.id.clone(), result.clone());
        Ok(result)
    }
}
