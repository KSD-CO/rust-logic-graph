use crate::repository::{InventoryRepository, PgInventoryRepository};
use tonic::{Request, Response, Status};

pub mod inventory {
    tonic::include_proto!("inventory");
}

use inventory::inventory_service_server::InventoryService;
use inventory::{HealthRequest, HealthResponse, LevelsRequest, LevelsResponse};

/// gRPC service handler
pub struct InventoryGrpcService {
    repository: PgInventoryRepository,
}

impl InventoryGrpcService {
    pub fn new(repository: PgInventoryRepository) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl InventoryService for InventoryGrpcService {
    async fn get_levels(
        &self,
        request: Request<LevelsRequest>,
    ) -> Result<Response<LevelsResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching inventory levels for product: {}", product_id);

        let levels = self.repository
            .get_levels(&product_id)
            .await
            .map_err(|e| {
                tracing::error!("[gRPC] Database error: {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        let response = LevelsResponse {
            product_id: levels.product_id,
            available_qty: levels.available_qty,
            reserved_qty: levels.reserved_qty,
            on_order_qty: 0, // Default value
        };

        tracing::info!("[gRPC] Inventory data retrieved: {:?}", response);
        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "healthy".to_string(),
            service: "inventory-service".to_string(),
        }))
    }
}
