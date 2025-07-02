//! Health check controller for providing system health status endpoints.
//! 
//! This module implements HTTP endpoints for monitoring system health,
//! supporting both simple query-based and detailed JSON-based health checks.

use actix_web::{web, HttpResponse, Responder, Result};
use crate::services::HealthService;
use tracing::instrument;
use crate::dtos::{HealthCheckResponseDto, HealthQuery, HealthCheckRequestDto};

/// HTTP controller for system health monitoring endpoints.
/// 
/// Provides RESTful endpoints for checking system health status,
/// supporting both lightweight query-based checks and comprehensive
/// JSON-based health assessments.
pub struct HealthController;

impl HealthController {
    /// Performs a basic health check with optional service filtering.
    /// 
    /// This endpoint provides a lightweight health check that can optionally
    /// filter results by specific service components using query parameters.
    /// 
    /// # Arguments
    /// 
    /// * `query` - Optional query parameters for service-specific filtering
    /// 
    /// # Returns
    /// 
    /// JSON response containing system health status and component details
    /// 
    /// # Example
    /// 
    /// ```
    /// GET /health?service=database
    /// GET /health (checks all services)
    /// ```
    #[instrument]
    pub async fn health_check(query: web::Query<HealthQuery>) -> Result<impl Responder> {
        let request = HealthCheckRequestDto { 
            service: query.service.clone() 
        };
        let response = HealthService::check_health(request).await;
        Ok(HttpResponse::Ok().json(response))
    }

    /// Performs a detailed health check using JSON request body.
    /// 
    /// This endpoint accepts comprehensive health check requests via JSON payload,
    /// allowing for more complex health assessment configurations and parameters.
    /// 
    /// # Arguments
    /// 
    /// * `req` - JSON request body containing health check parameters
    /// 
    /// # Returns
    /// 
    /// JSON response with detailed system health status and metrics
    /// 
    /// # Example Request Body
    /// 
    /// ```json
    /// {
    ///   "service": "database"
    /// }
    /// ```
    #[instrument]
    pub async fn detailed_health_check(req: web::Json<HealthCheckRequestDto>) -> Result<impl Responder> {
        let response = HealthService::check_health(req.into_inner()).await;
        Ok(HttpResponse::Ok().json(response))
    }
}

/// Basic health check endpoint with OpenAPI documentation
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

/// Detailed health check endpoint with OpenAPI documentation
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
pub async fn detailed_health_check_api(req: web::Json<HealthCheckRequestDto>) -> Result<impl Responder> {
    HealthController::detailed_health_check(req).await
}
