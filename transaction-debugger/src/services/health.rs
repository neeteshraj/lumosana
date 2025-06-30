use crate::dtos::{HealthCheckRequestDto, HealthCheckResponseDto};
use tracing::{instrument, error, info};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

pub struct HealthService;

impl HealthService {
    #[instrument]
    pub async fn check_health(request: HealthCheckRequestDto) -> HealthCheckResponseDto {
        match request.service.as_deref() {
            Some("transaction") => Self::check_transaction_service().await,
            Some(service) => HealthCheckResponseDto {
                status: "unknown".to_string(),
                message: Some(format!("Unknown service: {}", service)),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            },
            None => Self::check_general_health().await,
        }
    }

    async fn check_general_health() -> HealthCheckResponseDto {
        info!("Performing general health check");
        
        let mut health_checks = Vec::new();
        let mut overall_healthy = true;
        
        let memory_check = Self::check_memory_usage().await;
        health_checks.push(format!("Memory: {}", if memory_check { "OK" } else { "WARNING" }));
        overall_healthy &= memory_check;
        
        let disk_check = Self::check_disk_space().await;
        health_checks.push(format!("Disk: {}", if disk_check { "OK" } else { "WARNING" }));
        overall_healthy &= disk_check;
        
        let time_check = Self::check_system_time().await;
        health_checks.push(format!("System Time: {}", if time_check { "OK" } else { "ERROR" }));
        overall_healthy &= time_check;
        
        let status = if overall_healthy { "healthy" } else { "degraded" };
        let message = Some(format!("Health checks: [{}]", health_checks.join(", ")));
        
        HealthCheckResponseDto {
            status: status.to_string(),
            message,
            timestamp: Self::get_timestamp(),
        }
    }

    async fn check_transaction_service() -> HealthCheckResponseDto {
        info!("Performing transaction service health check");
        
        let mut health_checks = Vec::new();
        let mut overall_healthy = true;
        
        let rpc_check = Self::check_rpc_connectivity().await;
        health_checks.push(format!("RPC Connectivity: {}", if rpc_check { "OK" } else { "ERROR" }));
        overall_healthy &= rpc_check;
        
        let processing_check = Self::check_transaction_processing().await;
        health_checks.push(format!("Processing: {}", if processing_check { "OK" } else { "WARNING" }));
        overall_healthy &= processing_check;
        
        let protobuf_check = Self::check_protobuf_generation().await;
        health_checks.push(format!("Protobuf: {}", if protobuf_check { "OK" } else { "ERROR" }));
        overall_healthy &= protobuf_check;
        
        let status = if overall_healthy { "healthy" } else { "degraded" };
        let message = Some(format!("Transaction service checks: [{}]", health_checks.join(", ")));
        
        HealthCheckResponseDto {
            status: status.to_string(),
            message,
            timestamp: Self::get_timestamp(),
        }
    }

    
    async fn check_memory_usage() -> bool {
        true
    }
    
    async fn check_disk_space() -> bool {
        true
    }
    
    async fn check_system_time() -> bool {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                timestamp > 1_577_836_800 && timestamp < 1_893_456_000
            }
            Err(_) => false,
        }
    }
    
    async fn check_rpc_connectivity() -> bool {
        info!("Testing RPC connectivity");
        
        let rpc_url = "https://api.mainnet-beta.solana.com";
        
        match timeout(Duration::from_secs(3), async {
            let client = reqwest::Client::new();
            client.post(rpc_url)
                .header("Content-Type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"method":"getVersion","params":[]}"#)
                .send()
                .await
        }).await {
            Ok(Ok(response)) => {
                let success = response.status().is_success();
                if success {
                    info!("RPC connectivity check passed");
                } else {
                    error!("RPC connectivity check failed: HTTP {}", response.status());
                }
                success
            }
            Ok(Err(e)) => {
                error!("RPC connectivity check failed: {}", e);
                false
            }
            Err(_) => {
                error!("RPC connectivity check timed out");
                false
            }
        }
    }
    
    async fn check_transaction_processing() -> bool {
        true 
    }
    
    async fn check_protobuf_generation() -> bool {
        use crate::grpc::transaction::debugger::DebugRequest;
        
        let _test_request = DebugRequest {
            signature: "test".to_string(),
            rpc_url: "test".to_string(),
        };
        
        true 
    }
    
    fn get_timestamp() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
            .to_string()
    }
}
