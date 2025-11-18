use crate::models::UomConversionData;
use anyhow::Result;
use sqlx::{MySqlPool, Row};

pub struct UomService {
    pool: MySqlPool,
}

impl UomService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn get_uom_conversion(&self, product_id: &str) -> Result<UomConversionData> {
        let row = sqlx::query(
            "SELECT product_id, base_unit, case_qty, pallet_qty FROM uom_conversion WHERE product_id = ?"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(UomConversionData {
            product_id: row.get("product_id"),
            base_unit: row.get("base_unit"),
            case_qty: row.get("case_qty"),
            pallet_qty: row.get("pallet_qty"),
        })
    }
}
