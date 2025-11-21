use sqlx::{Pool, Postgres, Row};

/// Supplier data model
#[derive(Debug, Clone)]
pub struct SupplierInfo {
    pub supplier_id: String,
    pub product_id: String,
    pub moq: i32,
    pub lead_time_days: i32,
    pub unit_price: f64,
}

/// Supplier repository trait
#[async_trait::async_trait]
pub trait SupplierRepository {
    async fn get_info(&self, product_id: &str) -> Result<SupplierInfo, sqlx::Error>;
}

/// PostgreSQL implementation of Supplier repository
#[derive(Clone)]
pub struct PgSupplierRepository {
    pool: Pool<Postgres>,
}

impl PgSupplierRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SupplierRepository for PgSupplierRepository {
    async fn get_info(&self, product_id: &str) -> Result<SupplierInfo, sqlx::Error> {
        let row = sqlx::query(
            "SELECT supplier_id, product_id,
                    CAST(moq AS INTEGER) as moq,
                    lead_time_days,
                    unit_price::DOUBLE PRECISION as unit_price
             FROM supplier_info
             WHERE product_id = $1 AND is_active = TRUE"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(SupplierInfo {
            supplier_id: row.get("supplier_id"),
            product_id: row.get("product_id"),
            moq: row.get("moq"),
            lead_time_days: row.get("lead_time_days"),
            unit_price: row.get("unit_price"),
        })
    }
}
