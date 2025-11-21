use sqlx::{Pool, Postgres, Row};

/// UOM conversion data model
#[derive(Debug, Clone)]
pub struct UomConversion {
    pub product_id: String,
    pub from_uom: String,
    pub to_uom: String,
    pub conversion_factor: f64,
}

/// UOM repository trait
#[async_trait::async_trait]
pub trait UomRepository {
    async fn get_conversions(&self, product_id: &str) -> Result<UomConversion, sqlx::Error>;
}

/// PostgreSQL implementation of UOM repository
#[derive(Clone)]
pub struct PgUomRepository {
    pool: Pool<Postgres>,
}

impl PgUomRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UomRepository for PgUomRepository {
    async fn get_conversions(&self, product_id: &str) -> Result<UomConversion, sqlx::Error> {
        let row = sqlx::query(
            "SELECT product_id, from_uom, to_uom, conversion_factor::DOUBLE PRECISION as conversion_factor
             FROM uom_conversions
             WHERE product_id = $1"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(UomConversion {
            product_id: row.get("product_id"),
            from_uom: row.get("from_uom"),
            to_uom: row.get("to_uom"),
            conversion_factor: row.get("conversion_factor"),
        })
    }
}
