use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            user: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            name: env::var("DB_NAME").unwrap_or_else(|_| "supplier_db".to_string()),
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub rest_port: u16,
    pub grpc_port: u16,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        Self {
            rest_port: env::var("PORT")
                .unwrap_or_else(|_| "8083".to_string())
                .parse()
                .unwrap_or(8083),
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50053".to_string())
                .parse()
                .unwrap_or(50053),
        }
    }

    pub fn rest_addr(&self) -> String {
        format!("0.0.0.0:{}", self.rest_port)
    }

    pub fn grpc_addr(&self) -> String {
        format!("0.0.0.0:{}", self.grpc_port)
    }
}
