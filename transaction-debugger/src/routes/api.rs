use actix_web::web;
use crate::controllers::health::{health_check_api, detailed_health_check_api};
use crate::controllers::transaction::debug_transaction_api;
use crate::controllers::config;
use crate::dtos::{DebugRequestDto, DebugResponseDto, HealthCheckRequestDto, HealthCheckResponseDto, HealthQuery, TransactionDetailsDto, InstructionDetailDto, TransactionAnalysisDto};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::transaction::debug_transaction_api,
        crate::controllers::health::health_check_api,
        crate::controllers::health::detailed_health_check_api
    ),
    components(
        schemas(DebugRequestDto, DebugResponseDto, HealthCheckRequestDto, HealthCheckResponseDto, HealthQuery, TransactionDetailsDto, InstructionDetailDto, TransactionAnalysisDto)
    ),
    tags(
        (name = "transaction", description = "Solana transaction debugging and analysis"),
        (name = "health", description = "System health monitoring endpoints"),
        (name = "config", description = "Configuration management endpoints")
    ),
    info(
        title = "Solana Transaction Debugger API",
        version = "1.0.0",
        description = "A comprehensive API for analyzing and debugging Solana transactions",
        contact(
            name = "Solana Transaction Debugger",
            email = "support@lumosana.com"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.solana-debugger.com", description = "Production server")
    )
)]
pub struct ApiDoc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .route("/api-docs/openapi.json", web::get().to(|| async {
            actix_web::HttpResponse::Ok().json(ApiDoc::openapi())
        }))
        
        // API endpoints  
        .route("/debug", web::post().to(debug_transaction_api))
        .route("/health-detailed", web::post().to(detailed_health_check_api))
        .route("/health", web::get().to(health_check_api));
    
    // Add configuration routes
    config::config_routes(cfg);
}
