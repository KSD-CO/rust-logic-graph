use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use purchasing_models::{HealthResponse, RuleEngineRequest, RuleEngineResponse};
use rust_logic_graph::RuleEngine;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use parking_lot::Mutex;
use tower_http::trace::TraceLayer;
use tonic::{transport::Server, Request, Response, Status};

mod action_executor;

// Include generated gRPC code
pub mod rule_engine {
    tonic::include_proto!("rule_engine");
}

use rule_engine::{
    rule_engine_service_server::{RuleEngineService as GrpcRuleEngineService, RuleEngineServiceServer},
    EvaluateRequest, EvaluateResponse, HealthRequest, HealthResponse as GrpcHealthResponse,
};

#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<RuleEngine>>,
}

// gRPC service implementation
struct RuleEngineGrpcService {
    engine: Arc<Mutex<RuleEngine>>,
}

#[tonic::async_trait]
impl GrpcRuleEngineService for RuleEngineGrpcService {
    async fn evaluate(
        &self,
        request: Request<EvaluateRequest>,
    ) -> Result<Response<EvaluateResponse>, Status> {
        let req = request.into_inner();

        tracing::info!("gRPC: Evaluating rules for product: {}",
            req.oms_data.as_ref().map(|d| d.product_id.as_str()).unwrap_or("unknown"));

        // Convert gRPC request to context (HashMap with JSON values)
        let mut context = HashMap::new();

        // Extract raw data
        let avg_daily_demand = req.oms_data.as_ref().map(|d| d.avg_daily_demand).unwrap_or(0.0);
        let available_qty = req.inventory_data.as_ref().map(|d| d.available_qty).unwrap_or(0) as f64;
        let moq = req.supplier_data.as_ref().map(|d| d.moq).unwrap_or(0) as f64;
        let lead_time_days = req.supplier_data.as_ref().map(|d| d.lead_time_days).unwrap_or(0) as f64;
        let unit_price = req.supplier_data.as_ref().map(|d| d.unit_price).unwrap_or(0.0);
        let is_active = req.supplier_data.as_ref().map(|d| d.is_active).unwrap_or(false);

        // Calculate required_qty for GRL
        let required_qty = avg_daily_demand * lead_time_days;

        // Add all input fields to context
        if let Some(oms) = &req.oms_data {
            context.insert("product_id".to_string(), json!(oms.product_id));
            context.insert("avg_daily_demand".to_string(), json!(avg_daily_demand));
            context.insert("trend".to_string(), json!(oms.trend));
        }

        context.insert("available_qty".to_string(), json!(available_qty));
        context.insert("moq".to_string(), json!(moq));
        context.insert("lead_time_days".to_string(), json!(lead_time_days));
        context.insert("unit_price".to_string(), json!(unit_price));
        context.insert("is_active".to_string(), json!(is_active));
        context.insert("required_qty".to_string(), json!(required_qty));

        // Initialize output fields (required for rules to work properly)
    context.insert("shortage".to_string(), json!(0.0));
    context.insert("order_qty".to_string(), json!(0.0));
    context.insert("total_amount".to_string(), json!(0.0));
    context.insert("need_reorder".to_string(), json!(false));
    context.insert("requires_approval".to_string(), json!(false));
    context.insert("approval_status".to_string(), json!(""));
    context.insert("discount_amount".to_string(), json!(0.0));
    context.insert("final_amount".to_string(), json!(0.0));
    context.insert("tax_amount".to_string(), json!(0.0));
    context.insert("grand_total".to_string(), json!(0.0));
    context.insert("should_create_po".to_string(), json!(false));
    context.insert("should_send_po".to_string(), json!(false));
    context.insert("po_status".to_string(), json!(""));
    context.insert("send_method".to_string(), json!(""));

    tracing::info!("Input to GRL v0.17: required_qty={}, available_qty={}, moq={}, unit_price={}",
            required_qty, available_qty, moq, unit_price);

        // Execute rules with action handlers (wrapper handles Facts conversion)
        let mut engine = self.engine.lock();
        let result_json = engine.evaluate(&context)
            .map_err(|e| Status::internal(format!("Rule evaluation failed: {}", e)))?;

        tracing::info!("GRL v0.17 evaluation results: {:?}", result_json);

        // Extract results from JSON response
        let need_reorder = result_json.get("need_reorder")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let shortage = result_json.get("shortage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let order_qty = result_json.get("order_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0).ceil() as i64;

        let total_amount = result_json.get("total_amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let requires_approval = result_json.get("requires_approval")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let approval_status = result_json.get("approval_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let grand_total = result_json.get("grand_total")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let should_create_po = result_json.get("should_create_po")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let should_send_po = result_json.get("should_send_po")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let po_status = result_json.get("po_status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let send_method = result_json.get("send_method")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        tracing::info!("gRPC: ✅ shortage={:.0}, order_qty={}, total=${:.2}, approval={}, grand_total=${:.2}, should_create_po={}, should_send_po={}",
            shortage, order_qty, total_amount, requires_approval, grand_total, should_create_po, should_send_po);

        Ok(Response::new(EvaluateResponse {
            need_reorder,
            shortage,
            order_qty,
            total_amount,
            requires_approval,
            approval_status,
            should_create_po,
            should_send_po,
            po_status,
            send_method,
            grand_total,
        }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<GrpcHealthResponse>, Status> {
        Ok(Response::new(GrpcHealthResponse {
            status: "healthy".to_string(),
            service: "rule-engine-service".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    tracing::info!("Initializing Rule Engine Service with GRL v0.17 + Action Handlers...");

    // Create RuleEngine with GRL rules
    let mut engine = RuleEngine::new();

    // Load purchasing flow rules from GRL file
    let rules_file = "rules/purchasing_rules.grl";
    tracing::info!("Loading rules from file: {}", rules_file);

    let rules_grl = std::fs::read_to_string(rules_file)
        .map_err(|e| {
            tracing::error!("Failed to read rules file {}: {}", rules_file, e);
            anyhow::anyhow!("Failed to read rules file: {}", e)
        })?;

    tracing::info!("Rules file content loaded, {} bytes", rules_grl.len());

    // Add GRL rules
    match engine.add_grl_rule(&rules_grl) {
        Ok(_) => {
            tracing::info!("✓ GRL rules loaded successfully from {}", rules_file);
        }
        Err(e) => {
            tracing::error!("Failed to parse GRL rules: {}", e);
            return Err(anyhow::anyhow!("Failed to parse GRL rules: {}", e));
        }
    }

    // 🎯 Custom action handlers disabled - orchestrator handles execution
    // action_executor::register_action_handlers(engine.inner_mut());

    tracing::info!("✓ Rule Engine initialized successfully (calculation mode)");

    let engine_arc = Arc::new(Mutex::new(engine));

    // Setup HTTP server
    let http_state = AppState {
        engine: engine_arc.clone(),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/rules/evaluate", post(evaluate_rules))
        .layer(TraceLayer::new_for_http())
        .with_state(http_state);

    let http_port = env::var("PORT").unwrap_or_else(|_| "8085".to_string());
    let http_addr = format!("0.0.0.0:{}", http_port);

    // Setup gRPC server
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50055".to_string());
    let grpc_addr = format!("0.0.0.0:{}", grpc_port).parse()?;

    let grpc_service = RuleEngineGrpcService {
        engine: engine_arc,
    };

    tracing::info!("Rule Engine Service listening on HTTP {} and gRPC {}", http_addr, grpc_addr);

    // Run both servers concurrently
    let http_server = async {
        let listener = tokio::net::TcpListener::bind(&http_addr).await?;
        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    };

    let grpc_server = async {
        Server::builder()
            .add_service(RuleEngineServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await?;
        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(http_server, grpc_server)?;

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "rule-engine-service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn evaluate_rules(
    State(state): State<AppState>,
    Json(request): Json<RuleEngineRequest>,
) -> Result<Json<RuleEngineResponse>, StatusCode> {
    tracing::info!("HTTP: Evaluating rules for product: {}", request.oms_data.product_id);

    // Convert request data to context (HashMap with JSON values)
    let mut context = HashMap::new();

    // Extract values
    let avg_daily_demand = request.oms_data.avg_daily_demand;
    let available_qty = request.inventory_data.available_qty as f64;
    let lead_time_days = request.supplier_data.lead_time_days as f64;
    let unit_price = request.supplier_data.unit_price;
    let moq = request.supplier_data.moq as f64;
    let is_active = request.supplier_data.is_active;

    // Calculate required_qty
    let required_qty = avg_daily_demand * lead_time_days;

    // Add all input fields to context
    context.insert("product_id".to_string(), json!(request.oms_data.product_id));
    context.insert("avg_daily_demand".to_string(), json!(avg_daily_demand));
    context.insert("trend".to_string(), json!(request.oms_data.trend));
    context.insert("available_qty".to_string(), json!(available_qty));
    context.insert("moq".to_string(), json!(moq));
    context.insert("lead_time_days".to_string(), json!(lead_time_days));
    context.insert("unit_price".to_string(), json!(unit_price));
    context.insert("is_active".to_string(), json!(is_active));
    context.insert("required_qty".to_string(), json!(required_qty));

    // Initialize output fields (required for rules to work properly)
    context.insert("shortage".to_string(), json!(0.0));
    context.insert("order_qty".to_string(), json!(0.0));
    context.insert("total_amount".to_string(), json!(0.0));
    context.insert("need_reorder".to_string(), json!(false));
    context.insert("requires_approval".to_string(), json!(false));
    context.insert("approval_status".to_string(), json!(""));
    context.insert("discount_amount".to_string(), json!(0.0));
    context.insert("final_amount".to_string(), json!(0.0));
    context.insert("tax_amount".to_string(), json!(0.0));
    context.insert("grand_total".to_string(), json!(0.0));

    tracing::info!("Input to GRL v0.17: required_qty={}, available_qty={}, moq={}, unit_price={}, is_active={}", 
        required_qty, available_qty, moq, unit_price, is_active);

    // Execute rules with action handlers (wrapper handles Facts conversion)
    let mut engine = state.engine.lock();
    let result_json = match engine.evaluate(&context) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Rule evaluation failed: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Extract results from JSON response
    let need_reorder = result_json.get("need_reorder")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let shortage = result_json.get("shortage")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let order_qty = result_json.get("order_qty")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as i64;

    let total_amount = result_json.get("total_amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let requires_approval = result_json.get("requires_approval")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let approval_status = result_json.get("approval_status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let grand_total = result_json.get("grand_total")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let response = RuleEngineResponse {
        need_reorder,
        shortage,
        order_qty,
        total_amount,
        requires_approval,
        approval_status,
    };

    tracing::info!("HTTP: ✅ shortage={:.0}, order_qty={}, total=${:.2}, grand_total=${:.2}",
        response.shortage, response.order_qty, response.total_amount, grand_total);

    Ok(Json(response))
}
