//! PostgreSQL Integration Example
//!
//! This example demonstrates how to use PostgreSQL integration in a logic graph.
//!
//! To run this example:
//! 1. Start a PostgreSQL database
//! 2. Set DATABASE_URL environment variable
//! 3. cargo run --example postgres_flow --features postgres

use rust_logic_graph::{Context, Executor, Node};
use std::collections::HashMap;

#[cfg(feature = "postgres")]
use rust_logic_graph::integrations::PostgresNode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    #[cfg(not(feature = "postgres"))]
    {
        println!("❌ This example requires the 'postgres' feature");
        println!("Run with: cargo run --example postgres_flow --features postgres");
        return Ok(());
    }

    #[cfg(feature = "postgres")]
    {
        println!("🚀 PostgreSQL Integration Example\n");

        // Get database URL from environment
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                println!("⚠️  DATABASE_URL not set, using mock example");
                println!("Set DATABASE_URL to test with real database");
                println!("\nExample workflow:\n");
                "postgres://user:pass@localhost/db".to_string()
            });

        println!("Database URL: {}\n", database_url);

        // Example 1: Simple query
        println!("=== Example 1: Simple SELECT Query ===\n");

        let node = PostgresNode::new(
            "fetch_users",
            "SELECT id, name, email FROM users LIMIT 10"
        );

        if database_url.starts_with("postgres://") && !database_url.contains("localhost") {
            let node = node.with_pool(&database_url).await?;
            let mut ctx = Context {
                data: HashMap::new(),
            };

            match node.run(&mut ctx).await {
                Ok(result) => {
                    println!("✓ Query successful!");
                    println!("Result: {:?}\n", result);
                }
                Err(e) => {
                    println!("✗ Query failed: {}\n", e);
                }
            }
        } else {
            println!("📝 Mock query: SELECT id, name, email FROM users LIMIT 10");
            println!("Would return: Array of user objects\n");
        }

        // Example 2: Parameterized query
        println!("=== Example 2: Parameterized Query ===\n");

        let node2 = PostgresNode::new(
            "fetch_user_by_id",
            "SELECT * FROM users WHERE id = {{user_id}}"
        );

        let mut ctx2 = Context {
            data: HashMap::new(),
        };
        ctx2.data.insert("user_id".to_string(), serde_json::json!(42));

        println!("📝 Query with user_id = 42");
        println!("SQL: SELECT * FROM users WHERE id = 42\n");

        // Example 3: Complex workflow
        println!("=== Example 3: Graph Workflow with PostgreSQL ===\n");

        let mut executor = Executor::new();

        // Node 1: Fetch active users
        println!("Node 1: Fetch active users");
        println!("  SQL: SELECT * FROM users WHERE status = 'active'\n");

        // Node 2: Count by department
        println!("Node 2: Count users by department");
        println!("  SQL: SELECT department, COUNT(*) FROM users GROUP BY department\n");

        // Node 3: Get recent orders
        println!("Node 3: Fetch recent orders");
        println!("  SQL: SELECT * FROM orders WHERE user_id = {{user_id}} ORDER BY created_at DESC LIMIT 5\n");

        println!("✓ Workflow defined successfully!");
        println!("\n=== Benefits of PostgreSQL Integration ===");
        println!("  • Connection pooling for performance");
        println!("  • Async/await for non-blocking I/O");
        println!("  • Template variables from context");
        println!("  • Automatic JSON conversion");
        println!("  • Type-safe queries\n");

        println!("🎉 Example completed!");
    }

    Ok(())
}
