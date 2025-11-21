use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rust_logic_graph::{
    cache::{CacheConfig, CacheManager},
    Graph, GraphDef, NodeType, Executor, Context,
};
use rust_logic_graph::bench_helpers::ExpensiveComputeNode;
use serde_json::json;
use std::time::Duration;

async fn run_graph_once(executor: &Executor, graph_def: &GraphDef, input: i64) {
    let mut graph = Graph::new(graph_def.clone());
    graph.context.set("input", json!(input));
    executor.execute(&mut graph).await.unwrap();
}

fn bench_executor(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    // Simple graph definition with one RuleNode (simulate expensive node in example)
    let graph_def = GraphDef::from_node_types(
        vec![("compute".to_string(), NodeType::RuleNode)].into_iter().collect(),
        vec![],
    );

    // Executor without cache
    let mut exec_no_cache = Executor::new();
    exec_no_cache.register_node(Box::new(ExpensiveComputeNode::new("compute", 50)));

    // Executor with cache
    let cache_config = CacheConfig {
        max_entries: 1000,
        max_memory_bytes: 10 * 1024 * 1024,
        default_ttl: Some(Duration::from_secs(60)),
        eviction_policy: rust_logic_graph::cache::EvictionPolicy::LRU,
        enable_background_cleanup: false,
    };
    let cache = rt.block_on(CacheManager::new(cache_config)).unwrap();
    let mut exec_with_cache = Executor::with_cache(cache.clone());
    exec_with_cache.register_node(Box::new(ExpensiveComputeNode::new("compute", 50)));

    let mut group = c.benchmark_group("executor_cache_vs_no_cache");
    group.throughput(Throughput::Elements(1));

    for &use_cache in &[false, true] {
        let mut id = if use_cache { "with_cache" } else { "no_cache" };

            group.bench_function(BenchmarkId::new("execute", id), |b| {
            if use_cache {
                b.iter(|| rt.block_on(async { run_graph_once(&exec_with_cache, &graph_def, 10).await }));
            } else {
                b.iter(|| rt.block_on(async { run_graph_once(&exec_no_cache, &graph_def, 10).await }));
            }
        });
    }

    group.finish();
}

criterion_group!(benches, bench_executor);
criterion_main!(benches);
