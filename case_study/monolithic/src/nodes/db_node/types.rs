use sqlx::{MySqlPool, PgPool};
use std::sync::Arc;

/// Database pool wrapper supporting both MySQL and Postgres
#[derive(Clone)]
pub enum DatabasePool {
    MySql(Arc<MySqlPool>),
    Postgres(Arc<PgPool>),
}

impl DatabasePool {
    pub fn from_mysql(pool: MySqlPool) -> Self {
        Self::MySql(Arc::new(pool))
    }
    
    pub fn from_postgres(pool: PgPool) -> Self {
        Self::Postgres(Arc::new(pool))
    }
}
