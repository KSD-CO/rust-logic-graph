use std::collections::HashMap;
use std::time::Instant;
use std::pin::Pin;
use std::future::Future;
use serde_json::Value;
use tokio::task::JoinSet;
use tracing::{info, debug};

use crate::error::{RustLogicGraphError, ErrorContext};

type BoxedFuture = Pin<Box<dyn Future<Output = Result<Value, RustLogicGraphError>> + Send>>;

/// Result from a single database query
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub database: String,
    pub query: String,
    pub result: Value,
    pub duration_ms: u128,
    pub row_count: usize,
}

/// Parallel Database Executor
/// 
/// Executes multiple database queries concurrently across different databases,
/// collecting results and providing detailed execution statistics.
/// 
/// # Example
/// ```no_run
/// use rust_logic_graph::multi_db::ParallelDBExecutor;
/// 
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut executor = ParallelDBExecutor::new();
///     
///     // Register query closures for different databases
///     executor
///         .add_query("oms_db", "user_query", Box::new(|| Box::pin(async {
///             // Execute query against OMS database
///             Ok(serde_json::json!({"user_id": 123, "name": "John"}))
///         })))
///         .add_query("inventory_db", "stock_query", Box::new(|| Box::pin(async {
///             // Execute query against Inventory database
///             Ok(serde_json::json!({"product_id": "PROD-001", "qty": 50}))
///         })));
///     
///     let results = executor.execute_all().await?;
///     println!("Executed {} queries in parallel", results.len());
///     Ok(())
/// }
/// ```
pub struct ParallelDBExecutor {
    queries: Vec<(String, String, BoxedFuture)>,
    max_concurrent: usize,
}

impl ParallelDBExecutor {
    /// Create a new parallel database executor
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
            max_concurrent: 10, // Default: 10 concurrent queries
        }
    }
    
    /// Set maximum number of concurrent queries
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
    
    /// Add a query to execute
    /// 
    /// # Arguments
    /// * `database` - Database identifier (e.g., "oms_db", "inventory_db")
    /// * `query_id` - Unique query identifier for tracking
    /// * `query_fn` - Async closure that executes the query
    pub fn add_query<F, Fut>(
        &mut self,
        database: impl Into<String>,
        query_id: impl Into<String>,
        query_fn: F,
    ) -> &mut Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<Value, RustLogicGraphError>> + Send + 'static,
    {
        let db = database.into();
        let qid = query_id.into();
        
        // Box the future
        let boxed: BoxedFuture = Box::pin(query_fn());
        self.queries.push((db, qid, boxed));
        self
    }
    
    /// Execute all registered queries in parallel
    /// 
    /// Returns a HashMap with query_id as key and QueryResult as value.
    /// 
    /// # Errors
    /// If any query fails, returns the first encountered error.
    pub async fn execute_all(&mut self) -> Result<HashMap<String, QueryResult>, RustLogicGraphError> {
        let total_start = Instant::now();
        let query_count = self.queries.len();
        
        info!("🚀 Parallel DB Executor: Starting {} queries across databases", query_count);
        
        if query_count == 0 {
            return Ok(HashMap::new());
        }
        
        let mut join_set = JoinSet::new();
        let mut results = HashMap::new();
        
        // Take ownership of queries
        let queries = std::mem::take(&mut self.queries);
        
        // Spawn all queries as concurrent tasks
        for (database, query_id, query_future) in queries {
            let db_clone = database.clone();
            let qid_clone = query_id.clone();
            
            join_set.spawn(async move {
                let start = Instant::now();
                debug!("⏱️  Executing query '{}' on database '{}'", qid_clone, db_clone);
                
                match query_future.await {
                    Ok(result) => {
                        let duration_ms = start.elapsed().as_millis();
                        let row_count = if result.is_array() {
                            result.as_array().map(|arr| arr.len()).unwrap_or(0)
                        } else if result.is_object() {
                            1
                        } else {
                            0
                        };
                        
                        debug!("✅ Query '{}' completed in {}ms ({} rows)", qid_clone, duration_ms, row_count);
                        
                        Ok((query_id, QueryResult {
                            database: db_clone,
                            query: qid_clone,
                            result,
                            duration_ms,
                            row_count,
                        }))
                    }
                    Err(e) => {
                        Err(e.with_context(
                            ErrorContext::new()
                                .with_service(&db_clone)
                                .add_metadata("query_id", &qid_clone)
                        ))
                    }
                }
            });
        }
        
        // Collect all results
        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok(Ok((query_id, query_result))) => {
                    results.insert(query_id, query_result);
                }
                Ok(Err(e)) => {
                    // Cancel remaining tasks and return error
                    join_set.abort_all();
                    return Err(e);
                }
                Err(join_err) => {
                    join_set.abort_all();
                    return Err(RustLogicGraphError::node_execution_error(
                        "parallel_executor",
                        format!("Task join error: {}", join_err)
                    ));
                }
            }
        }
        
        let total_duration_ms = total_start.elapsed().as_millis();
        let total_rows: usize = results.values().map(|r| r.row_count).sum();
        
        info!("✅ Parallel DB Executor: Completed {} queries in {}ms ({} total rows)", 
            query_count, total_duration_ms, total_rows);
        
        // Log per-database statistics
        let mut db_stats: HashMap<String, (usize, u128)> = HashMap::new();
        for result in results.values() {
            let entry = db_stats.entry(result.database.clone()).or_insert((0, 0));
            entry.0 += 1; // query count
            entry.1 += result.duration_ms; // total duration
        }
        
        for (db, (count, duration)) in db_stats {
            info!("  📊 {}: {} queries, {}ms total", db, count, duration);
        }
        
        Ok(results)
    }
}

impl Default for ParallelDBExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_parallel_execution() {
        let mut executor = ParallelDBExecutor::new();
        
        executor
            .add_query("db1", "query1", || async {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok(json!({"result": 1}))
            })
            .add_query("db2", "query2", || async {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok(json!({"result": 2}))
            });
        
        let results = executor.execute_all().await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains_key("query1"));
        assert!(results.contains_key("query2"));
    }
    
    #[tokio::test]
    async fn test_error_propagation() {
        let mut executor = ParallelDBExecutor::new();
        
        executor
            .add_query("db1", "query1", || async {
                Ok(json!({"result": 1}))
            })
            .add_query("db2", "failing_query", || async {
                Err(RustLogicGraphError::database_connection_error("Test error"))
            });
        
        let result = executor.execute_all().await;
        assert!(result.is_err());
    }
}
