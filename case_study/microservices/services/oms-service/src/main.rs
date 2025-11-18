use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, OmsHistoryData, OmsHistoryResponse};
use sqlx::{MySql, Pool, Row};
use std::env;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tonic::{transport::Server, Request, Response, Status};

// gRPC proto definitions
pub mod oms {
    tonic::include_proto!("oms");
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<MySql>,
}

// gRPC service implementation
use oms::oms_service_server::{OmsService, OmsServiceServer};
use oms::{HealthRequest, HealthResponse as GrpcHealthResponse, HistoryRequest, HistoryResponse};

struct OmsServiceImpl {
    db_pool: Pool<MySql>,
}

#[tonic::async_trait]
impl OmsService for OmsServiceImpl {
    async fn get_history(
        &self,
        request: Request<HistoryRequest>,
    ) -> Result<Response<HistoryResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching OMS history for product: {}", product_id);

        let query = format!(
            "SELECT product_id, CAST(avg_daily_demand AS DOUBLE) as avg_daily_demand, trend
             FROM oms_history
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

        let response = HistoryResponse {
            product_id: row.get("product_id"),
            avg_daily_demand: row.get("avg_daily_demand"),
            trend: row.get("trend"),
        };

        tracing::info!("[gRPC] OMS data retrieved: {:?}", response);

        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        Ok(Response::new(GrpcHealthResponse {
            status: "healthy".to_string(),
            service: "oms-service".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let db_user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_password = env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let oms_db = env::var("OMS_DB").unwrap_or_else(|_| "oms_db".to_string());

    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, oms_db
    );

    tracing::info!("Connecting to database: {}", oms_db);
    let db_pool = sqlx::MySqlPool::connect(&db_url).await?;

    // Clone pool for gRPC server
    let grpc_pool = db_pool.clone();

    // Start gRPC server on port 50051
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let grpc_addr = format!("[::1]:{}", grpc_port).parse()?;

    let oms_grpc_service = OmsServiceImpl { db_pool: grpc_pool };

    tokio::spawn(async move {
        tracing::info!("[gRPC] OMS Service listening on {}", grpc_addr);
        Server::builder()
            .add_service(OmsServiceServer::new(oms_grpc_service))
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Start REST server on port 8081
    let state = AppState { db_pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/oms/history/:product_id", get(get_oms_history))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let rest_port = env::var("PORT").unwrap_or_else(|_| "8081".to_string());
    let rest_addr = format!("0.0.0.0:{}", rest_port);

    tracing::info!("[REST] OMS Service listening on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(&rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "oms-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_oms_history(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<OmsHistoryResponse>, StatusCode> {
    tracing::info!("[REST] Fetching OMS history for product: {}", product_id);

    let query = format!(
        "SELECT product_id, CAST(avg_daily_demand AS DOUBLE) as avg_daily_demand, trend
         FROM oms_history
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

    let data = OmsHistoryData {
        product_id: row.get("product_id"),
        avg_daily_demand: row.get("avg_daily_demand"),
        trend: row.get("trend"),
    };

    tracing::info!("[REST] OMS data retrieved: {:?}", data);

    Ok(Json(OmsHistoryResponse { data }))
}
