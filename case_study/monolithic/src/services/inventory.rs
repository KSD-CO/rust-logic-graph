use crate::models::InventoryData;
use anyhow::Result;
use sqlx::{MySqlPool, Row};

pub struct InventoryService {
    pool: MySqlPool,
}

impl InventoryService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn get_inventory(&self, product_id: &str) -> Result<InventoryData> {
        let row = sqlx::query(
            "SELECT product_id, available_qty, reserved_qty FROM inventory WHERE product_id = ?"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(InventoryData {
            product_id: row.get("product_id"),
            available_qty: row.get("available_qty"),
            reserved_qty: row.get("reserved_qty"),
        })
    }
}
