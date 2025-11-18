use crate::models::{PurchasingContext, RuleEvaluationResult};
use anyhow::Result;
use rust_logic_graph::rete::{IncrementalEngine, TemplateBuilder};
use rust_logic_graph::grl::GrlReteLoader;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct RuleEngineService {
    engine: Arc<Mutex<IncrementalEngine>>,
}

impl RuleEngineService {
    pub fn new(rule_file_path: &str) -> Result<Self> {
        let mut engine = IncrementalEngine::new();

        // Create template for purchasing data
        let purchasing_template = TemplateBuilder::new("PurchasingData")
            .float_field("avg_daily_demand")
            .string_field("trend")
            .float_field("available_qty")
            .float_field("reserved_qty")
            .float_field("moq")
            .integer_field("lead_time")
            .float_field("unit_price")
            .build();

        engine.templates_mut().register(purchasing_template);

        // Load rules from GRL file
        GrlReteLoader::load_from_file(rule_file_path, &mut engine)?;

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
        })
    }

    pub fn evaluate(&self, context: &PurchasingContext) -> Result<RuleEvaluationResult> {
        let mut engine = self.engine.lock();

        // Prepare facts
        let mut facts = std::collections::HashMap::new();
        facts.insert("avg_daily_demand".to_string(), context.oms_data.avg_daily_demand.into());
        facts.insert("trend".to_string(), context.oms_data.trend.clone().into());
        facts.insert("available_qty".to_string(), context.inventory_data.available_qty.into());
        facts.insert("reserved_qty".to_string(), context.inventory_data.reserved_qty.into());
        facts.insert("moq".to_string(), context.supplier_data.moq.into());
        facts.insert("lead_time".to_string(), (context.supplier_data.lead_time as i64).into());
        facts.insert("unit_price".to_string(), context.supplier_data.unit_price.into());

        // Insert facts and fire rules
        let _handle = engine.insert_with_template("PurchasingData", facts)?;
        engine.reset();
        let fired_count = engine.fire_all();

        // Extract results
        let should_order = fired_count > 0;
        let recommended_qty = if should_order {
            self.calculate_order_qty(context)
        } else {
            0.0
        };

        Ok(RuleEvaluationResult {
            should_order,
            recommended_qty,
            reason: format!("{} rules fired", fired_count),
        })
    }

    fn calculate_order_qty(&self, context: &PurchasingContext) -> f64 {
        let safety_stock = context.oms_data.avg_daily_demand * context.supplier_data.lead_time as f64;
        let reorder_point = safety_stock * 1.5;
        let current_available = context.inventory_data.available_qty - context.inventory_data.reserved_qty;

        if current_available < reorder_point {
            let order_qty = (context.oms_data.avg_daily_demand * (context.supplier_data.lead_time as f64 + 7.0)) - current_available;
            order_qty.max(context.supplier_data.moq)
        } else {
            0.0
        }
    }
}
