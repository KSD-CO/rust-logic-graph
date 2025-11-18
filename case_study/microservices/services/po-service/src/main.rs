use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use purchasing_models::{CreatePORequest, HealthResponse, POResponse, PurchaseOrder};
use std::env;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tower_http::trace::TraceLayer;
use tonic::{transport::Server, Request, Response, Status};

// Include generated gRPC code
pub mod po {
    tonic::include_proto!("po");
}

use po::{
    po_service_server::{PoService as GrpcPoService, PoServiceServer},
    CreateRequest, CreateResponse, SendRequest, SendResponse,
    PurchaseOrder as GrpcPurchaseOrder, HealthRequest, HealthResponse as GrpcHealthResponse,
};

#[derive(Clone)]
struct AppState {
    po_store: Arc<Mutex<HashMap<String, GrpcPurchaseOrder>>>,
}

// gRPC service implementation
struct PoGrpcService {
    po_store: Arc<Mutex<HashMap<String, GrpcPurchaseOrder>>>,
}

#[tonic::async_trait]
impl GrpcPoService for PoGrpcService {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();

        tracing::info!("gRPC: Creating PO for product: {}, qty: {}", req.product_id, req.qty);

        let po = GrpcPurchaseOrder {
            po_id: format!("PO-{}", chrono::Utc::now().timestamp()),
            product_id: req.product_id,
            supplier_id: req.supplier_id,
            qty: req.qty,
            unit_price: req.unit_price,
            total_amount: req.total_amount,
            status: "draft".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            sent_at: String::new(),
        };

        // Store PO in memory
        self.po_store.lock().unwrap().insert(po.po_id.clone(), po.clone());

        tracing::info!("gRPC: PO created: {}, total: ${:.2}", po.po_id, po.total_amount);

        Ok(Response::new(CreateResponse {
            po: Some(po),
        }))
    }

    async fn send(
        &self,
        request: Request<SendRequest>,
    ) -> Result<Response<SendResponse>, Status> {
        let req = request.into_inner();

        tracing::info!("gRPC: Sending PO: {}", req.po_id);

        // Retrieve PO from store
        let mut po = self.po_store.lock().unwrap().get(&req.po_id).cloned().ok_or_else(|| {
            tracing::error!("PO not found: {}", req.po_id);
            Status::not_found(format!("PO {} not found", req.po_id))
        })?;

        // Update status and sent_at
        po.status = "sent".to_string();
        po.sent_at = chrono::Utc::now().to_rfc3339();

        // Update store
        self.po_store.lock().unwrap().insert(req.po_id.clone(), po.clone());

        tracing::info!("gRPC: PO sent successfully: {}", req.po_id);

        Ok(Response::new(SendResponse {
            success: true,
            message: format!("PO {} sent successfully", req.po_id),
            po: Some(po),
        }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        Ok(Response::new(GrpcHealthResponse {
            status: "healthy".to_string(),
            service: "po-service".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let po_store = Arc::new(Mutex::new(HashMap::new()));

    let state = AppState {
        po_store: po_store.clone(),
    };

    // Setup HTTP server
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/po/create", post(create_po))
        .route("/po/send", post(send_po))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let http_port = env::var("PORT").unwrap_or_else(|_| "8086".to_string());
    let http_addr = format!("0.0.0.0:{}", http_port);

    // Setup gRPC server
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50056".to_string());
    let grpc_addr = format!("0.0.0.0:{}", grpc_port).parse()?;

    let grpc_service = PoGrpcService {
        po_store: po_store.clone(),
    };

    tracing::info!("PO Service listening on HTTP {} and gRPC {}", http_addr, grpc_addr);

    // Run both servers concurrently
    let http_server = async {
        let listener = tokio::net::TcpListener::bind(&http_addr).await?;
        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    };

    let grpc_server = async {
        Server::builder()
            .add_service(PoServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await?;
        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(http_server, grpc_server)?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "po-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn create_po(
    State(_state): State<AppState>,
    Json(req): Json<CreatePORequest>,
) -> Result<Json<POResponse>, StatusCode> {
    tracing::info!("HTTP: Creating PO for product: {}, qty: {}", req.product_id, req.qty);

    let po = PurchaseOrder {
        po_id: format!("PO-{}", chrono::Utc::now().timestamp()),
        product_id: req.product_id,
        supplier_id: req.supplier_id,
        qty: req.qty,
        unit_price: req.unit_price,
        total_amount: req.total_amount,
        status: "draft".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        sent_at: None,
    };

    tracing::info!("HTTP: PO created: {}, total: ${:.2}", po.po_id, po.total_amount);

    Ok(Json(POResponse { po }))
}

async fn send_po(
    State(_state): State<AppState>,
    Json(mut po): Json<PurchaseOrder>,
) -> Result<Json<POResponse>, StatusCode> {
    tracing::info!("HTTP: Sending PO: {}", po.po_id);

    // In a real system, this would call supplier API
    po.status = "sent".to_string();
    po.sent_at = Some(chrono::Utc::now().to_rfc3339());

    tracing::info!("HTTP: PO sent successfully: {}", po.po_id);

    Ok(Json(POResponse { po }))
}
