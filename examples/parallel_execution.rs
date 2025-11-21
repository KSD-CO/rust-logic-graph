//! Parallel Execution Example
//!
//! This example demonstrates parallel node execution with:
//! - Automatic layer detection
//! - Concurrent execution of independent nodes
//! - Parallelism analysis and statistics
//! - Performance comparison with sequential execution
//!
//! To run: cargo run --example parallel_execution

use rust_logic_graph::{Graph, GraphDef, Node, NodeType};
use rust_logic_graph::parallel::{ParallelExecutor, ParallelConfig};
use rust_logic_graph::node::RuleNode;
use std::collections::HashMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 Parallel Execution Example\n");

    // Example 1: Diamond Graph (Classic Parallel Pattern)
    println!("=== Example 1: Diamond Graph ===\n");
    println!("Graph structure:");
    println!("      A");
    println!("     / \\");
    println!("    B   C   <- Can run in parallel");
    println!("     \\ /");
    println!("      D\n");

    let diamond_graph = create_diamond_graph();
    analyze_and_execute(diamond_graph).await?;

    // Example 2: Wide Graph (Maximum Parallelism)
    println!("\n=== Example 2: Wide Graph (Maximum Parallelism) ===\n");
    println!("Graph structure:");
    println!("    A");
    println!("  / | | \\ ");
    println!(" B  C D  E  <- All run in parallel");
    println!("  \\ | | /");
    println!("    F\n");

    let wide_graph = create_wide_graph();
    analyze_and_execute(wide_graph).await?;

    // Example 3: Linear Graph (No Parallelism)
    println!("\n=== Example 3: Linear Graph (No Parallelism) ===\n");
    println!("Graph structure: A -> B -> C -> D -> E\n");

    let linear_graph = create_linear_graph();
    analyze_and_execute(linear_graph).await?;

    // Example 4: Complex Graph (Mixed Parallelism)
    println!("\n=== Example 4: Complex Graph (Mixed Parallelism) ===\n");
    println!("Graph structure:");
    println!("      A");
    println!("     / \\");
    println!("    B   C");
    println!("   /|   |\\");
    println!("  D E   F G  <- Multiple parallel layers");
    println!("   \\|   |/");
    println!("    H   I");
    println!("     \\ /");
    println!("      J\n");

    let complex_graph = create_complex_graph();
    analyze_and_execute(complex_graph).await?;

    // Example 5: Performance Comparison
    println!("\n=== Example 5: Performance Comparison ===\n");
    compare_performance().await?;

    println!("\n=== Benefits of Parallel Execution ===");
    println!("  • Reduced execution time for independent nodes");
    println!("  • Better resource utilization");
    println!("  • Scalable to large graphs");
    println!("  • Automatic parallelism detection");
    println!("  • No manual scheduling required");
    println!("\n🎉 Example completed!");

    Ok(())
}

/// Create a diamond-shaped graph
fn create_diamond_graph() -> GraphDef {
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), NodeType::RuleNode);
    nodes.insert("B".to_string(), NodeType::RuleNode);
    nodes.insert("C".to_string(), NodeType::RuleNode);
    nodes.insert("D".to_string(), NodeType::RuleNode);

    let mut edges = Vec::new();
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "B".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "C".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "B".to_string(),
        to: "D".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "C".to_string(),
        to: "D".to_string(),
        rule: None,
    });

    GraphDef::from_node_types(nodes, edges)
}

/// Create a wide graph with maximum parallelism in the middle
fn create_wide_graph() -> GraphDef {
    let mut nodes = HashMap::new();
    nodes.insert("A".to_string(), NodeType::RuleNode);
    nodes.insert("B".to_string(), NodeType::RuleNode);
    nodes.insert("C".to_string(), NodeType::RuleNode);
    nodes.insert("D".to_string(), NodeType::RuleNode);
    nodes.insert("E".to_string(), NodeType::RuleNode);
    nodes.insert("F".to_string(), NodeType::RuleNode);

    let mut edges: Vec<rust_logic_graph::Edge> = Vec::new();

    let mut edges = Vec::new();
    // A -> B, C, D, E
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "B".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "C".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "D".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "E".to_string(),
        rule: None,
    });

    // B, C, D, E -> F
    edges.push(rust_logic_graph::Edge {
        from: "B".to_string(),
        to: "F".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "C".to_string(),
        to: "F".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "D".to_string(),
        to: "F".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "E".to_string(),
        to: "F".to_string(),
        rule: None,
    });

    GraphDef::from_node_types(nodes, edges)
}

/// Create a linear graph with no parallelism
fn create_linear_graph() -> GraphDef {
    let node_names = vec!["A", "B", "C", "D", "E"];
    
    let mut nodes = HashMap::new();
    for node in &node_names {
        nodes.insert(node.to_string(), NodeType::RuleNode);
    }

    let mut edges = Vec::new();
    for i in 0..node_names.len() - 1 {
        edges.push(rust_logic_graph::Edge {
            from: node_names[i].to_string(),
            to: node_names[i + 1].to_string(),
            rule: None,
        });
    }

    GraphDef::from_node_types(nodes, edges)
}

/// Create a complex graph with mixed parallelism
fn create_complex_graph() -> GraphDef {
    let node_names = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    
    let mut nodes = HashMap::new();
    for node in &node_names {
        nodes.insert(node.to_string(), NodeType::RuleNode);
    }

    let mut edges = Vec::new();
    // Layer 1: A
    // Layer 2: B, C
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "B".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "A".to_string(),
        to: "C".to_string(),
        rule: None,
    });

    // Layer 3: D, E, F, G
    edges.push(rust_logic_graph::Edge {
        from: "B".to_string(),
        to: "D".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "B".to_string(),
        to: "E".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "C".to_string(),
        to: "F".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "C".to_string(),
        to: "G".to_string(),
        rule: None,
    });

    // Layer 4: H, I
    edges.push(rust_logic_graph::Edge {
        from: "D".to_string(),
        to: "H".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "E".to_string(),
        to: "H".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "F".to_string(),
        to: "I".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "G".to_string(),
        to: "I".to_string(),
        rule: None,
    });

    // Layer 5: J
    edges.push(rust_logic_graph::Edge {
        from: "H".to_string(),
        to: "J".to_string(),
        rule: None,
    });
    edges.push(rust_logic_graph::Edge {
        from: "I".to_string(),
        to: "J".to_string(),
        rule: None,
    });

    GraphDef::from_node_types(nodes, edges)
}

/// Analyze parallelism and execute the graph
async fn analyze_and_execute(def: GraphDef) -> anyhow::Result<()> {
    // Create parallel executor
    let mut executor = ParallelExecutor::new(ParallelConfig {
        max_concurrent: 10,
        verbose: true,
    });

    // Register nodes
    for node_id in def.nodes.keys() {
        let node: Box<dyn Node> = Box::new(RuleNode::new(node_id, "true"));
        executor.register_node(node);
    }

    // Analyze parallelism
    let stats = executor.get_parallelism_stats(&def)?;
    stats.print_summary();

    // Execute the graph
    let mut graph = Graph::new(def);
    let start = Instant::now();
    executor.execute(&mut graph).await?;
    let duration = start.elapsed();

    println!("Execution completed in {:?}", duration);

    Ok(())
}

/// Compare sequential vs parallel execution performance
async fn compare_performance() -> anyhow::Result<()> {
    println!("Comparing sequential vs parallel execution...\n");

    // Create a graph with good parallelism
    let def = create_wide_graph();

    // Sequential execution simulation
    let sequential_time = def.nodes.len(); // Assume 1 unit per node
    println!("Sequential execution time (simulated): {} units", sequential_time);

    // Parallel execution analysis
    let executor = ParallelExecutor::default();
    let stats = executor.get_parallelism_stats(&def)?;
    let parallel_time = stats.num_layers; // Each layer is 1 unit

    println!("Parallel execution time (simulated): {} units", parallel_time);
    println!("Speedup: {:.2}x", sequential_time as f64 / parallel_time as f64);
    println!("Efficiency: {:.1}%", (stats.theoretical_speedup / def.nodes.len() as f64) * 100.0);

    Ok(())
}
