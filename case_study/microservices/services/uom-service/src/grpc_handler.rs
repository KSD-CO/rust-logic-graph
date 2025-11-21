use crate::repository::{UomRepository, PgUomRepository};
use tonic::{Request, Response, Status};

pub mod uom {
    tonic::include_proto!("uom");
}

use uom::uom_service_server::UomService;
use uom::{ConversionRequest, ConversionResponse, HealthRequest, HealthResponse};

/// gRPC service handler
pub struct UomGrpcService {
    repository: PgUomRepository,
}

impl UomGrpcService {
    pub fn new(repository: PgUomRepository) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl UomService for UomGrpcService {
    async fn get_conversion(
        &self,
        request: Request<ConversionRequest>,
    ) -> Result<Response<ConversionResponse>, Status> {
        let product_id = request.into_inner().product_id;
        tracing::info!("[gRPC] Fetching UOM conversions for product: {}", product_id);

        let conversion = self.repository
            .get_conversions(&product_id)
            .await
            .map_err(|e| {
                tracing::error!("[gRPC] Database error: {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        let response = ConversionResponse {
            product_id: conversion.product_id,
            base_unit: conversion.from_uom,
            case_qty: conversion.conversion_factor as i32,
            pallet_qty: (conversion.conversion_factor * 10.0) as i32, // Example multiplier
        };

        tracing::info!("[gRPC] UOM data retrieved: {:?}", response);
        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "healthy".to_string(),
            service: "uom-service".to_string(),
        }))
    }
}
