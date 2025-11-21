use std::env;

/// Database configuration for a single database server
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    /// Create config from environment variables with prefix
    /// Example: prefix="OMS_DB" reads OMS_DB_HOST, OMS_DB_PORT, etc.
    pub fn from_env_prefix(prefix: &str) -> Self {
        let host_key = format!("{}_HOST", prefix);
        let port_key = format!("{}_PORT", prefix);
        let user_key = format!("{}_USER", prefix);
        let pass_key = format!("{}_PASSWORD", prefix);
        let name_key = format!("{}_NAME", prefix);
        
        Self {
            host: env::var(&host_key).unwrap_or_else(|_| "localhost".to_string()),
            port: env::var(&port_key)
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            user: env::var(&user_key).unwrap_or_else(|_| "postgres".to_string()),
            password: env::var(&pass_key).unwrap_or_else(|_| "postgres".to_string()),
            database: env::var(&name_key).unwrap_or_else(|_| "postgres".to_string()),
        }
    }
    
    /// Create default config from DB_* environment variables
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            user: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
            database: "postgres".to_string(),
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

/// Application configuration with multiple database servers
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub oms_db: DatabaseConfig,
    pub inventory_db: DatabaseConfig,
    pub supplier_db: DatabaseConfig,
    pub uom_db: DatabaseConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            oms_db: DatabaseConfig::from_env_prefix("OMS_DB"),
            inventory_db: DatabaseConfig::from_env_prefix("INVENTORY_DB"),
            supplier_db: DatabaseConfig::from_env_prefix("SUPPLIER_DB"),
            uom_db: DatabaseConfig::from_env_prefix("UOM_DB"),
        }
    }
}
