mod models;
mod config;
mod services;
mod handlers;
mod utils;

use anyhow::Result;
use config::AppConfig;
use handlers::PurchasingFlowHandler;
use services::{InventoryService, OmsService, RuleEngineService, SupplierService, UomService};
use utils::{create_pool, Metrics, Timer};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Purchasing Flow - Clean Architecture");
    println!("========================================\n");

    let total_timer = Timer::new();

    // Load configuration
    let config = AppConfig::default();
    println!("✅ Configuration loaded");

    // Create database connection pools
    println!("🔌 Connecting to databases...");
    let oms_pool = create_pool(&config.db.connection_string(&config.oms_db_name)).await?;
    let inventory_pool = create_pool(&config.db.connection_string(&config.inventory_db_name)).await?;
    let supplier_pool = create_pool(&config.db.connection_string(&config.supplier_db_name)).await?;
    let uom_pool = create_pool(&config.db.connection_string(&config.uom_db_name)).await?;
    println!("✅ All database pools created\n");

    // Initialize services
    let oms_service = OmsService::new(oms_pool);
    let inventory_service = InventoryService::new(inventory_pool);
    let supplier_service = SupplierService::new(supplier_pool);
    let uom_service = UomService::new(uom_pool);
    let rule_engine = RuleEngineService::new("case_study/rules/purchasing_rules.grl")?;

    // Create purchasing flow handler
    let handler = PurchasingFlowHandler::new(
        oms_service,
        inventory_service,
        supplier_service,
        uom_service,
        rule_engine,
    );

    // Execute purchasing flow for a test product
    let product_id = "PROD-001";
    match handler.execute(product_id).await? {
        Some(po) => {
            println!("\n✅ Purchase Order successfully created!");
        }
        None => {
            println!("\n❌ No purchase order created.");
        }
    }

    let total_time = total_timer.elapsed_ms();
    println!("\n⏱️  Total execution time: {} ms", total_time);

    Ok(())
}
