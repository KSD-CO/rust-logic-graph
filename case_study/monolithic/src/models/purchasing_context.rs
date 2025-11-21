use crate::nodes::{OmsHistoryData, InventoryData, SupplierData, UomConversionData};

/// Complete purchasing context
#[derive(Debug, Clone)]
pub struct PurchasingContext {
    pub oms_data: OmsHistoryData,
    pub inventory_data: InventoryData,
    pub supplier_data: SupplierData,
    pub uom_data: UomConversionData,
}
