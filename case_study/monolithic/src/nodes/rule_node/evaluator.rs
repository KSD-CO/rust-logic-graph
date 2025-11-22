use serde_json::{json, Value};
use rust_logic_graph::Context;
use rust_logic_graph::rule::RuleError;
use rust_logic_graph::error::{RustLogicGraphError, ErrorContext};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::services::RuleEngineService;

// Global cached rule engine service (initialized once)
static RULE_ENGINE: OnceLock<Result<Mutex<RuleEngineService>, String>> = OnceLock::new();

fn get_rule_engine() -> Result<&'static Mutex<RuleEngineService>, RuleError> {
    let is_first_init = RULE_ENGINE.get().is_none();
    if is_first_init {
        tracing::info!("🔧 [Cache] First request - initializing RuleEngineService (will take ~2.6s)");
    } else {
        tracing::info!("⚡ [Cache] Using cached RuleEngineService (should be instant)");
    }
    
    RULE_ENGINE.get_or_init(|| {
        tracing::info!("⏱️  [Cache] Calling RuleEngineService::new()...");
        RuleEngineService::new()
            .map(Mutex::new)
    })
    .as_ref()
    .map_err(|e| RuleError::Eval(
        RustLogicGraphError::rule_evaluation_error(
            format!("Failed to initialize rule engine: {}", e)
        )
        .with_context(
            ErrorContext::new()
                .with_service("RuleEngineService")
                .add_metadata("initialization", "global_cache")
        )
        .to_string()
    ))
}

/// Evaluate rule engine logic
pub fn evaluate_rule_engine(
    ctx: &mut Context,
    node_id: &str,
    field_mappings: &HashMap<String, String>,
) -> Result<Value, RuleError> {
    let total_start = std::time::Instant::now();
    
    tracing::debug!("🔍 Rule Engine: field_mappings = {:?}", field_mappings);
    tracing::debug!("🔍 Rule Engine: context keys = {:?}", ctx.data.keys().collect::<Vec<_>>());
    
    let inputs = extract_inputs(ctx, field_mappings);
    
    tracing::info!("📊 Rule Engine - Dynamic Input Fields: {:?}", inputs.keys().collect::<Vec<_>>());
    
    // Use cached rule engine service
    let cache_start = std::time::Instant::now();
    tracing::info!("⏱️  [Cache Access] Calling get_rule_engine()...");
    
    // TEMPORARY FIX: Create new RuleEngineService for each request to avoid cache issues
    eprintln!("⚠️  Creating NEW RuleEngineService (disabling cache)");
    let mut service = RuleEngineService::new()
        .map_err(|e| RuleError::Eval(
            RustLogicGraphError::rule_evaluation_error(
                format!("Failed to create rule engine: {}", e)
            )
            .with_context(
                ErrorContext::new()
                    .with_node(node_id)
                    .with_service("RuleEngineService")
                    .add_metadata("mode", "non_cached")
            )
            .to_string()
        ))?;
    let cache_elapsed = cache_start.elapsed();
    tracing::info!("✅ [Cache Access] RuleEngineService::new() took {:.3}ms", cache_elapsed.as_secs_f64() * 1000.0);
    
    let eval_start = std::time::Instant::now();    
    let rule_output = service.evaluate(inputs)
        .map_err(|e| RuleError::Eval(
            RustLogicGraphError::rule_evaluation_error(
                format!("Rule evaluation failed: {}", e)
            )
            .with_context(
                ErrorContext::new()
                    .with_node(node_id)
                    .with_graph("purchasing_flow")
                    .with_step("rule_evaluation")
                    .with_service("RuleEngineService")
            )
            .to_string()
        ))?;
    let eval_elapsed = eval_start.elapsed();
    tracing::info!("⏱️  Rule service.evaluate(): {:.3}s", eval_elapsed.as_secs_f64());
    
    let result = json!({
        "should_order": rule_output.should_order,
        "recommended_qty": rule_output.recommended_qty,
        "required_qty": rule_output.required_qty,
        "shortage": rule_output.shortage,
        "reason": rule_output.reason,
        "calculation_details": {
            "avg_daily_demand": rule_output.calculation_details.avg_daily_demand,
            "available_qty": rule_output.calculation_details.available_qty,
            "lead_time": rule_output.calculation_details.lead_time,
            "required_qty": rule_output.calculation_details.required_qty,
            "safety_stock": rule_output.calculation_details.safety_stock,
            "target_qty": rule_output.calculation_details.target_qty,
            "shortage": rule_output.calculation_details.shortage,
            "moq": rule_output.calculation_details.moq,
            "recommended_qty": rule_output.calculation_details.recommended_qty,
        }
    });
    
    // Insert with both node id and legacy "rule_result" for backward compatibility
    ctx.data.insert(node_id.to_string(), result.clone());
    ctx.data.insert("rule_result".to_string(), result.clone());
    
    let total_elapsed = total_start.elapsed();
    tracing::info!("✅ evaluate_rule_engine() total: {:.3}s", total_elapsed.as_secs_f64());
    
    Ok(result)
}

/// Create purchase order
pub fn create_purchase_order(
    ctx: &mut Context,
    field_mappings: &HashMap<String, String>,
) -> Result<Value, RuleError> {
    let inputs = extract_inputs(ctx, field_mappings);
    
    // Extract values with defaults
    let should_order = inputs.get("should_order").and_then(|v| v.as_bool()).unwrap_or(false);
    let recommended_qty = inputs.get("recommended_qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let product_id = inputs.get("product_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let unit_price = inputs.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let lead_time = inputs.get("lead_time").and_then(|v| v.as_i64()).unwrap_or(7);
    let from_uom = inputs.get("from_uom").and_then(|v| v.as_str()).unwrap_or("units").to_string();
    
    tracing::info!("🔍 Create PO - Config-driven mode: {}", !field_mappings.is_empty());
    tracing::info!("🔍 Create PO - should_order: {}, qty: {}, product: {}", 
        should_order, recommended_qty, product_id);
    
    if !should_order {
        tracing::info!("ℹ️  No PO needed");
        ctx.data.insert("po".to_string(), Value::Null);
        return Ok(Value::Null);
    }
    
    let expected_delivery = chrono::Utc::now() + chrono::Duration::days(lead_time);
    
    let po = json!({
        "product_id": product_id,
        "order_qty": recommended_qty,
        "order_unit": from_uom,
        "supplier_id": format!("SUPP-{}", product_id),
        "expected_delivery_date": expected_delivery.format("%Y-%m-%d").to_string(),
        "total_cost": recommended_qty * unit_price,
    });
    
    ctx.data.insert("po".to_string(), po.clone());
    Ok(po)
}

/// Extract inputs from context using field mappings
fn extract_inputs(ctx: &Context, field_mappings: &HashMap<String, String>) -> HashMap<String, Value> {
    let mut inputs = HashMap::new();
    
    if !field_mappings.is_empty() {
        // Config-driven: dynamically extract all fields from mappings
        for (key, path) in field_mappings {
            if let Some(value) = get_value_by_path(ctx, path) {
                inputs.insert(key.clone(), value);
            }
        }
    } else {
        // Hardcoded fallback: merge all context data
        for (node_id, node_data) in &ctx.data {
            if let Some(obj) = node_data.as_object() {
                for (field_key, field_value) in obj {
                    inputs.insert(field_key.clone(), field_value.clone());
                }
            }
        }
    }
    
    inputs
}

/// Get value from context using path notation (e.g., "oms_data.avg_daily_demand")
fn get_value_by_path(ctx: &Context, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    
    let mut current = ctx.data.get(parts[0])?.clone();
    
    for part in &parts[1..] {
        current = current.get(part)?.clone();
    }
    
    Some(current)
}
