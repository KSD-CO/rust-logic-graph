use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// Benchmark rust-logic-graph
mod rlg {
    use rust_logic_graph::{Executor, Graph, GraphDef, NodeType};
    use rust_logic_graph::node::RuleNode;
    use serde_json::json;

    pub async fn execute_linear_chain(size: usize, iterations: usize) {
        let mut nodes = std::collections::HashMap::new();
        let mut edges = Vec::new();

        for i in 0..size {
            nodes.insert(format!("node{}", i), NodeType::RuleNode);
            if i + 1 < size {
                edges.push(rust_logic_graph::core::Edge {
                    from: format!("node{}", i),
                    to: format!("node{}", i + 1),
                    rule: None,
                });
            }
        }

        let graph_def = GraphDef::from_node_types(nodes, edges);
        let mut executor = Executor::new();

        for i in 0..size {
            executor.register_node(Box::new(RuleNode::new(&format!("node{}", i), "true")));
        }

        for _ in 0..iterations {
            let mut graph = Graph::new(graph_def.clone());
            graph.context.set("input", json!(42)).unwrap();
            executor.execute(&mut graph).await.unwrap();
        }
    }

    pub async fn execute_parallel_tasks(size: usize, iterations: usize) {
        let mut nodes = std::collections::HashMap::new();

        for i in 0..size {
            nodes.insert(format!("node{}", i), NodeType::RuleNode);
        }

        let graph_def = GraphDef::from_node_types(
            nodes,
            vec![] // All nodes execute in parallel
        );

        let mut executor = Executor::new();
        for i in 0..size {
            executor.register_node(Box::new(RuleNode::new(&format!("node{}", i), "true")));
        }

        for _ in 0..iterations {
            let mut graph = Graph::new(graph_def.clone());
            graph.context.set("input", json!(42)).unwrap();
            executor.execute(&mut graph).await.unwrap();
        }
    }
}

// Benchmark dagrs
mod dagrs_bench {
    use async_trait::async_trait;
    use dagrs::{Action, Content, DefaultNode, EnvVar, Graph, InChannels, Node, NodeTable, OutChannels, Output};
    use std::sync::Arc;

    struct SimpleAction;

    #[async_trait]
    impl Action for SimpleAction {
        async fn run(
            &self,
            in_channels: &mut InChannels,
            out_channels: &mut OutChannels,
            _env: Arc<EnvVar>,
        ) -> Output {
            // Simple computation similar to RuleNode with "true"
            let mut sum = 0usize;

            in_channels
                .map(|content| content.unwrap().into_inner::<usize>().unwrap())
                .await
                .into_iter()
                .for_each(|x| sum += *x);

            out_channels.broadcast(Content::new(sum)).await;
            Output::Out(Some(Content::new(sum)))
        }
    }

    pub fn execute_linear_chain(size: usize, iterations: usize) {
        for _ in 0..iterations {
            let mut node_table = NodeTable::default();
            let mut graph = Graph::new();
            let mut node_ids = Vec::new();

            // Create nodes
            for i in 0..size {
                let node = DefaultNode::with_action(
                    format!("node{}", i),
                    SimpleAction,
                    &mut node_table,
                );
                node_ids.push(node.id());
                graph.add_node(node);
            }

            // Chain them together
            for i in 0..size - 1 {
                graph.add_edge(node_ids[i], vec![node_ids[i + 1]]);
            }

            let env = EnvVar::new(node_table);
            graph.set_env(env);
            graph.start().unwrap();
        }
    }

    pub fn execute_parallel_tasks(size: usize, iterations: usize) {
        for _ in 0..iterations {
            let mut node_table = NodeTable::default();
            let mut graph = Graph::new();

            // Create tasks without dependencies (parallel execution)
            for i in 0..size {
                let node = DefaultNode::with_action(
                    format!("node{}", i),
                    SimpleAction,
                    &mut node_table,
                );
                graph.add_node(node);
            }

            let env = EnvVar::new(node_table);
            graph.set_env(env);
            graph.start().unwrap();
        }
    }
}

fn bench_linear_chain(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let sizes = vec![5, 10, 20];
    let iterations = 10;

    let mut group = c.benchmark_group("linear_chain_comparison");
    group.measurement_time(Duration::from_secs(10));

    for &size in &sizes {
        group.throughput(Throughput::Elements((size * iterations) as u64));

        group.bench_with_input(
            BenchmarkId::new("rust-logic-graph", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    rt.block_on(async {
                        rlg::execute_linear_chain(s, iterations).await;
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dagrs", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    dagrs_bench::execute_linear_chain(s, iterations);
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_tasks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let sizes = vec![5, 10, 20];
    let iterations = 10;

    let mut group = c.benchmark_group("parallel_tasks_comparison");
    group.measurement_time(Duration::from_secs(10));

    for &size in &sizes {
        group.throughput(Throughput::Elements((size * iterations) as u64));

        group.bench_with_input(
            BenchmarkId::new("rust-logic-graph", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    rt.block_on(async {
                        rlg::execute_parallel_tasks(s, iterations).await;
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dagrs", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    dagrs_bench::execute_parallel_tasks(s, iterations);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(comparison_benches, bench_linear_chain, bench_parallel_tasks);
criterion_main!(comparison_benches);
