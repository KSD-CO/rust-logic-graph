use anyhow::Result;
use sqlx::{MySqlPool, PgPool};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;

pub async fn create_mysql_pool(connection_string: &str) -> Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await?;

    Ok(pool)
}

pub async fn create_postgres_pool(connection_string: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await?;

    Ok(pool)
}

pub async fn create_pool(connection_string: &str) -> Result<MySqlPool> {
    create_mysql_pool(connection_string).await
}
