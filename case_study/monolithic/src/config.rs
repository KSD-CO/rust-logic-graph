use std::env;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "171.244.10.40".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "6033".to_string())
                .parse()
                .unwrap_or(6033),
            user: env::var("DB_USER").unwrap_or_else(|_| "root".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
        }
    }

    pub fn connection_string(&self, database: &str) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, database
        )
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db: DatabaseConfig,
    pub oms_db_name: String,
    pub inventory_db_name: String,
    pub supplier_db_name: String,
    pub uom_db_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db: DatabaseConfig::from_env(),
            oms_db_name: "oms_db".to_string(),
            inventory_db_name: "inventory_db".to_string(),
            supplier_db_name: "supplier_db".to_string(),
            uom_db_name: "uom_db".to_string(),
        }
    }
}
