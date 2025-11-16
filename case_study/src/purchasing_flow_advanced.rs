use rust_logic_graph::{Graph, GraphDef, Executor, NodeType};
use serde_json::json;
use sqlx::{MySql, Pool, Row};
use async_trait::async_trait;
use std::time::Instant;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::env;

// Advanced purchasing flow with monitoring, metrics, and performance tracking

// Database configuration is loaded from .env file
fn get_db_config() -> (String, String, String, u16) {
    let db_user = env::var("DB_USER").expect("DB_USER must be set in .env file");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set in .env file");
    let db_host = env::var("DB_HOST").expect("DB_HOST must be set in .env file");
    let db_port = env::var("DB_PORT")
        .expect("DB_PORT must be set in .env file")
        .parse::<u16>()
        .expect("DB_PORT must be a valid port number");

    (db_user, db_password, db_host, db_port)
}

fn get_db_names() -> (String, String, String, String) {
    let oms_db = env::var("OMS_DB").unwrap_or_else(|_| "oms_db".to_string());
    let inventory_db = env::var("INVENTORY_DB").unwrap_or_else(|_| "inventory_db".to_string());
    let supplier_db = env::var("SUPPLIER_DB").unwrap_or_else(|_| "supplier_db".to_string());
    let uom_db = env::var("UOM_DB").unwrap_or_else(|_| "uom_db".to_string());

    (oms_db, inventory_db, supplier_db, uom_db)
}

// Metrics collector
#[derive(Clone)]
struct Metrics {
    total_queries: Arc<AtomicU64>,
    total_query_time_ms: Arc<AtomicU64>,
    total_calculations: Arc<AtomicU64>,
    total_pos_created: Arc<AtomicU64>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            total_queries: Arc::new(AtomicU64::new(0)),
            total_query_time_ms: Arc::new(AtomicU64::new(0)),
            total_calculations: Arc::new(AtomicU64::new(0)),
            total_pos_created: Arc::new(AtomicU64::new(0)),
        }
    }

    fn record_query(&self, duration_ms: u64) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.total_query_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    fn record_calculation(&self) {
        self.total_calculations.fetch_add(1, Ordering::Relaxed);
    }

    fn record_po_created(&self) {
        self.total_pos_created.fetch_add(1, Ordering::Relaxed);
    }

    fn print_summary(&self) {
        let queries = self.total_queries.load(Ordering::Relaxed);
        let query_time = self.total_query_time_ms.load(Ordering::Relaxed);
        let calculations = self.total_calculations.load(Ordering::Relaxed);
        let pos = self.total_pos_created.load(Ordering::Relaxed);

        println!("\n╔════════════════════════════════════════════╗");
        println!("║         Performance Metrics Summary         ║");
        println!("╠════════════════════════════════════════════╣");
        println!("║ Total Database Queries:        {:>11} ║", queries);
        println!("║ Total Query Time (ms):         {:>11} ║", query_time);
        if queries > 0 {
            println!("║ Avg Query Time (ms):           {:>11.2} ║", query_time as f64 / queries as f64);
        }
        println!("║ Total Calculations:            {:>11} ║", calculations);
        println!("║ Total POs Created:             {:>11} ║", pos);
        println!("╚════════════════════════════════════════════╝");
    }
}

// Enhanced MySQL DBNode with metrics
struct MySQLDBNodeWithMetrics {
    id: String,
    query: String,
    pool: Pool<MySql>,
    db_name: String,
    metrics: Metrics,
}

impl MySQLDBNodeWithMetrics {
    async fn new(id: &str, query: &str, db_name: &str, metrics: Metrics) -> anyhow::Result<Self> {
        let (db_user, db_password, db_host, db_port) = get_db_config();
        let db_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let pool = sqlx::MySqlPool::connect(&db_url).await?;

        Ok(Self {
            id: id.to_string(),
            query: query.to_string(),
            pool,
            db_name: db_name.to_string(),
            metrics,
        })
    }
}

#[async_trait]
impl rust_logic_graph::node::Node for MySQLDBNodeWithMetrics {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        let start = Instant::now();

        println!("  ⚡ [{}] Querying {} ...", self.id, self.db_name);

        let rows = sqlx::query(&self.query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(
                format!("Database query error on {}: {}", self.db_name, e)
            ))?;

        let duration = start.elapsed();
        self.metrics.record_query(duration.as_millis() as u64);

        println!("  ✓ [{}] Completed in {:.2}ms", self.id, duration.as_secs_f64() * 1000.0);

        // Convert rows to JSON
        let mut results = Vec::new();

        for row in rows {
            let mut obj = serde_json::Map::new();

            match self.id.as_str() {
                "oms_history" => {
                    obj.insert("product_id".to_string(), json!(row.get::<String, _>("product_id")));
                    obj.insert("avg_daily_demand".to_string(), json!(row.get::<f64, _>("avg_daily_demand")));
                    obj.insert("trend".to_string(), json!(row.get::<String, _>("trend")));
                }
                "inventory_levels" => {
                    obj.insert("product_id".to_string(), json!(row.get::<String, _>("product_id")));
                    obj.insert("warehouse_id".to_string(), json!(row.get::<String, _>("warehouse_id")));
                    obj.insert("current_qty".to_string(), json!(row.get::<i32, _>("current_qty")));
                    obj.insert("reserved_qty".to_string(), json!(row.get::<i32, _>("reserved_qty")));
                    obj.insert("available_qty".to_string(), json!(row.get::<i32, _>("available_qty")));
                }
                "supplier_info" => {
                    obj.insert("supplier_id".to_string(), json!(row.get::<String, _>("supplier_id")));
                    obj.insert("product_id".to_string(), json!(row.get::<String, _>("product_id")));
                    obj.insert("moq".to_string(), json!(row.get::<i32, _>("moq")));
                    obj.insert("lead_time_days".to_string(), json!(row.get::<i32, _>("lead_time_days")));
                    obj.insert("unit_price".to_string(), json!(row.get::<f64, _>("unit_price")));
                }
                "uom_conversion" => {
                    obj.insert("product_id".to_string(), json!(row.get::<String, _>("product_id")));
                    obj.insert("from_uom".to_string(), json!(row.get::<String, _>("from_uom")));
                    obj.insert("to_uom".to_string(), json!(row.get::<String, _>("to_uom")));
                    obj.insert("conversion_factor".to_string(), json!(row.get::<f64, _>("conversion_factor")));
                }
                _ => {}
            }

            results.push(serde_json::Value::Object(obj));
        }

        let result = if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            json!(results)
        };

        ctx.data.insert(self.id.clone(), result.clone());
        Ok(result)
    }
}

// Enhanced CalcOrderQty with metrics
struct CalcOrderQtyWithMetrics {
    metrics: Metrics,
}

#[async_trait]
impl rust_logic_graph::node::Node for CalcOrderQtyWithMetrics {
    fn id(&self) -> &str {
        "calc_order_qty"
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        let start = Instant::now();
        println!("  ⚡ [calc_order_qty] Running calculation logic...");

        let oms_data = ctx.data.get("oms_history").cloned().unwrap_or(json!({}));
        let inventory_data = ctx.data.get("inventory_levels").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_info").cloned().unwrap_or(json!({}));

        let avg_demand = oms_data.get("avg_daily_demand").and_then(|v| v.as_f64()).unwrap_or(10.0);
        let available_qty = inventory_data.get("available_qty").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
        let moq = supplier_data.get("moq").and_then(|v| v.as_i64()).unwrap_or(1);
        let lead_time = supplier_data.get("lead_time_days").and_then(|v| v.as_i64()).unwrap_or(7) as f64;

        let demand_during_lead_time = avg_demand * lead_time;
        let shortage = (demand_during_lead_time - available_qty).max(0.0);
        let order_qty = ((shortage / moq as f64).ceil() as i64) * moq;

        self.metrics.record_calculation();

        let result = json!({
            "order_qty": order_qty,
            "avg_demand": avg_demand,
            "available_qty": available_qty,
            "demand_during_lead_time": demand_during_lead_time,
            "shortage": shortage,
            "moq": moq,
            "lead_time_days": lead_time
        });

        let duration = start.elapsed();
        println!("  ✓ [calc_order_qty] Calculated order_qty={} in {:.2}ms",
            order_qty, duration.as_secs_f64() * 1000.0);

        ctx.data.insert("calc_order_qty_result".to_string(), result.clone());
        Ok(result)
    }
}

// Enhanced CreatePO with metrics
struct CreatePOWithMetrics {
    metrics: Metrics,
}

#[async_trait]
impl rust_logic_graph::node::Node for CreatePOWithMetrics {
    fn id(&self) -> &str {
        "create_po"
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        let start = Instant::now();
        println!("  ⚡ [create_po] Generating purchase order...");

        let calc_result = ctx.data.get("calc_order_qty_result").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_info").cloned().unwrap_or(json!({}));
        let oms_data = ctx.data.get("oms_history").cloned().unwrap_or(json!({}));

        let qty = calc_result.get("order_qty").and_then(|v| v.as_i64()).unwrap_or(0);
        let unit_price = supplier_data.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let total_amount = qty as f64 * unit_price;

        let po = json!({
            "po_id": format!("PO-{}", chrono::Utc::now().timestamp()),
            "product_id": oms_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("UNKNOWN"),
            "supplier_id": supplier_data.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("UNKNOWN"),
            "qty": qty,
            "unit_price": unit_price,
            "total_amount": total_amount,
            "status": "draft",
            "created_at": chrono::Utc::now().to_rfc3339()
        });

        self.metrics.record_po_created();

        let duration = start.elapsed();
        println!("  ✓ [create_po] PO created (total: ${:.2}) in {:.2}ms",
            total_amount, duration.as_secs_f64() * 1000.0);

        ctx.data.insert("po".to_string(), po.clone());
        Ok(po)
    }
}

// SendPO node
struct SendPO;

#[async_trait]
impl rust_logic_graph::node::Node for SendPO {
    fn id(&self) -> &str {
        "send_po"
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        println!("  ⚡ [send_po] Transmitting to supplier...");

        let po = ctx.data.get("po").cloned().unwrap_or(json!({}));

        let mut sent = po.clone();
        if let Some(obj) = sent.as_object_mut() {
            obj.insert("status".to_string(), json!("sent"));
            obj.insert("sent_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        }

        println!("  ✓ [send_po] PO transmitted successfully");

        ctx.data.insert("po_sent".to_string(), sent.clone());
        Ok(sent)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let (_db_user, _db_password, db_host, db_port) = get_db_config();
    let (oms_db, inventory_db, supplier_db, uom_db) = get_db_names();

    let total_start = Instant::now();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   Advanced Purchasing Flow - Production Monitoring Mode   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("📊 System Configuration:");
    println!("  • Database Host: {}:{}", db_host, db_port);
    println!("  • OMS Database: {}", oms_db);
    println!("  • Inventory Database: {}", inventory_db);
    println!("  • Supplier Database: {}", supplier_db);
    println!("  • UOM Database: {}", uom_db);
    println!();

    // Initialize metrics
    let metrics = Metrics::new();

    // Build graph definition
    let mut nodes = std::collections::HashMap::new();
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

    let graph_def = GraphDef { nodes, edges };

    // Build executor
    let mut exec = Executor::new();
    let product_id = "PROD-001";

    println!("🔗 Establishing Database Connections...\n");

    exec.register_node(Box::new(
        MySQLDBNodeWithMetrics::new(
            "oms_history",
            &format!("SELECT product_id, CAST(avg_daily_demand AS DOUBLE) as avg_daily_demand, trend FROM oms_history WHERE product_id = '{}'", product_id),
            &oms_db,
            metrics.clone()
        ).await?
    ));

    exec.register_node(Box::new(
        MySQLDBNodeWithMetrics::new(
            "inventory_levels",
            &format!("SELECT product_id, warehouse_id, current_qty, reserved_qty, available_qty FROM inventory_levels WHERE product_id = '{}'", product_id),
            &inventory_db,
            metrics.clone()
        ).await?
    ));

    exec.register_node(Box::new(
        MySQLDBNodeWithMetrics::new(
            "supplier_info",
            &format!("SELECT supplier_id, product_id, moq, lead_time_days, CAST(unit_price AS DOUBLE) as unit_price FROM supplier_info WHERE product_id = '{}' AND is_active = TRUE", product_id),
            &supplier_db,
            metrics.clone()
        ).await?
    ));

    exec.register_node(Box::new(
        MySQLDBNodeWithMetrics::new(
            "uom_conversion",
            &format!("SELECT product_id, from_uom, to_uom, CAST(conversion_factor AS DOUBLE) as conversion_factor FROM uom_conversion WHERE product_id = '{}'", product_id),
            &uom_db,
            metrics.clone()
        ).await?
    ));

    exec.register_node(Box::new(rust_logic_graph::node::RuleNode::new("rule_engine", "true")));
    exec.register_node(Box::new(CalcOrderQtyWithMetrics { metrics: metrics.clone() }));
    exec.register_node(Box::new(CreatePOWithMetrics { metrics: metrics.clone() }));
    exec.register_node(Box::new(SendPO));

    println!("✅ All connections established\n");
    println!("🚀 Starting Graph Execution...\n");

    let mut graph = Graph::new(graph_def);
    graph.context.set("input", json!({"product_id": product_id}))?;

    let exec_start = Instant::now();
    exec.execute(&mut graph).await?;
    let exec_duration = exec_start.elapsed();

    println!("\n✅ Graph Execution Complete in {:.2}ms\n", exec_duration.as_secs_f64() * 1000.0);

    // Display results
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    Final Purchase Order                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    if let Some(po) = graph.context.data.get("po_sent") {
        println!("{}\n", serde_json::to_string_pretty(po)?);
    }

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                   Calculation Details                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    if let Some(calc) = graph.context.data.get("calc_order_qty_result") {
        println!("{}\n", serde_json::to_string_pretty(calc)?);
    }

    // Print metrics summary
    metrics.print_summary();

    let total_duration = total_start.elapsed();
    println!("\n⏱️  Total Execution Time: {:.2}ms", total_duration.as_secs_f64() * 1000.0);
    println!("✨ Purchasing flow completed successfully!\n");

    Ok(())
}
