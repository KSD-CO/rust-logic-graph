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
use tower_http::trace::TraceLayer;
use tonic::Request;

// gRPC proto definitions
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

use oms::oms_service_client::OmsServiceClient;
use inventory::inventory_service_client::InventoryServiceClient;
use supplier::supplier_service_client::SupplierServiceClient;
use uom::uom_service_client::UomServiceClient;
use rule_engine::rule_engine_service_client::RuleEngineServiceClient;
use po::po_service_client::PoServiceClient;

#[derive(Clone)]
struct AppState {
    http_client: Client,
    // gRPC URLs for inter-service communication
    oms_grpc_url: String,
    inventory_grpc_url: String,
    supplier_grpc_url: String,
    uom_grpc_url: String,
    rule_engine_grpc_url: String,
    po_grpc_url: String,
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

    let state = AppState {
        http_client,
        oms_grpc_url,
        inventory_grpc_url,
        supplier_grpc_url,
        uom_grpc_url,
        rule_engine_grpc_url,
        po_grpc_url,
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
    tracing::info!("Starting purchasing flow for product: {}", req.product_id);

    // Step 1: Fetch data from all services in parallel
    tracing::info!("Step 1: Fetching data from all services...");

    let product_id = req.product_id.clone();

    let (oms_result, inventory_result, supplier_result, uom_result) = tokio::join!(
        fetch_oms_data(&state, &product_id),
        fetch_inventory_data(&state, &product_id),
        fetch_supplier_data(&state, &product_id),
        fetch_uom_data(&state, &product_id)
    );

    let oms_data = oms_result.map_err(|e| {
        tracing::error!("Failed to fetch OMS data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let inventory_data = inventory_result.map_err(|e| {
        tracing::error!("Failed to fetch inventory data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let supplier_data = supplier_result.map_err(|e| {
        tracing::error!("Failed to fetch supplier data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let _uom_data = uom_result.map_err(|e| {
        tracing::error!("Failed to fetch UOM data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Step 1: All data fetched successfully");

    // Step 2: Evaluate business rules
    tracing::info!("Step 2: Evaluating business rules via gRPC...");

    let mut rule_client = RuleEngineServiceClient::connect(state.rule_engine_grpc_url.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to rule engine: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let rule_request = rule_engine::EvaluateRequest {
        oms_data: Some(rule_engine::OmsData {
            product_id: oms_data.product_id.clone(),
            avg_daily_demand: oms_data.avg_daily_demand,
            trend: oms_data.trend.clone(),
        }),
        inventory_data: Some(rule_engine::InventoryData {
            product_id: inventory_data.product_id.clone(),
            warehouse_id: inventory_data.warehouse_id.clone(),
            current_qty: inventory_data.current_qty,
            reserved_qty: inventory_data.reserved_qty,
            available_qty: inventory_data.available_qty,
        }),
        supplier_data: Some(rule_engine::SupplierData {
            supplier_id: supplier_data.supplier_id.clone(),
            product_id: supplier_data.product_id.clone(),
            moq: supplier_data.moq,
            lead_time_days: supplier_data.lead_time_days,
            unit_price: supplier_data.unit_price,
            is_active: supplier_data.is_active,
        }),
    };

    let grpc_response = rule_client
        .evaluate(Request::new(rule_request))
        .await
        .map_err(|e| {
            tracing::error!("gRPC call to rule engine failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let rule_result_grpc = grpc_response.into_inner();

    // Extract flags we need before moving data
    let should_create_po = rule_result_grpc.should_create_po;
    let should_send_po = rule_result_grpc.should_send_po;
    let po_status = rule_result_grpc.po_status.clone();
    let send_method = rule_result_grpc.send_method.clone();

    // Convert gRPC response to REST model for compatibility
    let rule_result = RuleEngineResponse {
        need_reorder: rule_result_grpc.need_reorder,
        shortage: rule_result_grpc.shortage,
        order_qty: rule_result_grpc.order_qty,
        total_amount: rule_result_grpc.total_amount,
        requires_approval: rule_result_grpc.requires_approval,
        approval_status: rule_result_grpc.approval_status,
    };

    tracing::info!("Step 2: Rules evaluated - need_reorder: {}, order_qty: {}, total: ${:.2}, should_create_po: {}, should_send_po: {}",
                   rule_result.need_reorder, rule_result.order_qty, rule_result.total_amount,
                   should_create_po, should_send_po);

    // Step 3: Execute workflow based on rules decisions (orchestrator executes, rules decide)
    match execute_workflow_actions(&state, &req, &rule_result, should_create_po, should_send_po, &po_status, &send_method, &supplier_data).await {
        Ok(response) => Ok(Json(response)),
        Err(status) => Err(status),
    }
}

/// Execute workflow actions based on rule engine decisions
/// Orchestrator is a pure executor - rules engine makes all business decisions
async fn execute_workflow_actions(
    state: &AppState,
    req: &PurchasingFlowRequest,
    rule_result: &RuleEngineResponse,
    should_create_po: bool,
    should_send_po: bool,
    po_status: &str,
    send_method: &str,
    supplier_data: &SupplierData,
) -> Result<PurchasingFlowResponse, StatusCode> {
    
    // Action 1: Check if PO creation is needed (decided by rules via should_create_po flag)
    if !should_create_po {
        tracing::info!("Workflow: No PO creation (rules decided: should_create_po=false)");
        return Ok(PurchasingFlowResponse {
            success: true,
            po: None,
            calculation: Some(rule_result.clone()),
            message: "No PO creation needed based on business rules".to_string(),
        });
    }

    // Action 2: Create Purchase Order (rules approved via should_create_po=true)
    tracing::info!("Workflow: Creating PO (rules decided: should_create_po=true, po_status={})", po_status);

    let mut po_client = PoServiceClient::connect(state.po_grpc_url.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to PO service: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let create_po_request = po::CreateRequest {
        product_id: req.product_id.clone(),
        supplier_id: supplier_data.supplier_id.clone(),
        qty: rule_result.order_qty,
        unit_price: supplier_data.unit_price,
        total_amount: rule_result.total_amount,
    };

    let po_grpc_response = po_client
        .create(Request::new(create_po_request))
        .await
        .map_err(|e| {
            tracing::error!("gRPC call to create PO failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let created_po_grpc = po_grpc_response.into_inner().po.ok_or_else(|| {
        tracing::error!("PO not found in create response");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Workflow: PO created - {} (status: {})", created_po_grpc.po_id, po_status);

    // Action 3: Send Purchase Order (if rules decided should_send_po=true)
    let final_po = if should_send_po {
        tracing::info!("Workflow: Sending PO (rules decided: should_send_po=true, method={})", send_method);

        let send_po_request = po::SendRequest {
            po_id: created_po_grpc.po_id.clone(),
        };

        let sent_po_grpc_response = po_client
            .send(Request::new(send_po_request))
            .await
            .map_err(|e| {
                tracing::error!("gRPC call to send PO failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let sent_po_grpc = sent_po_grpc_response.into_inner().po.ok_or_else(|| {
            tracing::error!("PO not found in send response");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        tracing::info!("Workflow: PO sent successfully - {}", sent_po_grpc.po_id);

        // Convert gRPC PO to REST model for response
        PurchaseOrder {
            po_id: sent_po_grpc.po_id,
            product_id: sent_po_grpc.product_id,
            supplier_id: sent_po_grpc.supplier_id,
            qty: sent_po_grpc.qty,
            unit_price: sent_po_grpc.unit_price,
            total_amount: sent_po_grpc.total_amount,
            status: sent_po_grpc.status,
            created_at: sent_po_grpc.created_at,
            sent_at: if sent_po_grpc.sent_at.is_empty() {
                None
            } else {
                Some(sent_po_grpc.sent_at)
            },
        }
    } else {
        tracing::info!("Workflow: PO created but not sent (rules decided: should_send_po=false, requires_approval={})",
            rule_result.requires_approval);

        // Convert created PO (not sent)
        PurchaseOrder {
            po_id: created_po_grpc.po_id,
            product_id: created_po_grpc.product_id,
            supplier_id: created_po_grpc.supplier_id,
            qty: created_po_grpc.qty,
            unit_price: created_po_grpc.unit_price,
            total_amount: created_po_grpc.total_amount,
            status: created_po_grpc.status,
            created_at: created_po_grpc.created_at,
            sent_at: None,
        }
    };

    Ok(PurchasingFlowResponse {
        success: true,
        po: Some(final_po),
        calculation: Some(rule_result.clone()),
        message: "Purchasing flow completed - orchestrator executed rules decisions".to_string(),
    })
}

async fn fetch_oms_data(state: &AppState, product_id: &str) -> anyhow::Result<OmsHistoryData> {
    tracing::debug!("[gRPC] Connecting to OMS service: {}", state.oms_grpc_url);
    let mut client = OmsServiceClient::connect(state.oms_grpc_url.clone()).await?;

    let request = Request::new(oms::HistoryRequest {
        product_id: product_id.to_string(),
    });

    let response = client.get_history(request).await?;
    let grpc_data = response.into_inner();

    Ok(OmsHistoryData {
        product_id: grpc_data.product_id,
        avg_daily_demand: grpc_data.avg_daily_demand,
        trend: grpc_data.trend,
    })
}

async fn fetch_inventory_data(state: &AppState, product_id: &str) -> anyhow::Result<InventoryData> {
    tracing::debug!("[gRPC] Connecting to Inventory service: {}", state.inventory_grpc_url);
    let mut client = InventoryServiceClient::connect(state.inventory_grpc_url.clone()).await?;

    let request = Request::new(inventory::LevelsRequest {
        product_id: product_id.to_string(),
    });

    let response = client.get_levels(request).await?;
    let grpc_data = response.into_inner();

    // Map proto fields to model fields
    // Proto has: available_qty, reserved_qty, on_order_qty
    // Model needs: warehouse_id, current_qty, reserved_qty, available_qty
    Ok(InventoryData {
        product_id: grpc_data.product_id,
        warehouse_id: "WH-001".to_string(), // Default warehouse
        current_qty: grpc_data.available_qty + grpc_data.reserved_qty, // Calculate current from available + reserved
        reserved_qty: grpc_data.reserved_qty,
        available_qty: grpc_data.available_qty,
    })
}

async fn fetch_supplier_data(state: &AppState, product_id: &str) -> anyhow::Result<SupplierData> {
    tracing::debug!("[gRPC] Connecting to Supplier service: {}", state.supplier_grpc_url);
    let mut client = SupplierServiceClient::connect(state.supplier_grpc_url.clone()).await?;

    let request = Request::new(supplier::InfoRequest {
        product_id: product_id.to_string(),
    });

    let response = client.get_info(request).await?;
    let grpc_data = response.into_inner();

    Ok(SupplierData {
        supplier_id: grpc_data.supplier_name.clone(), // Map supplier_name to supplier_id
        product_id: grpc_data.product_id,
        moq: grpc_data.moq,
        lead_time_days: grpc_data.lead_time_days,
        unit_price: grpc_data.unit_price,
        is_active: true, // Default to true as proto doesn't have this field
    })
}

async fn fetch_uom_data(state: &AppState, product_id: &str) -> anyhow::Result<UomConversionData> {
    tracing::debug!("[gRPC] Connecting to UOM service: {}", state.uom_grpc_url);
    let mut client = UomServiceClient::connect(state.uom_grpc_url.clone()).await?;

    let request = Request::new(uom::ConversionRequest {
        product_id: product_id.to_string(),
    });

    let response = client.get_conversion(request).await?;
    let grpc_data = response.into_inner();

    // Map proto fields to model fields
    // Proto has: base_unit, case_qty, pallet_qty
    // Model needs: from_uom, to_uom, conversion_factor
    Ok(UomConversionData {
        product_id: grpc_data.product_id,
        from_uom: grpc_data.base_unit.clone(),
        to_uom: "case".to_string(), // Default to case conversion
        conversion_factor: grpc_data.case_qty as f64, // Use case_qty as conversion factor
    })
}
