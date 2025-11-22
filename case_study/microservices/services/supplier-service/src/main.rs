mod config;
mod repository;
mod grpc_handler;
mod rest_handler;

use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use config::{DatabaseConfig, ServerConfig};
use repository::PgSupplierRepository;
use grpc_handler::{supplier::supplier_service_server::SupplierServiceServer, SupplierGrpcService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    let _ = dotenvy::dotenv();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    // Load configurations
    let db_config = DatabaseConfig::from_env();
    let server_config = ServerConfig::from_env();

    // Connect to database
    tracing::info!("Connecting to database: {}", db_config.connection_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_config.connection_string())
        .await?;
    tracing::info!("Database connected successfully");

    // Create repository
    let repository = Arc::new(PgSupplierRepository::new(pool));

    // Start gRPC server
    let grpc_repository = repository.as_ref().clone();
    let grpc_addr = server_config.grpc_addr().parse()?;
    let grpc_service = SupplierGrpcService::new(grpc_repository);
    let grpc_server = Server::builder()
        .add_service(SupplierServiceServer::new(grpc_service))
        .serve(grpc_addr);
    tracing::info!("🚀 gRPC server starting on {}", grpc_addr);

    // Start REST server
    let rest_app = rest_handler::create_router(repository);
    let rest_addr = server_config.rest_addr();
    let listener = TcpListener::bind(&rest_addr).await?;
    tracing::info!("🚀 REST server starting on {}", rest_addr);

    // Run both servers concurrently
    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        result = axum::serve(listener, rest_app) => {
            if let Err(e) = result {
                tracing::error!("REST server error: {}", e);
            }
        }
    }

    Ok(())
}
