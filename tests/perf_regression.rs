use rust_logic_graph::{Executor, Graph, GraphDef, NodeType};
use rust_logic_graph::bench_helpers::ExpensiveComputeNode;
use serde_json::json;

#[tokio::test]
#[ignore]
async fn perf_regression_check() {
    // This is a long-running performance regression test. Run with:
    // cargo test -- --ignored --nocapture
    let graph_def = GraphDef {
        nodes: vec![("compute".to_string(), NodeType::RuleNode)].into_iter().collect(),
        edges: vec![],
    };

    let mut executor = Executor::new();
    executor.register_node(Box::new(ExpensiveComputeNode::new("compute", 20)));

    let runs = 1000;
    let mut total = 0f64;
    for i in 0..runs {
        let mut graph = Graph::new(graph_def.clone());
        graph.context.set("input", json!(i % 10)).unwrap();
        let start = std::time::Instant::now();
        executor.execute(&mut graph).await.unwrap();
        let dur = start.elapsed().as_secs_f64();
        total += dur;
    }

    let avg = total / runs as f64;
    println!("Average execution time over {} runs: {:.6}s", runs, avg);

    // A naive regression assertion (tunable)
    assert!(avg < 0.1, "Average execution time exceeded threshold");
}
