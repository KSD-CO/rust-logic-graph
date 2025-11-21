use serde::{Deserialize, Serialize};

/// OMS (Order Management System) historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmsHistoryData {
    pub product_id: String,
    pub avg_daily_demand: f64,
    pub trend: String, // "stable", "increasing", "decreasing"
}

/// Inventory data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryData {
    pub product_id: String,
    pub warehouse_id: String,
    pub available_qty: f64,
    pub reserved_qty: f64,
}

/// Supplier information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierData {
    pub supplier_id: String,
    pub product_id: String,
    pub moq: f64,          // Minimum Order Quantity
    pub lead_time: i32,    // Days
    pub unit_price: f64,
}

/// Unit of Measurement conversion data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UomConversionData {
    pub product_id: String,
    pub from_uom: String,
    pub to_uom: String,
    pub conversion_factor: f64,
}
