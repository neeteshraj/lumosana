use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HealthCheckRequestDto {
    /// Optional service name to check (e.g., "transaction", "rpc")
    pub service: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthCheckResponseDto {
    /// Health status ("healthy", "unhealthy", "degraded")
    pub status: String,
    /// Optional status message
    pub message: Option<String>,
    /// Timestamp of the health check
    pub timestamp: String,
}

/// Query parameters for basic health check endpoint.
/// 
/// Allows filtering health checks by specific service components.
#[derive(Deserialize, Debug, ToSchema)]
pub struct HealthQuery {
    /// Optional service name to check specifically (e.g., "database", "cache")
    pub service: Option<String>,
}

impl Default for HealthCheckResponseDto {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            message: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        }
    }
}
