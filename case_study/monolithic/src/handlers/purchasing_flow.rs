use crate::models::{PurchaseOrder, PurchasingContext};
use crate::services::{
    InventoryService, OmsService, RuleEngineService, SupplierService, UomService,
};
use anyhow::Result;
use chrono::{Duration, Utc};

pub struct PurchasingFlowHandler {
    oms_service: OmsService,
    inventory_service: InventoryService,
    supplier_service: SupplierService,
    uom_service: UomService,
    rule_engine: RuleEngineService,
}

impl PurchasingFlowHandler {
    pub fn new(
        oms_service: OmsService,
        inventory_service: InventoryService,
        supplier_service: SupplierService,
        uom_service: UomService,
        rule_engine: RuleEngineService,
    ) -> Self {
        Self {
            oms_service,
            inventory_service,
            supplier_service,
            uom_service,
            rule_engine,
        }
    }

    pub async fn execute(&self, product_id: &str) -> Result<Option<PurchaseOrder>> {
        println!("🔄 Starting purchasing flow for product: {}", product_id);

        // Step 1: Fetch data from all services in parallel
        let (oms_data, inventory_data, supplier_data, uom_data) = tokio::try_join!(
            self.oms_service.get_history(product_id),
            self.inventory_service.get_inventory(product_id),
            self.supplier_service.get_supplier_info(product_id),
            self.uom_service.get_uom_conversion(product_id),
        )?;

        println!("✅ Data fetched successfully");
        println!("   OMS: avg_daily_demand={}, trend={}", oms_data.avg_daily_demand, oms_data.trend);
        println!("   Inventory: available={}, reserved={}", inventory_data.available_qty, inventory_data.reserved_qty);
        println!("   Supplier: MOQ={}, lead_time={} days, price={}", supplier_data.moq, supplier_data.lead_time, supplier_data.unit_price);

        // Step 2: Create purchasing context
        let context = PurchasingContext {
            oms_data,
            inventory_data,
            supplier_data,
            uom_data,
        };

        // Step 3: Evaluate business rules
        println!("🔍 Evaluating business rules...");
        let rule_result = self.rule_engine.evaluate(&context)?;

        if !rule_result.should_order {
            println!("❌ No order needed: {}", rule_result.reason);
            return Ok(None);
        }

        println!("✅ Order approved: {}", rule_result.reason);

        // Step 4: Create purchase order
        let expected_delivery_date = Utc::now()
            + Duration::days(context.supplier_data.lead_time as i64);

        let purchase_order = PurchaseOrder {
            product_id: context.oms_data.product_id.clone(),
            order_qty: rule_result.recommended_qty,
            order_unit: context.uom_data.base_unit.clone(),
            supplier_id: format!("SUPP-{}", context.supplier_data.product_id),
            expected_delivery_date: expected_delivery_date.format("%Y-%m-%d").to_string(),
            total_cost: rule_result.recommended_qty * context.supplier_data.unit_price,
        };

        println!("📦 Purchase Order Created:");
        println!("   Quantity: {} {}", purchase_order.order_qty, purchase_order.order_unit);
        println!("   Total Cost: ${:.2}", purchase_order.total_cost);
        println!("   Expected Delivery: {}", purchase_order.expected_delivery_date);

        Ok(Some(purchase_order))
    }
}
