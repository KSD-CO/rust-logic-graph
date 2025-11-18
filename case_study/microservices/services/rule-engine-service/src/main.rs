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

        // Convert gRPC request to context
        let mut context = HashMap::new();

        // Extract data
        let avg_daily_demand = req.oms_data.as_ref().map(|d| d.avg_daily_demand).unwrap_or(0.0);
        let trend = req.oms_data.as_ref().map(|d| d.trend.as_str()).unwrap_or("stable");
        let available_qty = req.inventory_data.as_ref().map(|d| d.available_qty).unwrap_or(0) as f64;
        let moq = req.supplier_data.as_ref().map(|d| d.moq).unwrap_or(0) as f64;
        let lead_time_days = req.supplier_data.as_ref().map(|d| d.lead_time_days).unwrap_or(0) as f64;
        let unit_price = req.supplier_data.as_ref().map(|d| d.unit_price).unwrap_or(0.0);
        let is_active = req.supplier_data.as_ref().map(|d| d.is_active).unwrap_or(false);

        // PRE-CALCULATE ALL values (GRL doesn't support arithmetic operations!)
        let demand_lead_time = avg_daily_demand * lead_time_days;
        let shortage = if available_qty < demand_lead_time {
            demand_lead_time - available_qty
        } else {
            0.0
        };

        // Apply trend adjustments
        let adjusted_shortage = if shortage > 0.0 {
            match trend {
                "increasing" => shortage * 1.2,
                "decreasing" => shortage * 0.9,
                _ => shortage, // stable or unknown
            }
        } else {
            0.0
        };

        // Calculate order qty
        let order_qty_calc = if adjusted_shortage > 0.0 {
            if adjusted_shortage < moq {
                moq
            } else {
                adjusted_shortage
            }
        } else {
            0.0
        };

        // Calculate total
        let total_amount_calc = order_qty_calc * unit_price;

        if let Some(oms) = &req.oms_data {
            context.insert("product_id".to_string(), json!(oms.product_id));
            context.insert("avg_daily_demand".to_string(), json!(avg_daily_demand));
            context.insert("trend".to_string(), json!(trend));
        }

        if let Some(_inv) = &req.inventory_data {
            context.insert("available_qty".to_string(), json!(available_qty));
        }

        if let Some(_sup) = &req.supplier_data {
            context.insert("moq".to_string(), json!(moq));
            context.insert("lead_time_days".to_string(), json!(lead_time_days));
            context.insert("unit_price".to_string(), json!(unit_price));
            context.insert("is_active".to_string(), json!(is_active));
        }

        // Add ALL pre-calculated values
        context.insert("demand_lead_time".to_string(), json!(demand_lead_time));
        context.insert("shortage".to_string(), json!(shortage));
        context.insert("adjusted_shortage".to_string(), json!(adjusted_shortage));
        context.insert("order_qty".to_string(), json!(order_qty_calc));
        context.insert("total_amount".to_string(), json!(total_amount_calc));

        tracing::info!("Pre-calc: shortage={}, adj_shortage={}, order_qty={}, total={}",
            shortage, adjusted_shortage, order_qty_calc, total_amount_calc);

        // Execute rules
        let mut engine = self.engine.lock();
        let result_context = engine.evaluate(&context)
            .map_err(|e| Status::internal(format!("Rule evaluation failed: {}", e)))?;

        tracing::info!("Full result context: {:?}", result_context);

        // Extract results from rule engine (GRL handles all logic now)
        let need_reorder = result_context.get("need_reorder")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let shortage = result_context.get("shortage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let order_qty = result_context.get("order_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0).ceil() as i64;

        let total_amount = result_context.get("total_amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let requires_approval = result_context.get("requires_approval")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let approval_status = result_context.get("approval_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        tracing::info!("gRPC: Rule evaluation completed: need_reorder={}, shortage={}, order_qty={}, total={}, approval={}",
            need_reorder, shortage, order_qty, total_amount, requires_approval);

        Ok(Response::new(EvaluateResponse {
            need_reorder,
            shortage,
            order_qty,
            total_amount,
            requires_approval,
            approval_status,
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

    tracing::info!("Initializing Rule Engine Service with GRL...");

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

    tracing::info!("✓ Rule Engine initialized successfully");

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

    // Convert request data to context (HashMap<String, serde_json::Value>)
    let mut context = HashMap::new();

    // Add all fields to context from nested structures
    context.insert("product_id".to_string(), json!(request.oms_data.product_id));
    context.insert("avg_daily_demand".to_string(), json!(request.oms_data.avg_daily_demand));
    context.insert("trend".to_string(), json!(request.oms_data.trend));
    context.insert("available_qty".to_string(), json!(request.inventory_data.available_qty));
    context.insert("moq".to_string(), json!(request.supplier_data.moq));
    context.insert("lead_time_days".to_string(), json!(request.supplier_data.lead_time_days));
    context.insert("unit_price".to_string(), json!(request.supplier_data.unit_price));
    context.insert("is_active".to_string(), json!(request.supplier_data.is_active));

    // Execute rules
    let mut engine = state.engine.lock();
    let result_context = match engine.evaluate(&context) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Rule evaluation failed: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Extract results from context
    let need_reorder = result_context.get("need_reorder")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let shortage = result_context.get("shortage")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let order_qty = result_context.get("order_qty")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as i64;

    let total_amount = result_context.get("total_amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let requires_approval = result_context.get("requires_approval")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let approval_status = result_context.get("approval_status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let response = RuleEngineResponse {
        need_reorder,
        shortage,
        order_qty,
        total_amount,
        requires_approval,
        approval_status,
    };

    tracing::info!("HTTP: Rule evaluation completed: need_reorder={}, order_qty={}",
        response.need_reorder, response.order_qty);

    Ok(Json(response))
}
