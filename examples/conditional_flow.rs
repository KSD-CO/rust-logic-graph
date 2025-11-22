/// Example: Conditional branching with if/else logic
/// 
/// This example demonstrates how to use ConditionalNode to route execution
/// based on runtime conditions.

use rust_logic_graph::{Graph, GraphDef, Edge, Context};
use rust_logic_graph::node::{ConditionalNode, RuleNode};
use rust_logic_graph::Executor;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Conditional Flow Example ===\n");

    // Create a simple conditional workflow:
    // check_inventory -> [if available > 100] -> process_order
    //                 -> [else]                -> notify_supplier

    let mut executor = Executor::new();

    // Node 1: Check inventory
    let check_inventory = RuleNode::new("check_inventory", "true");
    executor.register_node(Box::new(check_inventory));

    // Node 2: Conditional router
    let condition = ConditionalNode::new(
        "route_based_on_stock",
        "available > 100"  // Condition to evaluate
    ).with_branches("process_order", "notify_supplier");
    executor.register_node(Box::new(condition));

    // Node 3: Process order (if condition is true)
    let process_order = RuleNode::new("process_order", "true");
    executor.register_node(Box::new(process_order));

    // Node 4: Notify supplier (if condition is false)
    let notify_supplier = RuleNode::new("notify_supplier", "true");
    executor.register_node(Box::new(notify_supplier));

    // Create graph definition
    let graph_def = GraphDef {
        nodes: vec![
            ("check_inventory".to_string(), Default::default()),
            ("route_based_on_stock".to_string(), Default::default()),
            ("process_order".to_string(), Default::default()),
            ("notify_supplier".to_string(), Default::default()),
        ].into_iter().collect(),
        edges: vec![
            Edge::new("check_inventory", "route_based_on_stock"),
            Edge::new("route_based_on_stock", "process_order"),
            Edge::new("route_based_on_stock", "notify_supplier"),
        ],
    };

    // Test Case 1: High inventory (should take process_order branch)
    println!("Test 1: High inventory (available = 150)");
    let mut graph = Graph::new(graph_def.clone());
    graph.context.set("available", json!(150));
    
    executor.execute(&mut graph).await?;
    
    let branch_taken = graph.context.get("_branch_taken");
    println!("Branch taken: {:?}", branch_taken);
    println!("Result: {:?}\n", graph.context.data.get("route_based_on_stock_result"));

    // Test Case 2: Low inventory (should take notify_supplier branch)
    println!("Test 2: Low inventory (available = 50)");
    let mut graph = Graph::new(graph_def.clone());
    graph.context.set("available", json!(50));
    
    executor.execute(&mut graph).await?;
    
    let branch_taken = graph.context.get("_branch_taken");
    println!("Branch taken: {:?}", branch_taken);
    println!("Result: {:?}", graph.context.data.get("route_based_on_stock_result"));

    Ok(())
}
