use serde::{Deserialize, Serialize};

// OMS Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmsHistoryData {
    pub product_id: String,
    pub avg_daily_demand: f64,
    pub trend: String, // "stable", "increasing", "decreasing"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmsHistoryResponse {
    pub data: OmsHistoryData,
}

// Inventory Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryData {
    pub product_id: String,
    pub warehouse_id: String,
    pub current_qty: i32,
    pub reserved_qty: i32,
    pub available_qty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryResponse {
    pub data: InventoryData,
}

// Supplier Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierData {
    pub supplier_id: String,
    pub product_id: String,
    pub moq: i32,
    pub lead_time_days: i32,
    pub unit_price: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierResponse {
    pub data: SupplierData,
}

// UOM Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UomConversionData {
    pub product_id: String,
    pub from_uom: String,
    pub to_uom: String,
    pub conversion_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UomConversionResponse {
    pub data: UomConversionData,
}

// Rule Engine Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineRequest {
    pub oms_data: OmsHistoryData,
    pub inventory_data: InventoryData,
    pub supplier_data: SupplierData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineResponse {
    pub need_reorder: bool,
    pub shortage: f64,
    pub order_qty: i64,
    pub total_amount: f64,
    pub requires_approval: bool,
    pub approval_status: String,
}

// Purchase Order Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePORequest {
    pub product_id: String,
    pub supplier_id: String,
    pub qty: i64,
    pub unit_price: f64,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub po_id: String,
    pub product_id: String,
    pub supplier_id: String,
    pub qty: i64,
    pub unit_price: f64,
    pub total_amount: f64,
    pub status: String, // "draft", "sent", "confirmed", "cancelled"
    pub created_at: String,
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct POResponse {
    pub po: PurchaseOrder,
}

// Orchestrator Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchasingFlowRequest {
    pub product_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchasingFlowResponse {
    pub success: bool,
    pub po: Option<PurchaseOrder>,
    pub calculation: Option<RuleEngineResponse>,
    pub message: String,
}

// Health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
    pub version: String,
}
