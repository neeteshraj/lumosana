use actix_web::{web, HttpResponse, Responder, Result};
use crate::services::HealthService;
use crate::dtos::HealthCheckRequestDto;
use tracing::instrument;

pub struct HealthController;

impl HealthController {
    #[instrument]
    pub async fn health_check() -> Result<impl Responder> {
        let request = HealthCheckRequestDto { service: None };
        let response = HealthService::check_health(request).await;
        Ok(HttpResponse::Ok().json(response))
    }

    #[instrument]
    pub async fn detailed_health_check(req: web::Json<HealthCheckRequestDto>) -> Result<impl Responder> {
        let response = HealthService::check_health(req.into_inner()).await;
        Ok(HttpResponse::Ok().json(response))
    }
}
