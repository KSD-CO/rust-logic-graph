use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use purchasing_models::{HealthResponse, InventoryData, InventoryResponse};
use sqlx::{MySql, Pool, Row};
use std::env;
use tower_http::trace::TraceLayer;
use tonic::{transport::Server, Request, Response, Status};

// gRPC proto definitions
pub mod inventory {
    tonic::include_proto!("inventory");
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<MySql>,
}

// gRPC service implementation
use inventory::inventory_service_server::{InventoryService, InventoryServiceServer};
use inventory::{HealthRequest, HealthResponse as GrpcHealthResponse, LevelsRequest, LevelsResponse};

struct InventoryServiceImpl {
    db_pool: Pool<MySql>,
}

#[tonic::async_trait]
impl InventoryService for InventoryServiceImpl {
    async fn get_levels(
        &self,
        request: Request<LevelsRequest>,
    ) -> Result<Response<LevelsResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching inventory levels for product: {}", product_id);

        let query = format!(
            "SELECT product_id, available_qty, reserved_qty
             FROM inventory_levels
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

        let response = LevelsResponse {
            product_id: row.get("product_id"),
            available_qty: row.get("available_qty"),
            reserved_qty: row.get("reserved_qty"),
            on_order_qty: 0, // Default value
        };

        tracing::info!("[gRPC] Inventory data retrieved: {:?}", response);

        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        Ok(Response::new(GrpcHealthResponse {
            status: "healthy".to_string(),
            service: "inventory-service".to_string(),
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
    let inventory_db = env::var("INVENTORY_DB").unwrap_or_else(|_| "inventory_db".to_string());

    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, inventory_db
    );

    tracing::info!("Connecting to database: {}", inventory_db);
    let db_pool = sqlx::MySqlPool::connect(&db_url).await?;

    // Clone pool for gRPC server
    let grpc_pool = db_pool.clone();

    // Start gRPC server on port 50052
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50052".to_string());
    let grpc_addr = format!("[::1]:{}", grpc_port).parse()?;

    let inventory_grpc_service = InventoryServiceImpl { db_pool: grpc_pool };

    tokio::spawn(async move {
        tracing::info!("[gRPC] Inventory Service listening on {}", grpc_addr);
        Server::builder()
            .add_service(InventoryServiceServer::new(inventory_grpc_service))
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Start REST server on port 8082
    let state = AppState { db_pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/inventory/levels/:product_id", get(get_inventory_levels))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let rest_port = env::var("PORT").unwrap_or_else(|_| "8082".to_string());
    let rest_addr = format!("0.0.0.0:{}", rest_port);

    tracing::info!("[REST] Inventory Service listening on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(&rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "inventory-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_inventory_levels(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<InventoryResponse>, StatusCode> {
    tracing::info!("[REST] Fetching inventory levels for product: {}", product_id);

    let query = format!(
        "SELECT product_id, warehouse_id, current_qty, reserved_qty, available_qty
         FROM inventory_levels
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

    let data = InventoryData {
        product_id: row.get("product_id"),
        warehouse_id: row.get("warehouse_id"),
        current_qty: row.get("current_qty"),
        reserved_qty: row.get("reserved_qty"),
        available_qty: row.get("available_qty"),
    };

    tracing::info!("[REST] Inventory data retrieved: {:?}", data);

    Ok(Json(InventoryResponse { data }))
}
