use sqlx::{Pool, Postgres, Row};

/// OMS data model
#[derive(Debug, Clone)]
pub struct OmsHistory {
    pub product_id: String,
    pub avg_daily_demand: f64,
    pub trend: String,
}

/// OMS repository trait
#[async_trait::async_trait]
pub trait OmsRepository {
    async fn get_history(&self, product_id: &str) -> Result<OmsHistory, sqlx::Error>;
}

/// PostgreSQL implementation of OMS repository
#[derive(Clone)]
pub struct PgOmsRepository {
    pool: Pool<Postgres>,
}

impl PgOmsRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OmsRepository for PgOmsRepository {
    async fn get_history(&self, product_id: &str) -> Result<OmsHistory, sqlx::Error> {
        let row = sqlx::query(
            "SELECT product_id, avg_daily_demand::DOUBLE PRECISION as avg_daily_demand, trend
             FROM oms_history
             WHERE product_id = $1"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(OmsHistory {
            product_id: row.get("product_id"),
            avg_daily_demand: row.get("avg_daily_demand"),
            trend: row.get("trend"),
        })
    }
}
