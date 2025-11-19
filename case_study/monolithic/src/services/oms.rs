use crate::models::OmsHistoryData;
use anyhow::Result;
use sqlx::{MySqlPool, Row};

pub struct OmsService {
    pool: MySqlPool,
}

impl OmsService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn get_oms_history(&self, product_id: &str) -> Result<OmsHistoryData> {
        let row = sqlx::query(
            "SELECT product_id, CAST(avg_daily_demand AS DOUBLE) as avg_daily_demand, trend FROM oms_history WHERE product_id = ?"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(OmsHistoryData {
            product_id: row.get("product_id"),
            avg_daily_demand: row.get("avg_daily_demand"),
            trend: row.get("trend"),
        })
    }
}
