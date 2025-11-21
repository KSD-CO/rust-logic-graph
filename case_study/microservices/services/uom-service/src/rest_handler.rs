use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, UomConversionData, UomConversionResponse};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::repository::{UomRepository, PgUomRepository};

/// Create REST router
pub fn create_router(
    repository: Arc<PgUomRepository>,
) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/uom/conversions/:product_id", get(get_uom_conversions))
        .layer(TraceLayer::new_for_http())
        .with_state(repository)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "uom-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_uom_conversions(
    State(repository): State<Arc<PgUomRepository>>,
    Path(product_id): Path<String>,
) -> Result<Json<UomConversionResponse>, StatusCode> {
    tracing::info!("[REST] Fetching UOM conversions for product: {}", product_id);

    let conversion = repository
        .get_conversions(&product_id)
        .await
        .map_err(|e| {
            tracing::error!("[REST] Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let data = UomConversionData {
        product_id: conversion.product_id,
        from_uom: conversion.from_uom,
        to_uom: conversion.to_uom,
        conversion_factor: conversion.conversion_factor,
    };

    tracing::info!("[REST] UOM data retrieved: {:?}", data);
    Ok(Json(UomConversionResponse { data }))
}
