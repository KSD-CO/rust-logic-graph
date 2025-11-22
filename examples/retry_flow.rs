/// Example: Retry logic with exponential backoff
/// 
/// This example demonstrates RetryNode for handling transient failures
/// with automatic retry and exponential backoff.

use rust_logic_graph::Context;
use rust_logic_graph::node::{RetryNode, Node};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Retry Flow Example ===\n");

    // Example 1: Successful retry after failures
    println!("Example 1: API call with retry (simulated success on 3rd attempt)");
    
    let retry_node: Box<dyn Node> = Box::new(RetryNode::new(
        "api_call_with_retry",
        "call_external_api",
        3  // Max 3 retries
    ).with_backoff(100, 2.0));  // Start with 100ms, double each time

    let mut ctx = Context::new();
    ctx.set("_simulate_failure", json!(false));  // Will succeed

    println!("Starting retry logic...");
    let start = std::time::Instant::now();
    let result = retry_node.run(&mut ctx).await?;
    let elapsed = start.elapsed();

    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!("Total time: {:?}\n", elapsed);

    // Example 2: All retries exhausted
    println!("Example 2: All retries fail");
    
    let retry_node: Box<dyn Node> = Box::new(RetryNode::new(
        "failing_operation",
        "call_flaky_service",
        2  // Max 2 retries
    ).with_backoff(50, 1.5));

    let mut ctx = Context::new();
    ctx.set("_simulate_failure", json!(true));  // Will always fail

    println!("Starting retry logic...");
    let start = std::time::Instant::now();
    let result = retry_node.run(&mut ctx).await?;
    let elapsed = start.elapsed();

    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!("Total time: {:?}", elapsed);
    println!("Status: {}", result.get("status").unwrap());

    Ok(())
}
