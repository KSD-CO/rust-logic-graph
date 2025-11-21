// Organized modules
mod config;
mod models;
mod nodes;
mod executors;
mod services;
mod utils;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use config::AppConfig;
use executors::PurchasingGraphExecutor;
use nodes::DatabasePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    executor: Arc<Mutex<PurchasingGraphExecutor>>,
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
    tracing::info!("   OMS: {}:{}/{}", config.oms_db.host, config.oms_db.port, config.oms_db.database);
    tracing::info!("   Inventory: {}:{}/{}", config.inventory_db.host, config.inventory_db.port, config.inventory_db.database);
    tracing::info!("   Supplier: {}:{}/{}", config.supplier_db.host, config.supplier_db.port, config.supplier_db.database);
    tracing::info!("   UOM: {}:{}/{}", config.uom_db.host, config.uom_db.port, config.uom_db.database);

    // Create database connection pools
    tracing::info!("🔌 Connecting to databases...");
    
    // Create pools for all databases (each may be on different server)
    let oms_pool = utils::database::create_postgres_pool(&config.oms_db.connection_string()).await?;
    tracing::info!("✅ OMS database pool created: {}", config.oms_db.database);
    
    let inventory_pool = utils::database::create_postgres_pool(&config.inventory_db.connection_string()).await?;
    tracing::info!("✅ Inventory database pool created: {}", config.inventory_db.database);
    
    let supplier_pool = utils::database::create_postgres_pool(&config.supplier_db.connection_string()).await?;
    tracing::info!("✅ Supplier database pool created: {}", config.supplier_db.database);
    
    let uom_pool = utils::database::create_postgres_pool(&config.uom_db.connection_string()).await?;
    tracing::info!("✅ UOM database pool created: {}", config.uom_db.database);

    // Create graph executor with multiple pools
    let mut executor = PurchasingGraphExecutor::from_postgres(oms_pool.clone());
    executor.add_pool("oms_db".to_string(), DatabasePool::from_postgres(oms_pool));
    executor.add_pool("inventory_db".to_string(), DatabasePool::from_postgres(inventory_pool));
    executor.add_pool("supplier_db".to_string(), DatabasePool::from_postgres(supplier_pool));
    executor.add_pool("uom_db".to_string(), DatabasePool::from_postgres(uom_pool));
    
    let executor = Arc::new(Mutex::new(executor));

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

    match state.executor.lock().await.execute(&req.product_id).await {
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
