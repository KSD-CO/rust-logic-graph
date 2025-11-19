use crate::models::SupplierData;
use anyhow::Result;
use sqlx::{MySqlPool, Row};

pub struct SupplierService {
    pool: MySqlPool,
}

impl SupplierService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn get_supplier_info(&self, product_id: &str) -> Result<SupplierData> {
        let row = sqlx::query(
            "SELECT supplier_id, product_id, CAST(moq AS DOUBLE) as moq, lead_time_days, CAST(unit_price AS DOUBLE) as unit_price FROM supplier_info WHERE product_id = ? AND is_active = TRUE"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(SupplierData {
            supplier_id: row.get("supplier_id"),
            product_id: row.get("product_id"),
            moq: row.get("moq"),
            lead_time: row.get("lead_time_days"),
            unit_price: row.get("unit_price"),
        })
    }
}
