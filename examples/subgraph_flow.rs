use rust_logic_graph::node::{Node, SubgraphNode};
/// Example: Subgraph execution for reusable components
///
/// This example demonstrates SubgraphNode for creating modular,
/// reusable workflow components using YAML configuration.
///
/// Files:
/// - examples/discount_subgraph.yaml: Reusable discount calculation logic
/// - examples/order_with_subgraph.yaml: Main workflow that calls the subgraph
use rust_logic_graph::{Context, GraphDef};
use serde_json::json;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Subgraph Flow Example ===\n");

    // Load the reusable discount subgraph from YAML
    println!("📄 Loading subgraph from: examples/discount_subgraph.yaml");
    let subgraph_yaml = fs::read_to_string("examples/discount_subgraph.yaml")?;
    let discount_subgraph: GraphDef = serde_yaml::from_str(&subgraph_yaml)?;

    println!(
        "✅ Subgraph loaded with {} nodes:",
        discount_subgraph.nodes.len()
    );
    for (name, _) in &discount_subgraph.nodes {
        println!("   • {}", name);
    }
    println!();

    // Load the main workflow that uses the subgraph
    println!("📄 Loading main workflow from: examples/order_with_subgraph.yaml");
    let main_yaml = fs::read_to_string("examples/order_with_subgraph.yaml")?;
    let main_workflow: GraphDef = serde_yaml::from_str(&main_yaml)?;

    println!(
        "✅ Main workflow loaded with {} nodes:",
        main_workflow.nodes.len()
    );
    for (name, config) in &main_workflow.nodes {
        println!(
            "   • {} ({})",
            name,
            if name == "calculate_discount" {
                "SubgraphNode"
            } else {
                &format!("{:?}", config.node_type)
            }
        );
    }
    println!();

    // Create the subgraph node manually to demonstrate execution
    println!("🔧 Creating SubgraphNode with input/output mapping:");
    let mut input_mapping = std::collections::HashMap::new();
    input_mapping.insert("product_price".to_string(), "base_price".to_string());
    input_mapping.insert("customer_tier".to_string(), "customer_tier".to_string());
    input_mapping.insert(
        "months_active".to_string(),
        "purchase_history_months".to_string(),
    );

    let mut output_mapping = std::collections::HashMap::new();
    output_mapping.insert("final_price".to_string(), "order_total".to_string());
    output_mapping.insert("discount_amount".to_string(), "savings".to_string());

    println!("   Input mapping:");
    for (parent, child) in &input_mapping {
        println!("      {} → {}", parent, child);
    }
    println!("   Output mapping:");
    for (child, parent) in &output_mapping {
        println!("      {} → {}", child, parent);
    }
    println!();

    let subgraph_node: Box<dyn Node> = Box::new(
        SubgraphNode::new("discount_calculator", discount_subgraph)
            .with_input_mapping(input_mapping)
            .with_output_mapping(output_mapping),
    );

    // Simulate execution with sample data
    println!("🚀 Executing subgraph with sample data:");
    let mut ctx = Context::new();
    ctx.set("product_price", json!(100.0));
    ctx.set("customer_tier", json!("gold"));
    ctx.set("months_active", json!(18));

    println!("   Input:");
    println!(
        "      product_price: ${}",
        ctx.get("product_price").unwrap()
    );
    println!("      customer_tier: {}", ctx.get("customer_tier").unwrap());
    println!(
        "      months_active: {} months",
        ctx.get("months_active").unwrap()
    );
    println!();

    let result = subgraph_node.run(&mut ctx).await?;

    println!("✅ Subgraph execution completed:");
    println!("{}", serde_json::to_string_pretty(&result)?);
    println!();

    if let Some(order_total) = ctx.get("order_total") {
        println!("📊 Results:");
        println!("   Order total: ${}", order_total);
        if let Some(savings) = ctx.get("savings") {
            println!("   You saved: ${}", savings);
        }
    }

    println!("\n💡 Benefits of YAML-based subgraphs:");
    println!("   ✅ Reusable components across multiple workflows");
    println!("   ✅ No recompilation needed for workflow changes");
    println!("   ✅ Clear separation of business logic");
    println!("   ✅ Easy to version control and review");
    println!("   ✅ Encapsulation and modularity");
    println!("   ✅ Easier testing and maintenance");

    println!("\n🔍 Inspect the YAML files:");
    println!("   cat examples/discount_subgraph.yaml");
    println!("   cat examples/order_with_subgraph.yaml");

    Ok(())
}
