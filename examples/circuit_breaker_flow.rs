use rust_logic_graph::node::{CircuitBreakerNode, Node};
/// Example: Circuit Breaker pattern for preventing cascading failures
///
/// This example demonstrates CircuitBreakerNode for protecting services
/// from overload and preventing cascading failures.
use rust_logic_graph::Context;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Circuit Breaker Example ===\n");

    // Example 1: Circuit closed (normal operation)
    println!("Example 1: Circuit closed - request goes through");

    let circuit_breaker: Box<dyn Node> = Box::new(CircuitBreakerNode::new(
        "protected_service",
        "external_api_call",
        5, // Open circuit after 5 failures
    ));

    let mut ctx = Context::new();
    ctx.set("_circuit_open", json!(false));

    let result = circuit_breaker.run(&mut ctx).await?;
    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!("Circuit state: {}\n", result.get("circuit_state").unwrap());

    // Example 2: Circuit open (fast-fail)
    println!("Example 2: Circuit open - request rejected immediately");

    let circuit_breaker: Box<dyn Node> = Box::new(CircuitBreakerNode::new(
        "failing_service",
        "unreliable_endpoint",
        3,
    ));

    let mut ctx = Context::new();
    ctx.set("_circuit_open", json!(true));

    let result = circuit_breaker.run(&mut ctx).await?;
    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!("Status: {}", result.get("status").unwrap());
    println!("Message: {}", result.get("message").unwrap());

    println!("\n💡 Note: This is a simplified example.");
    println!("   In production, circuit breaker state should be shared");
    println!("   across requests and automatically transition between");
    println!("   Closed → Open → HalfOpen → Closed states.");

    Ok(())
}
