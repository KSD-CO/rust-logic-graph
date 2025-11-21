use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, SupplierData, SupplierResponse};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::repository::{SupplierRepository, PgSupplierRepository};

/// Create REST router
pub fn create_router(
    repository: Arc<PgSupplierRepository>,
) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/supplier/info/:product_id", get(get_supplier_info))
        .layer(TraceLayer::new_for_http())
        .with_state(repository)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "supplier-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_supplier_info(
    State(repository): State<Arc<PgSupplierRepository>>,
    Path(product_id): Path<String>,
) -> Result<Json<SupplierResponse>, StatusCode> {
    tracing::info!("[REST] Fetching supplier info for product: {}", product_id);

    let info = repository
        .get_info(&product_id)
        .await
        .map_err(|e| {
            tracing::error!("[REST] Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let data = SupplierData {
        supplier_id: info.supplier_id,
        product_id: info.product_id,
        moq: info.moq,
        lead_time_days: info.lead_time_days,
        unit_price: info.unit_price,
        is_active: true, // Always true from query filter
    };

    tracing::info!("[REST] Supplier data retrieved: {:?}", data);
    Ok(Json(SupplierResponse { data }))
}
