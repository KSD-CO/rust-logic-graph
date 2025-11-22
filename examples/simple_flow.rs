
use rust_logic_graph::{Graph, Orchestrator, GraphIO};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Rust Logic Graph - Simple Flow Example ===\n");

    // Load graph definition from YAML
    let def = GraphIO::load_from_file("examples/simple_flow.yaml")?;
    println!("Loaded graph with {} nodes and {} edges\n",
        def.nodes.len(),
        def.edges.len()
    );

    // Create graph
    let mut graph = Graph::new(def);

    // Execute the graph
    println!("Starting graph execution...\n");
    Orchestrator::execute_graph(&mut graph).await?;

    // Display final context
    println!("\n=== Final Context ===");
    for (key, value) in &graph.context.data {
        println!("{}: {}", key, value);
    }

    println!("\n=== Execution Complete ===");
    Ok(())
}
