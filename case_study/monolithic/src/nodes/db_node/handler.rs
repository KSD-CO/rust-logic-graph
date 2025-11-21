use rust_logic_graph::{NodeType, Context};
use rust_logic_graph::node::Node;
use rust_logic_graph::rule::RuleResult;
use serde_json::Value;
use async_trait::async_trait;

use super::types::DatabasePool;
use super::parser::{parse_mysql_row, parse_postgres_row};

/// Dynamic DB node that executes queries from YAML config
pub struct DynamicDBNode {
    id: String,
    query: String,
    pool: DatabasePool,
    product_id: String,
}

impl DynamicDBNode {
    pub fn new(id: String, query: String, pool: DatabasePool, product_id: String) -> Self {
        Self { id, query, pool, product_id }
    }
}

#[async_trait]
impl Node for DynamicDBNode {
    fn id(&self) -> &str { &self.id }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("🗄️  DBNode[{}]: Executing query from YAML config", self.id);
        tracing::debug!("Query: {}", self.query);
        tracing::debug!("Product ID: {}", self.product_id);
        
        // Execute SQL query based on database type
        let result = match &self.pool {
            DatabasePool::MySql(pool) => {
                let row = sqlx::query(&self.query)
                    .bind(&self.product_id)
                    .fetch_one(&**pool)
                    .await
                    .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("MySQL error in {}: {}", self.id, e)))?;
                
                parse_mysql_row(&self.id, row)?
            }
            DatabasePool::Postgres(pool) => {
                // Convert MySQL placeholder ? to Postgres $1, $2, etc.
                let pg_query = self.query.replace("?", "$1");
                
                tracing::debug!("Postgres query: {}", pg_query);
                let row = sqlx::query(&pg_query)
                    .bind(&self.product_id)
                    .fetch_one(&**pool)
                    .await
                    .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Postgres error in {}: {}", self.id, e)))?;
                
                parse_postgres_row(&self.id, row)?
            }
        };
        
        tracing::info!("✅ DBNode[{}]: Query result: {}", self.id, result);
        ctx.data.insert(self.id.clone(), result.clone());
        Ok(result)
    }
}
