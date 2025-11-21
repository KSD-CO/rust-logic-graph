use serde::{Deserialize, Serialize};

/// Purchase Order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub product_id: String,
    pub order_qty: f64,
    pub order_unit: String,
    pub supplier_id: String,
    pub expected_delivery_date: String,
    pub total_cost: f64,
}
