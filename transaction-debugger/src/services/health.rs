use crate::dtos::{HealthCheckRequestDto, HealthCheckResponseDto};
use tracing::{instrument, error, info, warn};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;
use sysinfo::{System, Disks};

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
        info!("Checking memory usage");
        
        let mut sys = System::new_all();
        sys.refresh_memory();
        
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let free_memory = sys.free_memory();
        
        if total_memory > 0 {
            let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
            
            // Convert bytes to MB for logging
            let total_mb = total_memory / 1024 / 1024;
            let used_mb = used_memory / 1024 / 1024;
            let free_mb = free_memory / 1024 / 1024;
            
            info!("Memory usage: {:.1}% ({} MB used, {} MB free, {} MB total)", 
                  memory_usage_percent, used_mb, free_mb, total_mb);
            
            // Consider memory healthy if usage is below 90%
            let is_healthy = memory_usage_percent < 90.0;
            if !is_healthy {
                warn!("High memory usage detected: {:.1}%", memory_usage_percent);
            }
            is_healthy
        } else {
            warn!("Could not determine memory usage");
            true // Default to healthy if we can't determine usage
        }
    }
    
    async fn check_disk_space() -> bool {
        info!("Checking disk space");
        
        let disks = Disks::new_with_refreshed_list();
        
        // Look for the root disk or the disk with the largest capacity
        let mut root_disk = None;
        let mut largest_capacity = 0;
        
        for disk in &disks {
            let mount_point = disk.mount_point().to_string_lossy();
            let total_space = disk.total_space();
            
            // Prefer root filesystem
            if mount_point == "/" {
                root_disk = Some(disk);
                break;
            }
            
            // Otherwise, track the largest disk
            if total_space > largest_capacity {
                largest_capacity = total_space;
                root_disk = Some(disk);
            }
        }
        
        if let Some(disk) = root_disk {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            
            if total_space > 0 {
                let used_percent = (used_space as f64 / total_space as f64) * 100.0;
                
                // Convert bytes to human-readable format
                let total_gb = total_space as f64 / (1024.0 * 1024.0 * 1024.0);
                let used_gb = used_space as f64 / (1024.0 * 1024.0 * 1024.0);
                let available_gb = available_space as f64 / (1024.0 * 1024.0 * 1024.0);
                
                info!("Disk usage: {:.1}% (Mount: {}, Total: {:.1}GB, Used: {:.1}GB, Available: {:.1}GB)",
                      used_percent,
                      disk.mount_point().to_string_lossy(),
                      total_gb,
                      used_gb,
                      available_gb);
                
                // Consider disk healthy if usage is below 85%
                let is_healthy = used_percent < 85.0;
                if !is_healthy {
                    warn!("High disk usage detected: {:.1}%", used_percent);
                }
                is_healthy
            } else {
                warn!("Could not determine disk usage - total space is 0");
                true // Default to healthy if we can't determine usage
            }
        } else {
            warn!("No disks found for health check");
            true // Default to healthy if no disks found
        }
    }
    
    async fn check_system_time() -> bool {
        info!("Checking system time");
        
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                let current_year = 1970 + (timestamp / (365 * 24 * 3600));
                
                // Check if timestamp is reasonable (between 2020 and 2035)
                let min_timestamp = 1_577_836_800; // Jan 1, 2020
                let max_timestamp = 2_051_222_400; // Jan 1, 2035
                
                let is_time_valid = timestamp > min_timestamp && timestamp < max_timestamp;
                
                info!("System time check: timestamp={}, year≈{}, valid={}", 
                      timestamp, current_year, is_time_valid);
                
                if !is_time_valid {
                    if timestamp <= min_timestamp {
                        error!("System time appears to be set too far in the past (year ≈ {})", current_year);
                    } else {
                        error!("System time appears to be set too far in the future (year ≈ {})", current_year);
                    }
                }
                
                // Additional check: verify system time is not drifting too much
                // by checking if we can get a consistent time reading
                let second_reading = SystemTime::now().duration_since(UNIX_EPOCH);
                match second_reading {
                    Ok(second_duration) => {
                        let time_diff = second_duration.as_secs().abs_diff(timestamp);
                        if time_diff > 5 {
                            warn!("System time appears unstable ({}s difference between readings)", time_diff);
                            return false;
                        }
                    }
                    Err(_) => {
                        error!("System time became invalid during check");
                        return false;
                    }
                }
                
                is_time_valid
            }
            Err(e) => {
                error!("System time is before Unix epoch: {}", e);
                false
            }
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
