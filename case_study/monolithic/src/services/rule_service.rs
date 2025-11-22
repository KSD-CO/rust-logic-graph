/// Rule Engine Service - Business logic for purchasing decisions
/// Uses Grule Rule Language (GRL) files for flexible business rules
/// Pattern: Same as microservices/services/rule-engine-service

use serde::{Deserialize, Serialize};
use rust_logic_graph::RuleEngine;
use serde_json::{json, Value};
use std::collections::HashMap;

// No need for RuleEngineInput struct - use HashMap<String, Value> directly

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineOutput {
    pub should_order: bool,
    pub recommended_qty: f64,
    pub required_qty: f64,
    pub shortage: f64,
    pub reason: String,
    pub calculation_details: CalculationDetails,
    pub order_qty: f64,
    pub total_amount: f64,
    pub need_reorder: bool,
    pub requires_approval: bool,
    pub approval_status: String,
    pub should_create_po: bool,
    pub should_send_po: bool,
    pub po_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationDetails {
    pub avg_daily_demand: f64,
    pub available_qty: f64,
    pub lead_time: f64,
    pub required_qty: f64,
    pub safety_stock: f64,
    pub target_qty: f64,
    pub shortage: f64,
    pub moq: f64,
    pub recommended_qty: f64,
}

pub struct RuleEngineService {
    engine: RuleEngine,
}

impl RuleEngineService {
    pub fn new() -> Result<Self, String> {
        let start = std::time::Instant::now();
        tracing::info!("⏱️  [Step 1/4] Creating RuleEngine instance...");
        
        let engine_start = std::time::Instant::now();
        let mut engine = RuleEngine::new();
        tracing::info!("   ✅ RuleEngine::new() took {:.3}ms", engine_start.elapsed().as_secs_f64() * 1000.0);
        
        let grl_path = "purchasing_rules.grl";
        tracing::info!("⏱️  [Step 2/4] Reading GRL file: {}", grl_path);
        
        let read_start = std::time::Instant::now();
        let rules_grl = std::fs::read_to_string(grl_path)
            .map_err(|e| format!("Failed to read GRL file {}: {}", grl_path, e))?;
        tracing::info!("   ✅ File read took {:.3}ms ({} bytes)", 
            read_start.elapsed().as_secs_f64() * 1000.0, rules_grl.len());
        
        tracing::info!("⏱️  [Step 3/4] Parsing GRL rules with engine.add_grl_rule()...");
        let parse_start = std::time::Instant::now();
        engine.add_grl_rule(&rules_grl)
            .map_err(|e| format!("Failed to parse GRL rules: {}", e))?;
        tracing::info!("   ✅ GRL parsing took {:.3}s", parse_start.elapsed().as_secs_f64());
        
        let elapsed = start.elapsed();
        tracing::info!("⏱️  [Step 4/4] RuleEngineService initialization complete");
        tracing::info!("✅ Total GRL load time: {:.3}s", elapsed.as_secs_f64());
        
        Ok(Self { engine })
    }

    pub fn evaluate(&mut self, inputs: HashMap<String, Value>) -> Result<RuleEngineOutput, String> {
        eprintln!("💰💰💰 RuleEngineService.evaluate() CALLED 💰💰💰");
        eprintln!("Input keys: {:?}", inputs.keys().collect::<Vec<_>>());
        
        // Calculate required_qty for GRL if needed
        let avg_daily_demand = inputs.get("avg_daily_demand").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let lead_time = inputs.get("lead_time").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let required_qty = avg_daily_demand * lead_time;
        
        eprintln!("Calculated: required_qty={}, avg_daily_demand={}, lead_time={}", required_qty, avg_daily_demand, lead_time);
        
        // Start with input context
        let mut context = inputs.clone();
        
        // Add calculated field
        context.insert("required_qty".to_string(), json!(required_qty));
        
        // Add is_active field (default to true for suppliers)
        context.entry("is_active".to_string()).or_insert(json!(true));
        
        // Initialize output fields (required for GRL rules)
        context.entry("shortage".to_string()).or_insert(json!(0.0));
        context.entry("order_qty".to_string()).or_insert(json!(0.0));
        context.entry("total_amount".to_string()).or_insert(json!(0.0));
        context.entry("need_reorder".to_string()).or_insert(json!(false));
        context.entry("requires_approval".to_string()).or_insert(json!(false));
        context.entry("approval_status".to_string()).or_insert(json!(""));
        context.entry("discount_amount".to_string()).or_insert(json!(0.0));
        context.entry("final_amount".to_string()).or_insert(json!(0.0));
        context.entry("tax_amount".to_string()).or_insert(json!(0.0));
        context.entry("grand_total".to_string()).or_insert(json!(0.0));
        context.entry("should_create_po".to_string()).or_insert(json!(false));
        context.entry("should_send_po".to_string()).or_insert(json!(false));
        context.entry("po_status".to_string()).or_insert(json!(""));
        context.entry("send_method".to_string()).or_insert(json!(""));
        context.entry("supplier_error".to_string()).or_insert(json!(""));
        
        let available_qty = context.get("available_qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let moq = context.get("moq").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let unit_price = context.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        tracing::info!("Input to GRL: required_qty={}, available_qty={}, moq={}, unit_price={}",
            required_qty, available_qty, moq, unit_price);
        
        // Execute rules
        let rule_start = std::time::Instant::now();
        tracing::info!("⏱️  [Evaluation] Starting GRL rule execution...");
        tracing::info!("   📊 Context size: {} fields", context.len());
        
        eprintln!("🚀🚀🚀 ABOUT TO CALL engine.evaluate() 🚀🚀🚀");
        eprintln!("Context values: available_qty={}, required_qty={}, shortage={}", 
            context.get("available_qty").and_then(|v| v.as_f64()).unwrap_or(-1.0),
            context.get("required_qty").and_then(|v| v.as_f64()).unwrap_or(-1.0),
            context.get("shortage").and_then(|v| v.as_f64()).unwrap_or(-1.0)
        );
        
        let eval_start = std::time::Instant::now();
        let result_json = self.engine.evaluate(&context)
            .map_err(|e| format!("Rule evaluation failed: {}", e))?;
        let eval_elapsed = eval_start.elapsed();
        
        eprintln!("✅✅✅ engine.evaluate() RETURNED ✅✅✅");
        eprintln!("Result: should_create_po={}, order_qty={}", 
            result_json.get("should_create_po").and_then(|v| v.as_bool()).unwrap_or(false),
            result_json.get("order_qty").and_then(|v| v.as_f64()).unwrap_or(-1.0)
        );
        
        let rule_elapsed = rule_start.elapsed();
        tracing::info!("   ✅ engine.evaluate() took {:.3}s", eval_elapsed.as_secs_f64());
        tracing::info!("✅ [Evaluation] Total GRL execution: {:.3}s", rule_elapsed.as_secs_f64());
        tracing::debug!("GRL evaluation results: {:?}", result_json);
        
        // Extract results from JSON
        let shortage = result_json.get("shortage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let order_qty = result_json.get("order_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let total_amount = result_json.get("total_amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let need_reorder = result_json.get("need_reorder")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let requires_approval = result_json.get("requires_approval")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let approval_status = result_json.get("approval_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
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
        
        let supplier_error = result_json.get("supplier_error")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Determine reason and should_order
        let reason = if !supplier_error.is_empty() {
            supplier_error
        } else if need_reorder {
            format!("Shortage: {:.0} units, Order: {:.0} units", shortage, order_qty)
        } else {
            "No reorder needed".to_string()
        };
        
        let should_order = need_reorder && order_qty > 0.0;
        
        let safety_stock = context.get("safety_stock").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let target_qty = required_qty + safety_stock;
        
        tracing::info!("✅ shortage={:.0}, order_qty={:.0}, total=${:.2}, approval={}, should_create_po={}, should_send_po={}",
            shortage, order_qty, total_amount, requires_approval, should_create_po, should_send_po);
        
        Ok(RuleEngineOutput {
            should_order,
            recommended_qty: order_qty,
            required_qty,
            shortage,
            reason,
            calculation_details: CalculationDetails {
                avg_daily_demand,
                available_qty,
                lead_time,
                required_qty,
                safety_stock,
                target_qty,
                shortage,
                moq,
                recommended_qty: order_qty,
            },
            order_qty,
            total_amount,
            need_reorder,
            requires_approval,
            approval_status,
            should_create_po,
            should_send_po,
            po_status,
        })
    }
}
