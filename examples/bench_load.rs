use rust_logic_graph::{Executor, Graph, GraphDef, NodeType, DBNode};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting load-test example: launching many concurrent graph executions");

    // Build a simple graph with a DBNode (simulated 100ms delay in DBNode.run)
    let graph_def = GraphDef::from_node_types(
        vec![("dbnode".to_string(), NodeType::DBNode)].into_iter().collect(),
        vec![],
    );

    // We'll create a fresh executor per task to avoid needing Executor::clone

    let concurrency = 50usize;
    let total = 500usize;
    let sem = Arc::new(Semaphore::new(concurrency));

    let mut handles = Vec::with_capacity(total);
    for i in 0..total {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let def = graph_def.clone();

        handles.push(tokio::spawn(async move {
            let _p = permit; // keep permit until the end
            // create executor locally per task
            let mut exec = Executor::new();
            exec.register_node(Box::new(DBNode::new("dbnode", "SELECT 1")));
            let mut graph = Graph::new(def);
            graph.context.set("request_id", json!(i));
            if let Err(e) = exec.execute(&mut graph).await {
                eprintln!("Execution failed: {:?}", e);
            }
        }));
    }

    // Await all spawned tasks (they're already running)
    for h in handles {
        let _ = h.await;
    }

    println!("Load test complete: executed {} graphs with concurrency {}", total, concurrency);
    Ok(())
}
