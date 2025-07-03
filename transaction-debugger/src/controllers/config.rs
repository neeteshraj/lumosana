use crate::config::Config;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::sync::Arc;

pub async fn get_config_summary(config: web::Data<Arc<Config>>) -> Result<HttpResponse> {
    let summary = serde_json::json!({
        "app": {
            "name": config.app.name,
            "version": config.app.version,
            "environment": config.app.environment
        },
        "server": {
            "http_port": config.server.http_port,
            "grpc_port": config.server.grpc_port,
            "host": config.server.host
        },
        "observability": {
            "metrics_enabled": config.observability.enable_metrics,
            "tracing_enabled": config.observability.enable_tracing,
            "logging_enabled": config.observability.enable_logging
        }
    });

    Ok(HttpResponse::Ok().json(summary))
}

pub async fn get_config_detailed(config: web::Data<Arc<Config>>) -> Result<HttpResponse> {
    match config.to_json() {
        Ok(json_str) => {
            let json_value: serde_json::Value = serde_json::from_str(&json_str).map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to parse config JSON")
            })?;
            Ok(HttpResponse::Ok().json(json_value))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to serialize configuration"
        }))),
    }
}

pub async fn download_config(
    config: web::Data<Arc<Config>>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let environment = &config.app.environment;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}-config-{}.json", environment, timestamp);

    match config.to_json() {
        Ok(json_str) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            ))
            .body(json_str)),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to serialize configuration for download"
        }))),
    }
}

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/config")
            .route("/summary", web::get().to(get_config_summary))
            .route("/detailed", web::get().to(get_config_detailed))
            .route("/download", web::get().to(download_config)),
    );
}
