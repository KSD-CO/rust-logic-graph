use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, InventoryData, InventoryResponse};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::repository::{InventoryRepository, PgInventoryRepository};

/// Create REST router
pub fn create_router(
    repository: Arc<PgInventoryRepository>,
) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/inventory/levels/:product_id", get(get_inventory_levels))
        .layer(TraceLayer::new_for_http())
        .with_state(repository)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "inventory-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_inventory_levels(
    State(repository): State<Arc<PgInventoryRepository>>,
    Path(product_id): Path<String>,
) -> Result<Json<InventoryResponse>, StatusCode> {
    tracing::info!("[REST] Fetching inventory levels for product: {}", product_id);

    let levels = repository
        .get_levels(&product_id)
        .await
        .map_err(|e| {
            tracing::error!("[REST] Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let data = InventoryData {
        product_id: levels.product_id,
        warehouse_id: levels.warehouse_id,
        current_qty: levels.available_qty,
        reserved_qty: levels.reserved_qty,
        available_qty: levels.available_qty,
    };

    tracing::info!("[REST] Inventory data retrieved: {:?}", data);
    Ok(Json(InventoryResponse { data }))
}
