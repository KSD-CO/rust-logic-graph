use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, UomConversionData, UomConversionResponse};
use sqlx::{MySql, Pool, Row};
use std::env;
use tower_http::trace::TraceLayer;
use tonic::{transport::Server, Request, Response, Status};

// gRPC proto definitions
pub mod uom {
    tonic::include_proto!("uom");
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<MySql>,
}

// gRPC service implementation
use uom::uom_service_server::{UomService, UomServiceServer};
use uom::{ConversionRequest, ConversionResponse, HealthRequest, HealthResponse as GrpcHealthResponse};

struct UomServiceImpl {
    db_pool: Pool<MySql>,
}

#[tonic::async_trait]
impl UomService for UomServiceImpl {
    async fn get_conversion(
        &self,
        request: Request<ConversionRequest>,
    ) -> Result<Response<ConversionResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching UOM conversion for product: {}", product_id);

        let query = format!(
            "SELECT product_id, from_uom as base_unit, to_uom, CAST(conversion_factor AS DOUBLE) as conversion_factor
             FROM uom_conversion
             WHERE product_id = '{}'",
            product_id
        );

        let row = sqlx::query(&query)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("[gRPC] Database error: {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        let conversion_factor: f64 = row.get("conversion_factor");

        let response = ConversionResponse {
            product_id: row.get("product_id"),
            base_unit: row.get("base_unit"),
            case_qty: conversion_factor as i32, // Convert to i32 for case_qty
            pallet_qty: (conversion_factor * 10.0) as i32, // Example: pallet is 10x case
        };

        tracing::info!("[gRPC] UOM conversion data retrieved: {:?}", response);

        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        Ok(Response::new(GrpcHealthResponse {
            status: "healthy".to_string(),
            service: "uom-service".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let db_user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_password = env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let uom_db = env::var("UOM_DB").unwrap_or_else(|_| "uom_db".to_string());

    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, uom_db
    );

    tracing::info!("Connecting to database: {}", uom_db);
    let db_pool = sqlx::MySqlPool::connect(&db_url).await?;

    // Clone pool for gRPC server
    let grpc_pool = db_pool.clone();

    // Start gRPC server on port 50054
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50054".to_string());
    let grpc_addr = format!("[::1]:{}", grpc_port).parse()?;

    let uom_grpc_service = UomServiceImpl { db_pool: grpc_pool };

    tokio::spawn(async move {
        tracing::info!("[gRPC] UOM Service listening on {}", grpc_addr);
        Server::builder()
            .add_service(UomServiceServer::new(uom_grpc_service))
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Start REST server on port 8084
    let state = AppState { db_pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/uom/conversion/:product_id", get(get_uom_conversion))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let rest_port = env::var("PORT").unwrap_or_else(|_| "8084".to_string());
    let rest_addr = format!("0.0.0.0:{}", rest_port);

    tracing::info!("[REST] UOM Service listening on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(&rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "uom-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_uom_conversion(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<UomConversionResponse>, StatusCode> {
    tracing::info!("[REST] Fetching UOM conversion for product: {}", product_id);

    let query = format!(
        "SELECT product_id, from_uom, to_uom, CAST(conversion_factor AS DOUBLE) as conversion_factor
         FROM uom_conversion
         WHERE product_id = '{}'",
        product_id
    );

    let row = sqlx::query(&query)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("[REST] Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let data = UomConversionData {
        product_id: row.get("product_id"),
        from_uom: row.get("from_uom"),
        to_uom: row.get("to_uom"),
        conversion_factor: row.get("conversion_factor"),
    };

    tracing::info!("[REST] UOM conversion data retrieved: {:?}", data);

    Ok(Json(UomConversionResponse { data }))
}
