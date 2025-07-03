use crate::dtos::HealthCheckRequestDto;
use crate::services::HealthService;
use tonic::{Request, Response, Status};
use tracing::instrument;

pub mod debugger {
    tonic::include_proto!("debugger");
}

use debugger::health_check_server::HealthCheck;
use debugger::{health_check_response::ServingStatus, HealthCheckRequest, HealthCheckResponse};

#[derive(Debug)]
pub struct HealthCheckService;

#[tonic::async_trait]
impl HealthCheck for HealthCheckService {
    #[instrument(skip(self))]
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let req = request.into_inner();

        let health_request = HealthCheckRequestDto {
            service: if req.service.is_empty() {
                None
            } else {
                Some(req.service)
            },
        };

        let health_response = HealthService::check_health(health_request).await;

        let status = match health_response.status.as_str() {
            "healthy" => ServingStatus::Serving,
            "unknown" => ServingStatus::ServiceUnknown,
            _ => ServingStatus::NotServing,
        };

        let response = HealthCheckResponse {
            status: status.into(),
        };

        Ok(Response::new(response))
    }
}
