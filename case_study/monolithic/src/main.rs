mod models;
mod config;
mod services;
mod handlers;
mod utils;
mod graph_config;
mod graph_executor;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use config::AppConfig;
use graph_executor::PurchasingGraphExecutor;
use services::{InventoryService, OmsService, RuleEngineService, SupplierService, UomService};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utils::create_pool;

#[derive(Clone)]
struct AppState {
    executor: Arc<PurchasingGraphExecutor>,
}

#[derive(serde::Deserialize)]
struct PurchasingFlowRequest {
    product_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    // Try multiple locations to find .env
    let env_paths = [
        ".env",                           // Running from monolithic/
    ];
    
    let mut loaded = false;
    for path in &env_paths {
        if let Ok(_) = dotenvy::from_path(path) {
            eprintln!("✅ Loaded .env from: {}", path);
            loaded = true;
            break;
        }
    }
    
    if !loaded {
        eprintln!("⚠️  Warning: Could not find .env file in any expected location");
        eprintln!("   Tried: {:?}", env_paths);
        eprintln!("   Falling back to environment variables or defaults");
    }
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    tracing::info!("🚀 Purchasing Flow - Monolithic (Clean Architecture)");
    tracing::info!("====================================================");

    // Load configuration
    let config = AppConfig::default();
    tracing::info!("✅ Configuration loaded");
    tracing::info!("   DB Host: {}", config.db.host);
    tracing::info!("   DB User: {}", config.db.user);
    tracing::info!("   DB Port: {}", config.db.port);

    // Create database connection pools
    tracing::info!("🔌 Connecting to databases...");
    let oms_pool = create_pool(&config.db.connection_string(&config.oms_db_name)).await?;
    let inventory_pool = create_pool(&config.db.connection_string(&config.inventory_db_name)).await?;
    let supplier_pool = create_pool(&config.db.connection_string(&config.supplier_db_name)).await?;
    let uom_pool = create_pool(&config.db.connection_string(&config.uom_db_name)).await?;
    tracing::info!("✅ All database pools created");

    // Initialize services
    let oms_service = OmsService::new(oms_pool);
    let inventory_service = InventoryService::new(inventory_pool);
    let supplier_service = SupplierService::new(supplier_pool);
    let uom_service = UomService::new(uom_pool);
    
    // Try multiple paths for GRL rules file
    let grl_paths = [
        "../microservices/services/rule-engine-service/rules/purchasing_rules.grl",  // From monolithic/
        "case_study/microservices/services/rule-engine-service/rules/purchasing_rules.grl",  // From workspace root
        "microservices/services/rule-engine-service/rules/purchasing_rules.grl",  // From case_study/
    ];
    
    let mut grl_path = None;
    for path in &grl_paths {
        if std::path::Path::new(path).exists() {
            grl_path = Some(*path);
            break;
        }
    }
    
    let grl_file = grl_path.ok_or_else(|| {
        anyhow::anyhow!("Could not find GRL rules file. Tried: {:?}", grl_paths)
    })?;
    
    let rule_engine = RuleEngineService::new(grl_file)?;

    // Create graph executor (replaces PurchasingFlowHandler)
    let executor = Arc::new(PurchasingGraphExecutor::new(
        oms_service,
        inventory_service,
        supplier_service,
        uom_service,
        rule_engine,
    ));

    let state = AppState { executor };

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/purchasing/flow", post(handle_purchasing_flow))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("🚀 Monolithic server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_purchasing_flow(
    State(state): State<AppState>,
    Json(req): Json<PurchasingFlowRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("📦 Processing purchasing flow for product: {}", req.product_id);

    match state.executor.execute(&req.product_id).await {
        Ok(Some(po)) => {
            tracing::info!("✅ Purchase order created for product: {}", po.product_id);
            Ok(Json(serde_json::json!({
                "status": "success",
                "message": "Purchase order created successfully",
                "purchase_order": po
            })))
        }
        Ok(None) => {
            tracing::info!("ℹ️ No purchase order needed for product: {}", req.product_id);
            Ok(Json(serde_json::json!({
                "status": "no_order",
                "message": "No purchase order needed - sufficient inventory"
            })))
        }
        Err(e) => {
            tracing::error!("❌ Error processing purchasing flow: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error: {}", e),
            ))
        }
    }
}
