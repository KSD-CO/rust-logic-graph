use rust_logic_graph::{Executor, Graph, GraphDef, NodeType};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

// Run manual micro-runs and append simple CSV (name, avg_seconds)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tests = vec![
        ("rule_1", NodeType::RuleNode, 1usize),
        ("db_5", NodeType::DBNode, 5usize),
        ("ai_20", NodeType::AINode, 20usize),
    ];

    let mut w = OpenOptions::new()
        .create(true)
        .append(true)
        .open("target/bench_results.csv")?;

    for (name, node_type, size) in tests {
        // Build graph
        let graph_def = {
            let mut nodes = std::collections::HashMap::new();
            let mut edges = Vec::new();
            for i in 0..size {
                nodes.insert(format!("n{}", i), node_type.clone());
                if i + 1 < size {
                    edges.push(rust_logic_graph::core::Edge { from: format!("n{}", i), to: format!("n{}", i+1), rule: None });
                }
            }
            GraphDef::from_node_types(nodes, edges)
        };

        let mut exec = Executor::new();
        for id in graph_def.nodes.keys() {
            let boxed: Box<dyn rust_logic_graph::node::Node> = match node_type {
                NodeType::RuleNode => Box::new(rust_logic_graph::node::RuleNode::new(id, "true")),
                NodeType::DBNode => Box::new(rust_logic_graph::node::DBNode::new(id, format!("SELECT {}", id))),
                NodeType::AINode => Box::new(rust_logic_graph::node::AINode::new(id, format!("Prompt {}", id))),
            };
            exec.register_node(boxed);
        }

        let runs = 10;
        let mut total = 0f64;
        for i in 0..runs {
            let mut graph = Graph::new(graph_def.clone());
            graph.context.set("input", json!(i % 5));
            let start = std::time::Instant::now();
            exec.execute(&mut graph).await?;
            total += start.elapsed().as_secs_f64();
        }
        let avg = total / runs as f64;
        writeln!(w, "{},{:.6}", name, avg)?;
        println!("{} avg: {:.6}", name, avg);
    }

    println!("Wrote results to target/bench_results.csv");
    Ok(())
}
