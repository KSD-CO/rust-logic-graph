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
            "SELECT product_id, from_uom, to_uom, CAST(conversion_factor AS DOUBLE) as conversion_factor FROM uom_conversion WHERE product_id = ?"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(UomConversionData {
            product_id: row.get("product_id"),
            from_uom: row.get("from_uom"),
            to_uom: row.get("to_uom"),
            conversion_factor: row.get("conversion_factor"),
        })
    }
}
