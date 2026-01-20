use rust_logic_graph::{Executor, Graph, GraphDef, NodeType};
use serde_json::json;

// This example models the purchasing flow diagram. External systems are
// represented by DB nodes that return mock data. RuleNodes implement
// business rules. The flow: collect data -> rule engine -> calculate
// order qty -> create PO -> send PO.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build graph definition
    let mut nodes = std::collections::HashMap::new();
    let mut edges = Vec::new();

    // Data collection nodes (DBNodes simulating OMS, Inventory, Supplier, UnitConv)
    nodes.insert("oms_history".to_string(), NodeType::DBNode);
    nodes.insert("inventory_levels".to_string(), NodeType::DBNode);
    nodes.insert("supplier_info".to_string(), NodeType::DBNode);
    nodes.insert("uom_conversion".to_string(), NodeType::DBNode);

    // Rule engine node
    nodes.insert("rule_engine".to_string(), NodeType::RuleNode);

    // Calculate order quantity node (RuleNode)
    nodes.insert("calc_order_qty".to_string(), NodeType::RuleNode);

    // Create and send PO nodes
    nodes.insert("create_po".to_string(), NodeType::RuleNode);
    nodes.insert("send_po".to_string(), NodeType::RuleNode);

    // Edges: data collection -> rule_engine
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

    // rule_engine -> calc_order_qty -> create_po -> send_po
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

    // DBNodes return mock data and insert into context
    for id in [
        "oms_history",
        "inventory_levels",
        "supplier_info",
        "uom_conversion",
    ] {
        exec.register_node(Box::new(rust_logic_graph::node::DBNode::new(
            id,
            format!("SELECT * FROM {}", id),
        )));
    }

    // rule_engine: evaluate rules and write flags into context
    exec.register_node(Box::new(rust_logic_graph::node::RuleNode::new(
        "rule_engine",
        "true",
    )));

    // calc_order_qty: mock calculation based on context
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
            // Simple heuristic: order = max(0, average_demand - stock_level) * moq
            let demand = ctx
                .data
                .get("oms_history")
                .cloned()
                .unwrap_or(serde_json::json!({"avg": 10}));
            let stock = ctx
                .data
                .get("inventory_levels")
                .cloned()
                .unwrap_or(serde_json::json!({"qty": 3}));
            let moq = ctx
                .data
                .get("supplier_info")
                .and_then(|v| v.get("moq"))
                .and_then(|m| m.as_i64())
                .unwrap_or(1);
            let avg = demand.get("avg").and_then(|v| v.as_i64()).unwrap_or(10);
            let stock_qty = stock.get("qty").and_then(|v| v.as_i64()).unwrap_or(0);
            let order = ((avg - stock_qty).max(0) as i64 / moq as i64) * moq as i64;
            let res = serde_json::json!({"order_qty": order});
            ctx.data
                .insert("calc_order_qty_result".to_string(), res.clone());
            Ok(res)
        }
    }

    exec.register_node(Box::new(CalcOrderQty));

    // create_po: create a PO object
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
            let po = serde_json::json!({"po_id": "PO-12345", "qty": qty});
            ctx.data.insert("po".to_string(), po.clone());
            Ok(po)
        }
    }

    exec.register_node(Box::new(CreatePO));

    // send_po: simulate sending
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
            let po = ctx.data.get("po").cloned().unwrap_or(serde_json::json!({}));
            // In a real system you'd call an external API; here we just flag sent=true
            let mut sent = po.clone();
            if let Some(obj) = sent.as_object_mut() {
                obj.insert("sent".to_string(), serde_json::json!(true));
            }
            ctx.data.insert("po_sent".to_string(), sent.clone());
            Ok(sent)
        }
    }

    exec.register_node(Box::new(SendPO));

    // Execute the graph once
    let mut graph = Graph::new(graph_def);
    graph.context.set("input", json!({"run_id": 1}));
    exec.execute(&mut graph).await?;

    println!("Final context: {:?}", graph.context.data);
    Ok(())
}
