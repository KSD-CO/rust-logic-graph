use rust_logic_graph::distributed::InMemoryStore;
use rust_logic_graph::fault_tolerance::{
    CircuitBreaker, CircuitConfig, FailoverManager, ServiceEndpoint,
};
use rust_logic_graph::{Executor, Graph, GraphDef, NodeConfig, NodeType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Mock endpoints
    let endpoints = vec![
        ServiceEndpoint {
            name: "primary".into(),
            url: "http://127.0.0.1:3001/health".into(),
        },
        ServiceEndpoint {
            name: "backup".into(),
            url: "http://127.0.0.1:3002/health".into(),
        },
    ];

    let store = Arc::new(InMemoryStore::new());
    let cb = Arc::new(CircuitBreaker::new(
        "payment-svc",
        Some(store.clone()),
        Some(CircuitConfig {
            failure_threshold: 2,
            recovery_timeout: std::time::Duration::from_secs(3),
            probe_interval: std::time::Duration::from_secs(1),
        }),
    ));

    // Simple FailoverManager
    let fm = FailoverManager::new(endpoints, cb.clone());

    // Executor with graceful degradation fallback
    let mut exec = Executor::new();
    exec.set_fallback_handler(|node_id, ctx| {
        println!("Fallback invoked for node: {}", node_id);
        ctx.data.insert(
            format!("{}_result", node_id),
            serde_json::Value::String("fallback".into()),
        );
        Some(serde_json::Value::String("fallback".into()))
    });

    // Simple graph with a node that simulates failure
    let mut nodes = std::collections::HashMap::new();
    nodes.insert("call_service".into(), NodeType::GrpcNode);
    let def = GraphDef::from_node_types(nodes, vec![]);
    let mut graph = Graph::new(def);

    // Simulate executing the node; in real life we'd call the selected endpoint
    let selected = fm.select().await;
    println!("Selected endpoint: {:?}", selected.map(|e| e.name));

    // Simulate node failure by invoking a node that returns Err in run(); here we'll just call fallback directly
    exec.register_node(Box::new(rust_logic_graph::node::RuleNode::new(
        "call_service",
        "false",
    )));
    exec.execute(&mut graph).await?;

    println!("Graph context after execution: {:?}", graph.context.data);
    Ok(())
}
