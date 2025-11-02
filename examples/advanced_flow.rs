use rust_logic_graph::{Graph, GraphIO, Executor, RuleNode, DBNode, AINode};
use tracing_subscriber;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== Rust Logic Graph - Advanced Flow Example ===");
    println!("Scenario: User Analytics Report Generation with Permission Checks\n");

    // Load graph definition
    let def = GraphIO::load_from_file("examples/advanced_flow.json")?;

    // Create custom executor with specific node configurations
    let mut executor = Executor::new();

    // Register nodes with custom logic
    executor.register_node(Box::new(RuleNode::new(
        "validate_input",
        "user_id > 0"
    )));

    executor.register_node(Box::new(DBNode::new(
        "fetch_user_data",
        "SELECT * FROM users WHERE id = ?"
    )));

    executor.register_node(Box::new(RuleNode::new(
        "check_permissions",
        "user_role == \"admin\""
    )));

    executor.register_node(Box::new(DBNode::new(
        "query_analytics",
        "SELECT * FROM analytics WHERE user_id = ?"
    )));

    executor.register_node(Box::new(AINode::new(
        "generate_report",
        "Generate comprehensive analytics report from data"
    )));

    executor.register_node(Box::new(AINode::new(
        "send_notification",
        "Send notification to user about report status"
    )));

    // Create graph and set initial context
    let mut graph = Graph::new(def);

    // Simulate input data
    graph.context.data.insert("user_id".to_string(), json!(1001));
    graph.context.data.insert("user_role".to_string(), json!("admin"));

    println!("Initial Context:");
    println!("  - user_id: 1001");
    println!("  - user_role: admin\n");

    // Execute the graph
    println!("Starting advanced flow execution...\n");
    executor.execute(&mut graph).await?;

    // Display results
    println!("\n=== Execution Results ===");
    println!("\nContext Data:");
    for (key, value) in &graph.context.data {
        if key.ends_with("_result") {
            println!("  {}: {}", key, serde_json::to_string_pretty(&value)?);
        }
    }

    println!("\n=== Flow Complete ===");
    Ok(())
}
