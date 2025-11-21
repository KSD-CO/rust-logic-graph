use crate::repository::{SupplierRepository, PgSupplierRepository};
use tonic::{Request, Response, Status};

pub mod supplier {
    tonic::include_proto!("supplier");
}

use supplier::supplier_service_server::SupplierService;
use supplier::{HealthRequest, HealthResponse, InfoRequest, InfoResponse};

/// gRPC service handler
pub struct SupplierGrpcService {
    repository: PgSupplierRepository,
}

impl SupplierGrpcService {
    pub fn new(repository: PgSupplierRepository) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl SupplierService for SupplierGrpcService {
    async fn get_info(
        &self,
        request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching supplier info for product: {}", product_id);

        let info = self.repository
            .get_info(&product_id)
            .await
            .map_err(|e| {
                tracing::error!("[gRPC] Database error: {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        let response = InfoResponse {
            product_id: info.product_id,
            moq: info.moq,
            lead_time_days: info.lead_time_days,
            unit_price: info.unit_price,
            supplier_name: info.supplier_id, // Map supplier_id to supplier_name
        };

        tracing::info!("[gRPC] Supplier data retrieved: {:?}", response);
        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "healthy".to_string(),
            service: "supplier-service".to_string(),
        }))
    }
}
