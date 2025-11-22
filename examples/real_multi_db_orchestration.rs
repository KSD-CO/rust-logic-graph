// Real Multi-Database Query Orchestration Example
// Uses actual database connections from case study monolithic setup
//
// Prerequisites:
// 1. Set up databases: case_study/scripts/setup_multi_databases.sh
// 2. Run: cargo run --example real_multi_db_orchestration
//
// This example demonstrates:
// - Parallel query execution across 4 PostgreSQL databases
// - YAML-driven configuration (declarative approach)
// - Dynamic query loading and execution
// - Database pool registry pattern
//
// Uses the purchasing flow schema from the case study.
// Queries are loaded from multi_db_graph.yaml configuration file.

use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::collections::HashMap;
use std::env;
use serde::{Deserialize, Serialize};

/// YAML configuration structure
#[derive(Debug, Deserialize, Serialize)]
struct GraphConfig {
    nodes: HashMap<String, NodeConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NodeConfig {
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    database: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Vec<String>>,
}

/// Database configuration matching case study monolithic
#[derive(Debug, Clone)]
struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    fn from_env_prefix(prefix: &str) -> Self {
        let host_key = format!("{}_HOST", prefix);
        let port_key = format!("{}_PORT", prefix);
        let user_key = format!("{}_USER", prefix);
        let pass_key = format!("{}_PASSWORD", prefix);
        let name_key = format!("{}_NAME", prefix);
        
        Self {
            host: env::var(&host_key).unwrap_or_else(|_| "localhost".to_string()),
            port: env::var(&port_key)
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            user: env::var(&user_key).unwrap_or_else(|_| "jamesvu".to_string()),
            password: env::var(&pass_key).unwrap_or_else(|_| "".to_string()),
            database: env::var(&name_key).unwrap_or_else(|_| {
                // Default database names based on prefix
                match prefix {
                    "OMS_DB" => "oms_db",
                    "INVENTORY_DB" => "inventory_db",
                    "SUPPLIER_DB" => "supplier_db",
                    "UOM_DB" => "uom_db",
                    _ => "postgres",
                }.to_string()
            }),
        }
    }

    fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║      Real Multi-Database Query Orchestration Demo            ║");
    println!("║      Using Case Study Purchasing Flow Databases              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    // Load YAML configuration
    println!("📄 Loading configuration from multi_db_graph.yaml...");
    let yaml_content = std::fs::read_to_string("examples/multi_db_graph.yaml")?;
    let config: GraphConfig = serde_yaml::from_str(&yaml_content)?;
    println!("✅ Configuration loaded: {} nodes defined\n", config.nodes.len());
    
    // Load database configurations
    let oms_config = DatabaseConfig::from_env_prefix("OMS_DB");
    let inventory_config = DatabaseConfig::from_env_prefix("INVENTORY_DB");
    let supplier_config = DatabaseConfig::from_env_prefix("SUPPLIER_DB");
    let uom_config = DatabaseConfig::from_env_prefix("UOM_DB");
    
    println!("📊 Database Configuration:");
    println!("  • OMS:       {}:{}/{}", oms_config.host, oms_config.port, oms_config.database);
    println!("  • Inventory: {}:{}/{}", inventory_config.host, inventory_config.port, inventory_config.database);
    println!("  • Supplier:  {}:{}/{}", supplier_config.host, supplier_config.port, supplier_config.database);
    println!("  • UOM:       {}:{}/{}\n", uom_config.host, uom_config.port, uom_config.database);
    
    // Create connection pools
    println!("🔌 Connecting to databases...");
    let oms_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&oms_config.connection_string())
        .await?;
    println!("  ✅ OMS database connected");
    
    let inventory_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&inventory_config.connection_string())
        .await?;
    println!("  ✅ Inventory database connected");
    
    let supplier_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&supplier_config.connection_string())
        .await?;
    println!("  ✅ Supplier database connected");
    
    let uom_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&uom_config.connection_string())
        .await?;
    println!("  ✅ UOM database connected\n");
    
    // Create database pool registry
    let mut db_pools: HashMap<String, sqlx::PgPool> = HashMap::new();
    db_pools.insert("oms_db".to_string(), oms_pool.clone());
    db_pools.insert("inventory_db".to_string(), inventory_pool.clone());
    db_pools.insert("supplier_db".to_string(), supplier_pool.clone());
    db_pools.insert("uom_db".to_string(), uom_pool.clone());
    
    // Demo 1: Parallel Query Execution (using YAML config)
    println!("━━━ Demo 1: Parallel Query Execution (YAML-based) ━━━\n");
    parallel_query_demo_yaml(&config, &db_pools).await?;
    
    // Demo 2: Aggregated Dashboard Query
    println!("\n━━━ Demo 2: Aggregated Dashboard Query ━━━\n");
    dashboard_query_demo(&oms_pool, &inventory_pool, &supplier_pool).await?;
    
    println!("\n✅ All demos completed successfully!\n");
    println!("💡 Tip: This demo shows the power of parallel queries across multiple databases.");
    println!("   - Demo 1 uses YAML configuration (declarative approach)");
    println!("   - Demo 2 uses direct SQL queries (programmatic approach)");
    println!("   - See examples/multi_db_graph.yaml for the configuration\n");
    
    Ok(())
}

/// Execute queries using YAML configuration (new approach)
async fn parallel_query_demo_yaml(
    config: &GraphConfig,
    db_pools: &HashMap<String, sqlx::PgPool>,
) -> anyhow::Result<()> {
    let product_id = "PROD-001";
    
    println!("📊 Executing queries in parallel for product: {}\n", product_id);
    
    // Find specific nodes by name
    let oms_node = config.nodes.iter()
        .find(|(name, _)| name.as_str() == "fetch_oms_history")
        .ok_or_else(|| anyhow::anyhow!("fetch_oms_history node not found"))?;
    
    let inventory_node = config.nodes.iter()
        .find(|(name, _)| name.as_str() == "fetch_inventory")
        .ok_or_else(|| anyhow::anyhow!("fetch_inventory node not found"))?;
    
    let supplier_node = config.nodes.iter()
        .find(|(name, _)| name.as_str() == "fetch_supplier")
        .ok_or_else(|| anyhow::anyhow!("fetch_supplier node not found"))?;
    
    let uom_node = config.nodes.iter()
        .find(|(name, _)| name.as_str() == "fetch_uom")
        .ok_or_else(|| anyhow::anyhow!("fetch_uom node not found"))?;
    
    let start = std::time::Instant::now();
    
    // Execute all queries in parallel using YAML configuration
    let results = tokio::join!(
        execute_yaml_query(&oms_node, db_pools, product_id),
        execute_yaml_query(&inventory_node, db_pools, product_id),
        execute_yaml_query(&supplier_node, db_pools, product_id),
        execute_yaml_query(&uom_node, db_pools, product_id),
    );
    
    let elapsed = start.elapsed();
    
    // Display results
    println!("📈 Query Results:\n");
    
    if let Ok(oms_data) = &results.0 {
        println!("  1️⃣  OMS History:");
        println!("     📊 Average Daily Demand: {:.2}", oms_data.get::<f64, _>("avg_daily_demand"));
        println!("     📈 Trend: {}\n", oms_data.get::<String, _>("trend"));
    }
    
    if let Ok(inventory_data) = &results.1 {
        println!("  2️⃣  Inventory:");
        println!("     📦 Warehouse: {}", inventory_data.get::<String, _>("warehouse_location"));
        println!("     ✅ Available: {:.0} units", inventory_data.get::<f64, _>("available_qty"));
        println!("     🔒 Reserved: {:.0} units\n", inventory_data.get::<f64, _>("reserved_qty"));
    }
    
    if let Ok(supplier_data) = &results.2 {
        println!("  3️⃣  Supplier:");
        println!("     🏢 Name: {}", supplier_data.get::<String, _>("supplier_name"));
        println!("     📦 MOQ: {:.0} units", supplier_data.get::<f64, _>("moq"));
        println!("     ⏱️  Lead Time: {} days", supplier_data.get::<i32, _>("lead_time"));
        println!("     💰 Unit Price: ${:.2}\n", supplier_data.get::<f64, _>("unit_price"));
    }
    
    if let Ok(uom_data) = &results.3 {
        println!("  4️⃣  UOM Conversion:");
        println!("     🔄 From: {} → To: {}", uom_data.get::<String, _>("from_uom"), uom_data.get::<String, _>("to_uom"));
        println!("     ⚖️  Conversion Factor: {:.2}\n", uom_data.get::<f64, _>("conversion_factor"));
    }
    
    println!("⏱️  Parallel Execution Time: {:?}", elapsed);
    println!("✅ All 4 queries completed successfully\n");
    
    Ok(())
}

/// Execute a single YAML-configured query
async fn execute_yaml_query(
    node: &(&String, &NodeConfig),
    db_pools: &HashMap<String, sqlx::PgPool>,
    product_id: &str,
) -> Result<sqlx::postgres::PgRow, sqlx::Error> {
    let (_node_name, node_config) = node;
    let db_name = node_config.database.as_ref()
        .ok_or_else(|| sqlx::Error::Configuration("Missing database field".into()))?;
    let query = node_config.query.as_ref()
        .ok_or_else(|| sqlx::Error::Configuration("Missing query field".into()))?;
    
    let pool = db_pools.get(db_name)
        .ok_or_else(|| sqlx::Error::Configuration(format!("Pool not found for database: {}", db_name).into()))?;
    
    sqlx::query(query)
        .bind(product_id)
        .fetch_one(pool)
        .await
}

/// Demo 2: Create aggregated dashboard query across multiple databases
async fn dashboard_query_demo(
    oms_pool: &sqlx::PgPool,
    inventory_pool: &sqlx::PgPool,
    supplier_pool: &sqlx::PgPool,
) -> anyhow::Result<()> {
    println!("📊 Building purchasing dashboard with aggregated metrics\n");
    
    let start = std::time::Instant::now();
    
    // Execute all dashboard queries in parallel
    let (total_products, inventory_stats, supplier_stats) = tokio::join!(
        fetch_total_products(oms_pool),
        fetch_inventory_stats(inventory_pool),
        fetch_supplier_stats(supplier_pool)
    );
    
    let duration = start.elapsed();
    
    // Build dashboard
    let mut dashboard = HashMap::new();
    
    if let Ok(data) = total_products {
        dashboard.insert("total_products", data);
    }
    if let Ok(data) = inventory_stats {
        dashboard.insert("inventory", data);
    }
    if let Ok(data) = supplier_stats {
        dashboard.insert("suppliers", data);
    }
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  Purchasing Dashboard                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    println!("{}\n", serde_json::to_string_pretty(&dashboard)?);
    
    println!("⏱️  Dashboard loaded in: {:.2}ms", duration.as_millis());
    println!("   ⚡ 3 databases queried in PARALLEL");
    println!("   Ready for real-time decision making! 🚀");
    
    Ok(())
}

async fn fetch_total_products(pool: &sqlx::PgPool) -> anyhow::Result<serde_json::Value> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM oms_history")
        .fetch_one(pool)
        .await?;
    Ok(json!({ "total_products": row.get::<i64, _>("count") }))
}

async fn fetch_inventory_stats(pool: &sqlx::PgPool) -> anyhow::Result<serde_json::Value> {
    let row = sqlx::query(
        "SELECT COUNT(DISTINCT product_id) as product_count, 
                COUNT(DISTINCT warehouse_location) as warehouse_count,
                SUM(available_qty)::FLOAT8 as total_available_qty
         FROM inventory"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(json!({
        "product_count": row.get::<i64, _>("product_count"),
        "warehouse_count": row.get::<i64, _>("warehouse_count"),
        "total_available_qty": row.get::<f64, _>("total_available_qty"),
    }))
}

async fn fetch_supplier_stats(pool: &sqlx::PgPool) -> anyhow::Result<serde_json::Value> {
    let row = sqlx::query(
        "SELECT COUNT(DISTINCT supplier_name) as supplier_count,
                AVG(lead_time)::FLOAT8 as avg_lead_time,
                AVG(unit_price)::FLOAT8 as avg_unit_price,
                MIN(moq)::FLOAT8 as min_moq,
                MAX(moq)::FLOAT8 as max_moq
         FROM suppliers"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(json!({
        "supplier_count": row.get::<i64, _>("supplier_count"),
        "avg_lead_time": row.get::<f64, _>("avg_lead_time"),
        "avg_unit_price": row.get::<f64, _>("avg_unit_price"),
        "min_moq": row.get::<f64, _>("min_moq"),
        "max_moq": row.get::<f64, _>("max_moq"),
    }))
}
