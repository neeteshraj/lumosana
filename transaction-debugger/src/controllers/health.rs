use crate::dtos::{HealthCheckRequestDto, HealthCheckResponseDto, HealthQuery};
use crate::services::HealthService;
use actix_web::{web, HttpResponse, Responder, Result};
use once_cell::sync::Lazy;
use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::Counter;
use tracing::{info, instrument, Span, event, Level};

static HEALTH_CHECK_COUNTER: Lazy<Counter<u64>> = Lazy::new(|| {
    global::meter("transaction-debugger")
        .u64_counter("health_check_requests_total")
        .with_description("Total number of health check requests")
        .build()
});

pub struct HealthController;

impl HealthController {
    #[instrument(name = "basic_health_check", skip(query))]
    pub async fn health_check(query: web::Query<HealthQuery>) -> Result<impl Responder> {
        let service_name = query.service.clone().unwrap_or_else(|| "all".to_string());

        Span::current().record("service", &service_name.as_str());

        event!(
            Level::INFO,
            service = %service_name,
            kind = "basic",
            message = "starting basic health check"
        );

        HEALTH_CHECK_COUNTER.add(1, &[
            KeyValue::new("type", "basic"),
            KeyValue::new("service", service_name.clone()),
        ]);

        let request = HealthCheckRequestDto {
            service: query.service.clone(),
        };

        let response = HealthService::check_health(request).await;
        info!("Health check completed for service: {}", service_name);
        Ok(HttpResponse::Ok().json(response))
    }

    #[instrument(name = "detailed_health_check", skip(req))]
    pub async fn detailed_health_check(
        req: web::Json<HealthCheckRequestDto>,
    ) -> Result<impl Responder> {
        let service_name = req.service.clone().unwrap_or_else(|| "all".to_string());

        Span::current().record("service", &service_name.as_str());

        event!(
            Level::INFO,
            service = %service_name,
            kind = "detailed",
            message = "starting detailed health check"
        );

        HEALTH_CHECK_COUNTER.add(1, &[
            KeyValue::new("type", "detailed"),
            KeyValue::new("service", service_name.clone()),
        ]);

        let response = HealthService::check_health(req.into_inner()).await;
        info!("Detailed health check completed for service: {}", service_name);
        Ok(HttpResponse::Ok().json(response))
    }
}

#[utoipa::path(
    get,
    path = "/health",
    params(
        ("service" = Option<String>, Query, description = "Optional service name to check")
    ),
    responses(
        (status = 200, description = "Health check successful", body = HealthCheckResponseDto),
        (status = 503, description = "Service unhealthy")
    ),
    tag = "health"
)]
pub async fn health_check_api(query: web::Query<HealthQuery>) -> Result<impl Responder> {
    HealthController::health_check(query).await
}

#[utoipa::path(
    post,
    path = "/health-detailed",
    request_body = HealthCheckRequestDto,
    responses(
        (status = 200, description = "Detailed health check completed", body = HealthCheckResponseDto),
        (status = 503, description = "Service unhealthy")
    ),
    tag = "health"
)]
pub async fn detailed_health_check_api(
    req: web::Json<HealthCheckRequestDto>,
) -> Result<impl Responder> {
    HealthController::detailed_health_check(req).await
}
