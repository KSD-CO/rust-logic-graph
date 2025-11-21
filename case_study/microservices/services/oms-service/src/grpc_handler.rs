use crate::repository::{OmsRepository, PgOmsRepository};
use tonic::{Request, Response, Status};

pub mod oms {
    tonic::include_proto!("oms");
}

use oms::oms_service_server::OmsService;
use oms::{HealthRequest, HealthResponse, HistoryRequest, HistoryResponse};

/// gRPC service handler
pub struct OmsGrpcService {
    repository: PgOmsRepository,
}

impl OmsGrpcService {
    pub fn new(repository: PgOmsRepository) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl OmsService for OmsGrpcService {
    async fn get_history(
        &self,
        request: Request<HistoryRequest>,
    ) -> Result<Response<HistoryResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching OMS history for product: {}", product_id);

        let history = self.repository
            .get_history(&product_id)
            .await
            .map_err(|e| {
                tracing::error!("[gRPC] Database error: {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        let response = HistoryResponse {
            product_id: history.product_id,
            avg_daily_demand: history.avg_daily_demand,
            trend: history.trend,
        };

        tracing::info!("[gRPC] OMS data retrieved: {:?}", response);
        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "healthy".to_string(),
            service: "oms-service".to_string(),
        }))
    }
}
