use rust_logic_graph::DatabaseExecutor;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{MySqlPool, PgPool, Row};
use std::sync::Arc;

/// MySQL implementation of DatabaseExecutor
pub struct MySqlExecutor {
    pool: Arc<MySqlPool>,
}

impl MySqlExecutor {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[async_trait]
impl DatabaseExecutor for MySqlExecutor {
    async fn execute(&self, query: &str, params: &[&str]) -> Result<Value, String> {
        let mut query_builder = sqlx::query(query);
        
        // Bind parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        let row = query_builder
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| format!("MySQL error: {}", e))?;
        
        // Convert row to generic JSON (simple implementation)
        // In production, you'd want more sophisticated column mapping
        Ok(serde_json::json!({
            "executed": true,
            "query": query,
        }))
    }
}

/// PostgreSQL implementation of DatabaseExecutor
pub struct PostgresExecutor {
    pool: Arc<PgPool>,
}

impl PostgresExecutor {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[async_trait]
impl DatabaseExecutor for PostgresExecutor {
    async fn execute(&self, query: &str, params: &[&str]) -> Result<Value, String> {
        // Convert MySQL-style ? to Postgres $1, $2, etc.
        let mut pg_query = query.to_string();
        for (i, _) in params.iter().enumerate() {
            pg_query = pg_query.replacen("?", &format!("${}", i + 1), 1);
        }
        
        let mut query_builder = sqlx::query(&pg_query);
        
        // Bind parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        let row = query_builder
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| format!("Postgres error: {}", e))?;
        
        // Convert row to generic JSON
        Ok(serde_json::json!({
            "executed": true,
            "query": pg_query,
        }))
    }
}
