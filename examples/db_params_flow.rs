/// Example demonstrating DBNode with context-based parameter extraction
///
/// This example shows how to use the params feature to extract values from
/// the execution context and pass them as SQL query parameters.
use rust_logic_graph::{Edge, Executor, Graph, GraphDef, NodeConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create a graph with DBNodes that use parameters from context
    let mut nodes = HashMap::new();

    // Node 1: DBNode with parameter extracted from context
    // The query uses $1 placeholder, and "product_id" will be extracted from context
    nodes.insert(
        "fetch_product".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM products WHERE product_id = $1",
            vec!["product_id".to_string()],
        ),
    );

    // Node 2: DBNode with multiple parameters
    // Uses $1, $2 placeholders, extracts "user_id" and "status" from context
    nodes.insert(
        "fetch_orders".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM orders WHERE user_id = $1 AND status = $2",
            vec!["user_id".to_string(), "status".to_string()],
        ),
    );

    // Node 3: RuleNode to process results
    nodes.insert("validate".to_string(), NodeConfig::rule_node("true"));

    let edges = vec![
        Edge {
            from: "fetch_product".to_string(),
            to: "fetch_orders".to_string(),
            rule: None,
        },
        Edge {
            from: "fetch_orders".to_string(),
            to: "validate".to_string(),
            rule: None,
        },
    ];

    let def = GraphDef { nodes, edges };
    let mut graph = Graph::new(def);

    // Initialize context with parameter values
    // These will be extracted by the DBNodes
    graph
        .context
        .set("product_id", serde_json::json!("PROD-001"));
    graph.context.set("user_id", serde_json::json!("USER-123"));
    graph.context.set("status", serde_json::json!("pending"));

    println!("\n🚀 Starting graph execution with context parameters:");
    println!("   product_id: PROD-001");
    println!("   user_id: USER-123");
    println!("   status: pending\n");

    // Create executor and run
    let mut executor = Executor::from_graph_def(&graph.def)?;
    executor.execute(&mut graph).await?;

    println!("\n✅ Graph execution completed!");
    println!("\n📊 Execution Metrics:");
    let metrics = executor.metrics();
    println!("   Total duration: {:?}", metrics.total_duration);
    println!("   Nodes executed: {}", metrics.nodes_executed);
    println!("   Nodes skipped: {}", metrics.nodes_skipped);
    println!("   Cache hits: {}", metrics.cache_hits);

    println!("\n📝 Context after execution:");
    for (key, value) in &graph.context.data {
        println!("   {}: {}", key, value);
    }

    Ok(())
}
