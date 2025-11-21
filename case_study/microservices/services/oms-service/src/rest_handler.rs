use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, OmsHistoryData, OmsHistoryResponse};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::repository::{OmsRepository, PgOmsRepository};

/// Create REST router
pub fn create_router(
    repository: Arc<PgOmsRepository>,
) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/oms/history/:product_id", get(get_oms_history))
        .layer(TraceLayer::new_for_http())
        .with_state(repository)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "oms-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_oms_history(
    State(repository): State<Arc<PgOmsRepository>>,
    Path(product_id): Path<String>,
) -> Result<Json<OmsHistoryResponse>, StatusCode> {
    tracing::info!("[REST] Fetching OMS history for product: {}", product_id);

    let history = repository
        .get_history(&product_id)
        .await
        .map_err(|e| {
            tracing::error!("[REST] Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let data = OmsHistoryData {
        product_id: history.product_id,
        avg_daily_demand: history.avg_daily_demand,
        trend: history.trend,
    };

    tracing::info!("[REST] OMS data retrieved: {:?}", data);
    Ok(Json(OmsHistoryResponse { data }))
}
