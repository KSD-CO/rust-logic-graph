use sqlx::{Pool, Postgres, Row};

/// Inventory data model
#[derive(Debug, Clone)]
pub struct InventoryLevels {
    pub product_id: String,
    pub warehouse_id: String,
    pub available_qty: i32,
    pub reserved_qty: i32,
}

/// Inventory repository trait
#[async_trait::async_trait]
pub trait InventoryRepository {
    async fn get_levels(&self, product_id: &str) -> Result<InventoryLevels, sqlx::Error>;
}

/// PostgreSQL implementation of Inventory repository
#[derive(Clone)]
pub struct PgInventoryRepository {
    pool: Pool<Postgres>,
}

impl PgInventoryRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl InventoryRepository for PgInventoryRepository {
    async fn get_levels(&self, product_id: &str) -> Result<InventoryLevels, sqlx::Error> {
        let row = sqlx::query(
            "SELECT product_id, warehouse_id,
                    CAST(available_qty AS INTEGER) as available_qty,
                    CAST(reserved_qty AS INTEGER) as reserved_qty
             FROM inventory_levels
             WHERE product_id = $1"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(InventoryLevels {
            product_id: row.get("product_id"),
            warehouse_id: row.get("warehouse_id"),
            available_qty: row.get("available_qty"),
            reserved_qty: row.get("reserved_qty"),
        })
    }
}
