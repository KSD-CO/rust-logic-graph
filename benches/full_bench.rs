use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use rust_logic_graph::cache::{CacheConfig, CacheManager, EvictionPolicy};
use rust_logic_graph::core::GraphDef;
use rust_logic_graph::{Executor, Graph, NodeType};
use serde_json::json;
use std::time::Duration;

// Helper that creates simple graphs of varying sizes
fn make_graph_def(size: usize, node_type: NodeType) -> GraphDef {
    let mut nodes = std::collections::HashMap::new();
    let mut edges = Vec::new();

    for i in 0..size {
        nodes.insert(format!("node{}", i), node_type.clone());
        // link node i -> i+1 for a chain except last
        if i + 1 < size {
            edges.push(rust_logic_graph::core::Edge {
                from: format!("node{}", i),
                to: format!("node{}", i + 1),
                rule: None,
            });
        }
    }

    GraphDef::from_node_types(nodes, edges)
}

// Bench runner for a given graph and executor
async fn run_n_times(executor: &Executor, graph_def: &GraphDef, n: usize) {
    for i in 0..n {
        let mut graph = Graph::new(graph_def.clone());
        graph.context.set("input", json!(i % 10));
        executor.execute(&mut graph).await.unwrap();
    }
}

fn full_bench(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    // Make variations: node types, sizes, cache on/off
    let node_types = vec![NodeType::RuleNode, NodeType::DBNode, NodeType::AINode];
    let sizes = vec![1usize, 5usize, 20usize];

    // Cache config for 'with cache' executors
    let cache_config = CacheConfig {
        max_entries: 10_000,
        max_memory_bytes: 50 * 1024 * 1024,
        default_ttl: Some(Duration::from_secs(300)),
        eviction_policy: EvictionPolicy::LRU,
        enable_background_cleanup: false,
    };

    for node_type in node_types {
        for &size in &sizes {
            let graph_def = make_graph_def(size, node_type.clone());

            // Executor without cache
            let mut exec_no_cache = Executor::new();
            // Register nodes with appropriate concrete node types
            for node_id in graph_def.nodes.keys() {
                let boxed: Box<dyn rust_logic_graph::node::Node> = match node_type {
                    NodeType::RuleNode => Box::new(rust_logic_graph::node::RuleNode::new(node_id, "true")),
                    NodeType::DBNode => Box::new(rust_logic_graph::node::DBNode::new(node_id, format!("SELECT {}", node_id))),
                    NodeType::AINode => Box::new(rust_logic_graph::node::AINode::new(node_id, format!("Prompt {}", node_id))),
                };
                exec_no_cache.register_node(boxed);
            }

            // Executor with cache
            let cache = rt.block_on(CacheManager::new(cache_config.clone())).unwrap();
            let mut exec_with_cache = Executor::with_cache(cache.clone());
            for node_id in graph_def.nodes.keys() {
                let boxed: Box<dyn rust_logic_graph::node::Node> = match node_type {
                    NodeType::RuleNode => Box::new(rust_logic_graph::node::RuleNode::new(node_id, "true")),
                    NodeType::DBNode => Box::new(rust_logic_graph::node::DBNode::new(node_id, format!("SELECT {}", node_id))),
                    NodeType::AINode => Box::new(rust_logic_graph::node::AINode::new(node_id, format!("Prompt {}", node_id))),
                };
                exec_with_cache.register_node(boxed);
            }

            let mut group = c.benchmark_group(format!("full/{}-nodes-{}", match node_type {
                NodeType::RuleNode => "rule",
                NodeType::DBNode => "db",
                NodeType::AINode => "ai",
            }, size));
            group.sampling_mode(SamplingMode::Flat);

            group.bench_with_input(BenchmarkId::new("no_cache", size), &size, |b, &_s| {
                b.iter(|| rt.block_on(async { run_n_times(&exec_no_cache, &graph_def, 5).await }));
            });

            group.bench_with_input(BenchmarkId::new("with_cache", size), &size, |b, &_s| {
                b.iter(|| rt.block_on(async { run_n_times(&exec_with_cache, &graph_def, 5).await }));
            });

            group.finish();
        }
    }
}

criterion_group!(full_benches, full_bench);
criterion_main!(full_benches);
