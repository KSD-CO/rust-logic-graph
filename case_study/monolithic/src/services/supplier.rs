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
            "SELECT product_id, moq, lead_time, unit_price FROM supplier WHERE product_id = ?"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(SupplierData {
            product_id: row.get("product_id"),
            moq: row.get("moq"),
            lead_time: row.get("lead_time"),
            unit_price: row.get("unit_price"),
        })
    }
}
