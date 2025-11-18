use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, SupplierData, SupplierResponse};
use sqlx::{MySql, Pool, Row};
use std::env;
use tower_http::trace::TraceLayer;
use tonic::{transport::Server, Request, Response, Status};

// gRPC proto definitions
pub mod supplier {
    tonic::include_proto!("supplier");
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<MySql>,
}

// gRPC service implementation
use supplier::supplier_service_server::{SupplierService, SupplierServiceServer};
use supplier::{HealthRequest, HealthResponse as GrpcHealthResponse, InfoRequest, InfoResponse};

struct SupplierServiceImpl {
    db_pool: Pool<MySql>,
}

#[tonic::async_trait]
impl SupplierService for SupplierServiceImpl {
    async fn get_info(
        &self,
        request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching supplier info for product: {}", product_id);

        let query = format!(
            "SELECT supplier_id, product_id, moq, lead_time_days, CAST(unit_price AS DOUBLE) as unit_price
             FROM supplier_info
             WHERE product_id = '{}' AND is_active = TRUE",
            product_id
        );

        let row = sqlx::query(&query)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("[gRPC] Database error: {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        let response = InfoResponse {
            product_id: row.get("product_id"),
            moq: row.get("moq"),
            lead_time_days: row.get("lead_time_days"),
            unit_price: row.get("unit_price"),
            supplier_name: row.get("supplier_id"), // Map supplier_id to supplier_name
        };

        tracing::info!("[gRPC] Supplier data retrieved: {:?}", response);

        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        Ok(Response::new(GrpcHealthResponse {
            status: "healthy".to_string(),
            service: "supplier-service".to_string(),
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
    let supplier_db = env::var("SUPPLIER_DB").unwrap_or_else(|_| "supplier_db".to_string());

    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, supplier_db
    );

    tracing::info!("Connecting to database: {}", supplier_db);
    let db_pool = sqlx::MySqlPool::connect(&db_url).await?;

    // Clone pool for gRPC server
    let grpc_pool = db_pool.clone();

    // Start gRPC server on port 50053
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50053".to_string());
    let grpc_addr = format!("[::1]:{}", grpc_port).parse()?;

    let supplier_grpc_service = SupplierServiceImpl { db_pool: grpc_pool };

    tokio::spawn(async move {
        tracing::info!("[gRPC] Supplier Service listening on {}", grpc_addr);
        Server::builder()
            .add_service(SupplierServiceServer::new(supplier_grpc_service))
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Start REST server on port 8083
    let state = AppState { db_pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/supplier/info/:product_id", get(get_supplier_info))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let rest_port = env::var("PORT").unwrap_or_else(|_| "8083".to_string());
    let rest_addr = format!("0.0.0.0:{}", rest_port);

    tracing::info!("[REST] Supplier Service listening on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(&rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "supplier-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_supplier_info(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<SupplierResponse>, StatusCode> {
    tracing::info!("[REST] Fetching supplier info for product: {}", product_id);

    let query = format!(
        "SELECT supplier_id, product_id, moq, lead_time_days, CAST(unit_price AS DOUBLE) as unit_price, is_active
         FROM supplier_info
         WHERE product_id = '{}' AND is_active = TRUE",
        product_id
    );

    let row = sqlx::query(&query)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("[REST] Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let data = SupplierData {
        supplier_id: row.get("supplier_id"),
        product_id: row.get("product_id"),
        moq: row.get("moq"),
        lead_time_days: row.get("lead_time_days"),
        unit_price: row.get("unit_price"),
        is_active: row.get("is_active"),
    };

    tracing::info!("[REST] Supplier data retrieved: {:?}", data);

    Ok(Json(SupplierResponse { data }))
}
