/// Example: Error handling with Try/Catch pattern
/// 
/// This example demonstrates TryCatchNode for graceful error handling
/// and recovery strategies.

use rust_logic_graph::Context;
use rust_logic_graph::node::{TryCatchNode, Node};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Try/Catch Flow Example ===\n");

    // Example 1: Successful operation (no error)
    println!("Example 1: Normal execution (no errors)");
    
    let try_catch: Box<dyn Node> = Box::new(TryCatchNode::new(
        "safe_operation",
        "risky_database_call"
    )
    .with_catch("fallback_handler")
    .with_finally("cleanup_resources"));

    let mut ctx = Context::new();
    ctx.set("_simulate_error", json!(false));

    let result = try_catch.run(&mut ctx).await?;
    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!("Status: {}\n", result.get("status").unwrap());

    // Example 2: Error occurs, catch handler executes
    println!("Example 2: Error occurs and is caught");
    
    let try_catch: Box<dyn Node> = Box::new(TryCatchNode::new(
        "error_prone_task",
        "might_fail_operation"
    )
    .with_catch("error_recovery"));

    let mut ctx = Context::new();
    ctx.set("_simulate_error", json!(true));

    let result = try_catch.run(&mut ctx).await?;
    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!("Status: {}", result.get("status").unwrap());
    
    if let Some(error) = ctx.data.get("_error") {
        println!("Error message: {}", error);
    }

    Ok(())
}
