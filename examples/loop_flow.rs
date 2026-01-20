use rust_logic_graph::node::{LoopNode, Node};
/// Example: Loop execution with collections and while loops
///
/// This example demonstrates LoopNode for iterating over collections
/// and executing conditional loops.
use rust_logic_graph::Context;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Loop Flow Example ===\n");

    // Example 1: Foreach loop over collection
    println!("Example 1: Iterate over product list");

    let foreach_loop: Box<dyn Node> =
        Box::new(LoopNode::new_foreach("process_products", "products"));

    let mut ctx = Context::new();
    ctx.set(
        "products",
        json!([
            {"id": "PROD-001", "name": "Widget A", "price": 10.99},
            {"id": "PROD-002", "name": "Widget B", "price": 15.50},
            {"id": "PROD-003", "name": "Widget C", "price": 8.75},
        ]),
    );

    let result = foreach_loop.run(&mut ctx).await?;
    println!("Foreach result: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // Example 2: While loop with condition
    println!("Example 2: While loop until counter reaches 5");

    let while_loop: Box<dyn Node> = Box::new(LoopNode::new_while(
        "count_to_five",
        "counter < 5",
        10, // Max 10 iterations for safety
    ));

    let mut ctx = Context::new();
    ctx.set("counter", json!(0));

    let result = while_loop.run(&mut ctx).await?;
    println!(
        "While loop result: {}",
        serde_json::to_string_pretty(&result)?
    );
    println!();

    // Example 3: Limited iteration with max_iterations
    println!("Example 3: Loop with safety limit");

    let limited_loop: Box<dyn Node> = Box::new(LoopNode::new_while(
        "infinite_protection",
        "true", // Always true - would be infinite
        5,      // But limited to 5 iterations
    ));

    let mut ctx = Context::new();
    let result = limited_loop.run(&mut ctx).await?;
    println!(
        "Limited loop result: {}",
        serde_json::to_string_pretty(&result)?
    );
    println!("Iterations: {}", result.get("iterations").unwrap());

    Ok(())
}
