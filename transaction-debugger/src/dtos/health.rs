use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheckRequestDto {
    pub service: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthCheckResponseDto {
    pub status: String,
    pub message: Option<String>,
    pub timestamp: String,
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
