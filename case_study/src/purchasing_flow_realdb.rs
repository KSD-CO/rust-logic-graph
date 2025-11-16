use rust_logic_graph::{Graph, GraphDef, Executor, NodeType};
use serde_json::json;
use sqlx::{MySql, Pool, Row};
use async_trait::async_trait;
use std::env;

// This example models the purchasing flow diagram with REAL MySQL database.
// External systems are represented by custom DB nodes that query actual tables.
// RuleNodes implement business rules. The flow: collect data -> rule engine ->
// calculate order qty -> create PO -> send PO.

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

// Custom DBNode that queries real MySQL database with its own connection
struct MySQLDBNode {
    id: String,
    query: String,
    pool: Pool<MySql>,
    db_name: String,
}

impl MySQLDBNode {
    async fn new(id: &str, query: &str, db_name: &str) -> anyhow::Result<Self> {
        // Create dedicated connection pool for this node
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
        })
    }
}

#[async_trait]
impl rust_logic_graph::node::Node for MySQLDBNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        println!("[{}] Database: {} | Executing query: {}", self.id, self.db_name, self.query);

        let rows = sqlx::query(&self.query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(
                format!("Database query error on {}: {}", self.db_name, e)
            ))?;

        // Convert rows to JSON
        let mut results = Vec::new();

        for row in rows {
            let mut obj = serde_json::Map::new();

            // Parse based on which table we're querying
            match self.id.as_str() {
                "oms_history" => {
                    obj.insert("product_id".to_string(), json!(row.get::<String, _>("product_id")));
                    // avg_daily_demand is CAST to DOUBLE in SQL query
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
                    // unit_price is CAST to DOUBLE in SQL query
                    obj.insert("unit_price".to_string(), json!(row.get::<f64, _>("unit_price")));
                }
                "uom_conversion" => {
                    obj.insert("product_id".to_string(), json!(row.get::<String, _>("product_id")));
                    obj.insert("from_uom".to_string(), json!(row.get::<String, _>("from_uom")));
                    obj.insert("to_uom".to_string(), json!(row.get::<String, _>("to_uom")));
                    // conversion_factor is CAST to DOUBLE in SQL query
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

        println!("[{}] Result: {}", self.id, result);
        ctx.data.insert(self.id.clone(), result.clone());
        Ok(result)
    }
}

// Calculate order quantity based on real data from database
struct CalcOrderQty;

#[async_trait]
impl rust_logic_graph::node::Node for CalcOrderQty {
    fn id(&self) -> &str {
        "calc_order_qty"
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        println!("[calc_order_qty] Calculating order quantity...");

        // Get real data from context
        let oms_data = ctx.data.get("oms_history").cloned().unwrap_or(json!({}));
        let inventory_data = ctx.data.get("inventory_levels").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_info").cloned().unwrap_or(json!({}));

        // Extract values
        let avg_demand = oms_data.get("avg_daily_demand")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0);

        let available_qty = inventory_data.get("available_qty")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as f64;

        let moq = supplier_data.get("moq")
            .and_then(|v| v.as_i64())
            .unwrap_or(1);

        let lead_time = supplier_data.get("lead_time_days")
            .and_then(|v| v.as_i64())
            .unwrap_or(7) as f64;

        // Business logic:
        // - Calculate demand during lead time
        // - Subtract current available inventory
        // - Round up to MOQ
        let demand_during_lead_time = avg_demand * lead_time;
        let shortage = (demand_during_lead_time - available_qty).max(0.0);
        let order_qty = ((shortage / moq as f64).ceil() as i64) * moq;

        let result = json!({
            "order_qty": order_qty,
            "avg_demand": avg_demand,
            "available_qty": available_qty,
            "demand_during_lead_time": demand_during_lead_time,
            "shortage": shortage,
            "moq": moq,
            "lead_time_days": lead_time
        });

        println!("[calc_order_qty] Result: {}", result);
        ctx.data.insert("calc_order_qty_result".to_string(), result.clone());
        Ok(result)
    }
}

// Create Purchase Order
struct CreatePO;

#[async_trait]
impl rust_logic_graph::node::Node for CreatePO {
    fn id(&self) -> &str {
        "create_po"
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, ctx: &mut rust_logic_graph::core::Context) -> rust_logic_graph::rule::RuleResult {
        println!("[create_po] Creating purchase order...");

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

        println!("[create_po] PO created: {}", po);
        ctx.data.insert("po".to_string(), po.clone());
        Ok(po)
    }
}

// Send Purchase Order
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
        println!("[send_po] Sending purchase order...");

        let po = ctx.data.get("po").cloned().unwrap_or(json!({}));

        // In a real system, this would call supplier API
        let mut sent = po.clone();
        if let Some(obj) = sent.as_object_mut() {
            obj.insert("status".to_string(), json!("sent"));
            obj.insert("sent_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        }

        println!("[send_po] PO sent: {}", sent);
        ctx.data.insert("po_sent".to_string(), sent.clone());
        Ok(sent)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let (oms_db, inventory_db, supplier_db, uom_db) = get_db_names();

    println!("=== Purchasing Flow with Real MySQL Databases ===");
    println!("Each node connects to a separate database:");
    println!("  - OMS Node        -> {}", oms_db);
    println!("  - Inventory Node  -> {}", inventory_db);
    println!("  - Supplier Node   -> {}", supplier_db);
    println!("  - UOM Node        -> {}", uom_db);
    println!();

    // Build graph definition
    let mut nodes = std::collections::HashMap::new();
    let mut edges = Vec::new();

    // Data collection nodes (DBNodes querying real MySQL tables)
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

    let graph_def = GraphDef { nodes, edges };

    // Build executor and register nodes
    let mut exec = Executor::new();

    // Query for PROD-001 for this example
    let product_id = "PROD-001";

    println!("Creating database connections for each node...");

    // Register MySQL DBNodes - each with its own database connection
    println!("  [oms_history] Connecting to {}...", oms_db);
    exec.register_node(Box::new(
        MySQLDBNode::new(
            "oms_history",
            &format!("SELECT product_id, CAST(avg_daily_demand AS DOUBLE) as avg_daily_demand, trend FROM oms_history WHERE product_id = '{}'", product_id),
            &oms_db
        ).await?
    ));

    println!("  [inventory_levels] Connecting to {}...", inventory_db);
    exec.register_node(Box::new(
        MySQLDBNode::new(
            "inventory_levels",
            &format!("SELECT product_id, warehouse_id, current_qty, reserved_qty, available_qty FROM inventory_levels WHERE product_id = '{}'", product_id),
            &inventory_db
        ).await?
    ));

    println!("  [supplier_info] Connecting to {}...", supplier_db);
    exec.register_node(Box::new(
        MySQLDBNode::new(
            "supplier_info",
            &format!("SELECT supplier_id, product_id, moq, lead_time_days, CAST(unit_price AS DOUBLE) as unit_price FROM supplier_info WHERE product_id = '{}' AND is_active = TRUE", product_id),
            &supplier_db
        ).await?
    ));

    println!("  [uom_conversion] Connecting to {}...", uom_db);
    exec.register_node(Box::new(
        MySQLDBNode::new(
            "uom_conversion",
            &format!("SELECT product_id, from_uom, to_uom, CAST(conversion_factor AS DOUBLE) as conversion_factor FROM uom_conversion WHERE product_id = '{}'", product_id),
            &uom_db
        ).await?
    ));

    println!("All database connections established successfully!\n");

    // rule_engine: evaluate rules
    exec.register_node(Box::new(rust_logic_graph::node::RuleNode::new(
        "rule_engine",
        "true",
    )));

    // Register business logic nodes
    exec.register_node(Box::new(CalcOrderQty));
    exec.register_node(Box::new(CreatePO));
    exec.register_node(Box::new(SendPO));

    // Execute the graph
    let mut graph = Graph::new(graph_def);
    graph.context.set("input", json!({"product_id": product_id}))?;

    println!("Starting graph execution...\n");
    exec.execute(&mut graph).await?;

    // Display final results
    println!("\n=== Execution Complete ===\n");
    println!("Final Purchase Order:");
    if let Some(po) = graph.context.data.get("po_sent") {
        println!("{}", serde_json::to_string_pretty(po)?);
    }

    println!("\nCalculation Details:");
    if let Some(calc) = graph.context.data.get("calc_order_qty_result") {
        println!("{}", serde_json::to_string_pretty(calc)?);
    }

    println!("\nAll database connections will be closed automatically.");

    Ok(())
}
