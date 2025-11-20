use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use purchasing_models::*;
use reqwest::Client;
use std::env;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

mod graph_executor;
use graph_executor::OrchestratorGraphExecutor;

// gRPC proto definitions (used by graph_executor module)
pub mod oms {
    tonic::include_proto!("oms");
}
pub mod inventory {
    tonic::include_proto!("inventory");
}
pub mod supplier {
    tonic::include_proto!("supplier");
}
pub mod uom {
    tonic::include_proto!("uom");
}
pub mod rule_engine {
    tonic::include_proto!("rule_engine");
}
pub mod po {
    tonic::include_proto!("po");
}

#[derive(Clone)]
struct AppState {
    http_client: Client,
    executor: Arc<OrchestratorGraphExecutor>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let http_client = Client::new();

    // gRPC URLs for inter-service communication (high performance)
    let oms_grpc_url = env::var("OMS_GRPC_URL").unwrap_or_else(|_| "http://localhost:50051".to_string());
    let inventory_grpc_url = env::var("INVENTORY_GRPC_URL").unwrap_or_else(|_| "http://localhost:50052".to_string());
    let supplier_grpc_url = env::var("SUPPLIER_GRPC_URL").unwrap_or_else(|_| "http://localhost:50053".to_string());
    let uom_grpc_url = env::var("UOM_GRPC_URL").unwrap_or_else(|_| "http://localhost:50054".to_string());
    let rule_engine_grpc_url = env::var("RULE_ENGINE_GRPC_URL").unwrap_or_else(|_| "http://localhost:50055".to_string());
    let po_grpc_url = env::var("PO_GRPC_URL").unwrap_or_else(|_| "http://localhost:50056".to_string());

    tracing::info!("Service URLs configured:");
    tracing::info!("  [gRPC] OMS: {}", oms_grpc_url);
    tracing::info!("  [gRPC] Inventory: {}", inventory_grpc_url);
    tracing::info!("  [gRPC] Supplier: {}", supplier_grpc_url);
    tracing::info!("  [gRPC] UOM: {}", uom_grpc_url);
    tracing::info!("  [gRPC] Rule Engine: {}", rule_engine_grpc_url);
    tracing::info!("  [gRPC] PO: {}", po_grpc_url);

    // Create graph executor
    let executor = Arc::new(OrchestratorGraphExecutor::new(
        oms_grpc_url,
        inventory_grpc_url,
        supplier_grpc_url,
        uom_grpc_url,
        rule_engine_grpc_url,
        po_grpc_url,
    ));

    let state = AppState {
        http_client,
        executor,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/purchasing/flow", post(execute_purchasing_flow))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Orchestrator Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "orchestrator-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn execute_purchasing_flow(
    State(state): State<AppState>,
    Json(req): Json<PurchasingFlowRequest>,
) -> Result<Json<PurchasingFlowResponse>, StatusCode> {
    tracing::info!("🎯 Starting purchasing flow via Graph Executor for product: {}", req.product_id);

    match state.executor.execute(&req.product_id).await {
        Ok(response) => {
            tracing::info!("✅ Purchasing flow completed successfully");
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("❌ Purchasing flow failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
