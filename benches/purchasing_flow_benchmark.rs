use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_logic_graph::{Graph, GraphDef, Executor, NodeType};
use serde_json::json;
use std::collections::HashMap;

// Benchmark the mock version (no database)
fn benchmark_mock_purchasing_flow(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("purchasing_flow_mock", |b| {
        b.to_async(&rt).iter(|| async {
            // Build graph definition
            let mut nodes = HashMap::new();
            let mut edges = Vec::new();

            // Data collection nodes
            nodes.insert("oms_history".to_string(), NodeType::DBNode);
            nodes.insert("inventory_levels".to_string(), NodeType::DBNode);
            nodes.insert("supplier_info".to_string(), NodeType::DBNode);
            nodes.insert("uom_conversion".to_string(), NodeType::DBNode);
            nodes.insert("rule_engine".to_string(), NodeType::RuleNode);
            nodes.insert("calc_order_qty".to_string(), NodeType::RuleNode);
            nodes.insert("create_po".to_string(), NodeType::RuleNode);
            nodes.insert("send_po".to_string(), NodeType::RuleNode);

            // Edges
            edges.push(rust_logic_graph::core::Edge {
                from: "oms_history".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "inventory_levels".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "supplier_info".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "uom_conversion".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "rule_engine".to_string(),
                to: "calc_order_qty".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "calc_order_qty".to_string(),
                to: "create_po".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "create_po".to_string(),
                to: "send_po".to_string(),
                rule: None,
            });

            let graph_def = GraphDef::from_node_types(nodes, edges);

            // Build executor and register nodes
            let mut exec = Executor::new();

            // Register mock DBNodes
            for id in ["oms_history", "inventory_levels", "supplier_info", "uom_conversion"] {
                exec.register_node(Box::new(rust_logic_graph::node::DBNode::new(
                    id,
                    format!("SELECT * FROM {}", id),
                )));
            }

            exec.register_node(Box::new(rust_logic_graph::node::RuleNode::new(
                "rule_engine",
                "true",
            )));

            // Mock calculation node
            struct CalcOrderQty;
            #[async_trait::async_trait]
            impl rust_logic_graph::node::Node for CalcOrderQty {
                fn id(&self) -> &str {
                    "calc_order_qty"
                }
                fn node_type(&self) -> NodeType {
                    NodeType::RuleNode
                }
                async fn run(
                    &self,
                    ctx: &mut rust_logic_graph::core::Context,
                ) -> rust_logic_graph::rule::RuleResult {
                    let demand = ctx
                        .data
                        .get("oms_history")
                        .cloned()
                        .unwrap_or(json!({"avg": 10}));
                    let stock = ctx
                        .data
                        .get("inventory_levels")
                        .cloned()
                        .unwrap_or(json!({"qty": 3}));
                    let moq = ctx
                        .data
                        .get("supplier_info")
                        .and_then(|v| v.get("moq"))
                        .and_then(|m| m.as_i64())
                        .unwrap_or(1);
                    let avg = demand.get("avg").and_then(|v| v.as_i64()).unwrap_or(10);
                    let stock_qty = stock.get("qty").and_then(|v| v.as_i64()).unwrap_or(0);
                    let order = ((avg - stock_qty).max(0) as i64 / moq as i64) * moq as i64;
                    let res = json!({"order_qty": order});
                    ctx.data.insert("calc_order_qty_result".to_string(), res.clone());
                    Ok(res)
                }
            }

            exec.register_node(Box::new(CalcOrderQty));

            // Mock CreatePO node
            struct CreatePO;
            #[async_trait::async_trait]
            impl rust_logic_graph::node::Node for CreatePO {
                fn id(&self) -> &str {
                    "create_po"
                }
                fn node_type(&self) -> NodeType {
                    NodeType::RuleNode
                }
                async fn run(
                    &self,
                    ctx: &mut rust_logic_graph::core::Context,
                ) -> rust_logic_graph::rule::RuleResult {
                    let qty = ctx
                        .data
                        .get("calc_order_qty_result")
                        .and_then(|v| v.get("order_qty"))
                        .and_then(|q| q.as_i64())
                        .unwrap_or(0);
                    let po = json!({"po_id": "PO-12345", "qty": qty});
                    ctx.data.insert("po".to_string(), po.clone());
                    Ok(po)
                }
            }

            exec.register_node(Box::new(CreatePO));

            // Mock SendPO node
            struct SendPO;
            #[async_trait::async_trait]
            impl rust_logic_graph::node::Node for SendPO {
                fn id(&self) -> &str {
                    "send_po"
                }
                fn node_type(&self) -> NodeType {
                    NodeType::RuleNode
                }
                async fn run(
                    &self,
                    ctx: &mut rust_logic_graph::core::Context,
                ) -> rust_logic_graph::rule::RuleResult {
                    let po = ctx.data.get("po").cloned().unwrap_or(json!({}));
                    let mut sent = po.clone();
                    if let Some(obj) = sent.as_object_mut() {
                        obj.insert("sent".to_string(), json!(true));
                    }
                    ctx.data.insert("po_sent".to_string(), sent.clone());
                    Ok(sent)
                }
            }

            exec.register_node(Box::new(SendPO));

            // Execute
            let mut graph = Graph::new(graph_def);
            graph.context.set("input", json!({"run_id": 1})).unwrap();
            exec.execute(&mut graph).await.unwrap();

            black_box(graph.context.data.get("po_sent").cloned())
        });
    });
}

// Benchmark with different numbers of products
fn benchmark_batch_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_processing");

    for num_products in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_products),
            num_products,
            |b, &num_products| {
                b.to_async(&rt).iter(|| async move {
                    // Simulate processing multiple products
                    let mut results = Vec::new();
                    for i in 0..num_products {
                        // Simple mock processing
                        let demand = 15.5;
                        let stock = 20;
                        let moq = 20;
                        let lead_time = 7;

                        let demand_during_lead_time = demand * lead_time as f64;
                        let shortage = (demand_during_lead_time - stock as f64).max(0.0);
                        let order_qty = ((shortage / moq as f64).ceil() as i64) * moq;

                        results.push(json!({
                            "product_id": format!("PROD-{:03}", i),
                            "order_qty": order_qty
                        }));
                    }
                    black_box(results)
                });
            },
        );
    }
    group.finish();
}

// Benchmark graph construction overhead
fn benchmark_graph_construction(c: &mut Criterion) {
    c.bench_function("graph_construction", |b| {
        b.iter(|| {
            let mut nodes = HashMap::new();
            let mut edges = Vec::new();

            nodes.insert("oms_history".to_string(), NodeType::DBNode);
            nodes.insert("inventory_levels".to_string(), NodeType::DBNode);
            nodes.insert("supplier_info".to_string(), NodeType::DBNode);
            nodes.insert("uom_conversion".to_string(), NodeType::DBNode);
            nodes.insert("rule_engine".to_string(), NodeType::RuleNode);
            nodes.insert("calc_order_qty".to_string(), NodeType::RuleNode);
            nodes.insert("create_po".to_string(), NodeType::RuleNode);
            nodes.insert("send_po".to_string(), NodeType::RuleNode);

            edges.push(rust_logic_graph::core::Edge {
                from: "oms_history".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "inventory_levels".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "supplier_info".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "uom_conversion".to_string(),
                to: "rule_engine".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "rule_engine".to_string(),
                to: "calc_order_qty".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "calc_order_qty".to_string(),
                to: "create_po".to_string(),
                rule: None,
            });
            edges.push(rust_logic_graph::core::Edge {
                from: "create_po".to_string(),
                to: "send_po".to_string(),
                rule: None,
            });

            let graph_def = GraphDef::from_node_types(nodes, edges);
            black_box(Graph::new(graph_def))
        });
    });
}

criterion_group!(
    benches,
    benchmark_mock_purchasing_flow,
    benchmark_batch_processing,
    benchmark_graph_construction
);
criterion_main!(benches);
