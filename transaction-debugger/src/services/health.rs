//! Health check service for monitoring system and service component status.
//!
//! This module provides comprehensive health monitoring capabilities including
//! memory usage, disk space, system time validation, and service-specific checks.
//! Uses the sysinfo crate for cross-platform system resource monitoring.

use crate::dtos::{HealthCheckRequestDto, HealthCheckResponseDto};
use crate::utils::{protobuf, rpc, system, time, transaction};
use tracing::{info, instrument};

/// Service for performing comprehensive system and application health checks.
///
/// Provides methods to monitor various system resources, validate service
/// availability, and generate detailed health status reports for monitoring
/// and alerting systems.
pub struct HealthService;

impl HealthService {
    /// Performs health check based on the provided request parameters.
    ///
    /// Routes health check requests to appropriate service-specific or general
    /// health monitoring functions based on the service parameter in the request.
    ///
    /// # Arguments
    ///
    /// * `request` - Health check request specifying target service or general check
    ///
    /// # Returns
    ///
    /// Health check response containing status, message, and timestamp
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

    /// Performs comprehensive system health assessment.
    ///
    /// Executes multiple health checks including memory usage, disk space,
    /// and system time validation. Aggregates results to provide an overall
    /// system health status.
    ///
    /// # Returns
    ///
    /// Aggregated health status with detailed component check results
    async fn check_general_health() -> HealthCheckResponseDto {
        info!("Performing general health check");

        let mut health_checks = Vec::new();
        let mut overall_healthy = true;

        let memory_check = system::check_memory_usage().await;
        health_checks.push(format!(
            "Memory: {}",
            if memory_check { "OK" } else { "WARNING" }
        ));
        overall_healthy &= memory_check;

        let disk_check = system::check_disk_space().await;
        health_checks.push(format!(
            "Disk: {}",
            if disk_check { "OK" } else { "WARNING" }
        ));
        overall_healthy &= disk_check;

        let time_check = system::check_system_time().await;
        health_checks.push(format!(
            "System Time: {}",
            if time_check { "OK" } else { "ERROR" }
        ));
        overall_healthy &= time_check;

        let status = if overall_healthy {
            "healthy"
        } else {
            "degraded"
        };
        let message = Some(format!("Health checks: [{}]", health_checks.join(", ")));

        HealthCheckResponseDto {
            status: status.to_string(),
            message,
            timestamp: time::get_timestamp(),
        }
    }

    /// Performs health check specific to transaction processing services.
    ///
    /// Validates RPC connectivity, transaction processing capabilities,
    /// and protobuf message generation functionality to ensure the
    /// transaction debugging service is fully operational.
    ///
    /// # Returns
    ///
    /// Health status specifically for transaction service components
    async fn check_transaction_service() -> HealthCheckResponseDto {
        info!("Performing transaction service health check");

        let mut health_checks = Vec::new();
        let mut overall_healthy = true;

        let rpc_check = rpc::check_rpc_connectivity().await;
        health_checks.push(format!(
            "RPC Connectivity: {}",
            if rpc_check { "OK" } else { "ERROR" }
        ));
        overall_healthy &= rpc_check;

        let processing_check = transaction::check_transaction_processing().await;
        health_checks.push(format!(
            "Processing: {}",
            if processing_check { "OK" } else { "WARNING" }
        ));
        overall_healthy &= processing_check;

        let protobuf_check = protobuf::check_protobuf_generation().await;
        health_checks.push(format!(
            "Protobuf: {}",
            if protobuf_check { "OK" } else { "ERROR" }
        ));
        overall_healthy &= protobuf_check;

        let status = if overall_healthy {
            "healthy"
        } else {
            "degraded"
        };
        let message = Some(format!(
            "Transaction service checks: [{}]",
            health_checks.join(", ")
        ));

        HealthCheckResponseDto {
            status: status.to_string(),
            message,
            timestamp: time::get_timestamp(),
        }
    }
}
