use crate::models::{PurchasingContext, RuleEvaluationResult};
use anyhow::Result;
use rust_logic_graph::RuleEngine;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct RuleEngineService {
    engine: Mutex<RuleEngine>,
}

impl RuleEngineService {
    pub fn new(rule_file_path: &str) -> Result<Self> {
        tracing::info!("🔧 Initializing Rule Engine Service (Monolithic with GRL)");
        tracing::info!("📂 Loading rules from: {}", rule_file_path);
        
        let grl_content = std::fs::read_to_string(rule_file_path)?;
        let engine = RuleEngine::from_grl(&grl_content)?;
        
        tracing::info!("✅ GRL rules loaded successfully");
        
        Ok(Self { engine: Mutex::new(engine) })
    }

    pub fn evaluate(&self, context: &PurchasingContext) -> Result<RuleEvaluationResult> {
        tracing::info!("📊 Starting rule evaluation for product: {}", context.oms_data.product_id);
        
        // Calculate required_qty (demand during lead time) - this is an INPUT to GRL
        let required_qty = context.oms_data.avg_daily_demand * (context.supplier_data.lead_time as f64);
        
        // Prepare input facts for rule engine
        let mut facts: HashMap<String, Value> = HashMap::new();
        facts.insert("product_id".to_string(), json!(context.oms_data.product_id));
        facts.insert("avg_daily_demand".to_string(), json!(context.oms_data.avg_daily_demand));
        facts.insert("trend".to_string(), json!(context.oms_data.trend));
        facts.insert("available_qty".to_string(), json!(context.inventory_data.available_qty));
        facts.insert("reserved_qty".to_string(), json!(context.inventory_data.reserved_qty));
        facts.insert("warehouse_id".to_string(), json!(context.inventory_data.warehouse_id));
        facts.insert("supplier_id".to_string(), json!(context.supplier_data.supplier_id));
        facts.insert("moq".to_string(), json!(context.supplier_data.moq));
        facts.insert("lead_time_days".to_string(), json!(context.supplier_data.lead_time));
        facts.insert("unit_price".to_string(), json!(context.supplier_data.unit_price));
        facts.insert("conversion_factor".to_string(), json!(context.uom_data.conversion_factor));
        facts.insert("from_uom".to_string(), json!(context.uom_data.from_uom));
        facts.insert("to_uom".to_string(), json!(context.uom_data.to_uom));
        
        // Required qty - calculated INPUT field (not output!)
        facts.insert("required_qty".to_string(), json!(required_qty));
        
        // Supplier status
        facts.insert("is_active".to_string(), json!(true));
        
        // Initialize output fields
        facts.insert("shortage".to_string(), json!(0.0));
        facts.insert("order_qty".to_string(), json!(0.0));
        facts.insert("total_amount".to_string(), json!(0.0));
        facts.insert("need_reorder".to_string(), json!(false));
        facts.insert("supplier_error".to_string(), json!(""));
        
        tracing::info!("📋 Input facts prepared: required_qty={}, available_qty={}, moq={}, unit_price={}", 
            required_qty, context.inventory_data.available_qty, context.supplier_data.moq, context.supplier_data.unit_price);
        
        // Evaluate rules (needs mutable access to engine)
        let mut engine = self.engine.lock().unwrap();
        let result = engine.evaluate(&facts)?;
        
        // Extract results from GRL evaluation
        let need_reorder = result.get("need_reorder")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let order_qty = result.get("order_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let shortage = result.get("shortage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
            
        let total_amount = result.get("total_amount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        // Log detailed results
        tracing::info!("📋 GRL Rules Evaluated:");
        tracing::info!("   ├── Required Qty (Demand): {}", required_qty);
        tracing::info!("   ├── Available Qty: {}", context.inventory_data.available_qty);
        tracing::info!("   ├── Shortage: {}", shortage);
        tracing::info!("   ├── Order Qty: {}", order_qty);
        tracing::info!("   ├── Total Amount: ${:.2}", total_amount);
        tracing::info!("   └── Need Reorder: {}", need_reorder);
        
        if !need_reorder || order_qty == 0.0 {
            tracing::info!("✅ No order needed: sufficient inventory or no shortage");
            return Ok(RuleEvaluationResult {
                should_order: false,
                recommended_qty: 0.0,
                reason: "No order needed: sufficient inventory".to_string(),
            });
        }
        
        tracing::info!("✅ Order recommended by GRL rules");
        tracing::info!("📋 LOG: Shortage detected: {} units", shortage);
        tracing::info!("📋 LOG: Order quantity determined: {} units", order_qty);
        tracing::info!("📋 LOG: Total cost: ${:.2}", total_amount);

        Ok(RuleEvaluationResult {
            should_order: true,
            recommended_qty: order_qty,
            reason: format!("Order approved: calculated {} units", order_qty),
        })
    }
}
