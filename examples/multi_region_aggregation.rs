//! Multi-region data aggregation example
//! Simulates querying multiple region databases in parallel and aggregating results.

use rust_logic_graph::{Graph, GraphDef, NodeType, NodeConfig, Edge, Executor};
use std::collections::HashMap;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Simulate three regions
    let regions = vec!["us_east", "eu_west", "ap_south"];
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Each region gets a DBNode
    for region in &regions {
        nodes.insert(region.to_string(), NodeType::DBNode);
        edges.push(Edge::new(region.to_string(), "aggregate"));
    }
    // Aggregation node
    nodes.insert("aggregate".to_string(), NodeType::RuleNode);

    let def = GraphDef::from_node_types(nodes, edges);
    let mut graph = Graph::new(def);

    // Node registration is handled by Executor, not Graph.

    let mut exec = Executor::new();
    exec.register_node(Box::new(MockRegionDBNode::new("us_east")));
    exec.register_node(Box::new(MockRegionDBNode::new("eu_west")));
    exec.register_node(Box::new(MockRegionDBNode::new("ap_south")));
    exec.register_node(Box::new(AggregateNode::new("aggregate", regions.clone())));

    exec.execute(&mut graph).await?;
    println!("Aggregated context: {:#?}", graph.context.data);
    Ok(())
}

struct MockRegionDBNode {
    id: String,
}

impl MockRegionDBNode {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait::async_trait]
impl rust_logic_graph::node::Node for MockRegionDBNode {
    fn id(&self) -> &str { &self.id }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        // Simulate region data
        let data = serde_json::json!({
            "region": self.id,
            "sales": match self.id.as_str() {
                "us_east" => 1200,
                "eu_west" => 900,
                "ap_south" => 700,
                _ => 0,
            },
            "customers": match self.id.as_str() {
                "us_east" => 300,
                "eu_west" => 220,
                "ap_south" => 180,
                _ => 0,
            }
        });
        ctx.data.insert(format!("{}_result", self.id), data.clone());
        Ok(data)
    }
}

struct AggregateNode {
    id: String,
    regions: Vec<&'static str>,
}

impl AggregateNode {
    fn new(id: &str, regions: Vec<&'static str>) -> Self {
        Self { id: id.to_string(), regions }
    }
}

#[async_trait::async_trait]
impl rust_logic_graph::node::Node for AggregateNode {
    fn id(&self) -> &str { &self.id }
    fn node_type(&self) -> NodeType { NodeType::RuleNode }
    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        // Aggregate results from all regions
        let mut total_sales = 0;
        let mut total_customers = 0;
        for region in &self.regions {
            if let Some(val) = ctx.data.get(&format!("{}_result", region)) {
                if let Some(sales) = val.get("sales").and_then(|v| v.as_i64()) {
                    total_sales += sales;
                }
                if let Some(customers) = val.get("customers").and_then(|v| v.as_i64()) {
                    total_customers += customers;
                }
            }
        }
        let agg = serde_json::json!({
            "total_sales": total_sales,
            "total_customers": total_customers,
        });
        ctx.data.insert(format!("{}_result", self.id), agg.clone());
        Ok(agg)
    }
}
