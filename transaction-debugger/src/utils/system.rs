//! System utility functions for performing system-level health checks.
//!
//! This module provides functionality for monitoring system resources such as
//! memory usage and disk space, and for validating system time. These utilities
//! are used by the health check service to provide a comprehensive view of the
//! system's health.

use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, System};
use tracing::{error, info, warn};

/// Monitors system memory usage and validates availability.
///
/// Checks available memory against threshold values to ensure
/// the system has sufficient memory for optimal operation.
///
/// # Returns
///
/// `true` if memory usage is within acceptable limits, `false` otherwise
pub async fn check_memory_usage() -> bool {
    info!("Checking memory usage");

    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let free_memory = sys.free_memory();

    if total_memory > 0 {
        let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

        let total_mb = total_memory / 1024 / 1024;
        let used_mb = used_memory / 1024 / 1024;
        let free_mb = free_memory / 1024 / 1024;

        info!(
            "Memory usage: {:.1}% ({} MB used, {} MB free, {} MB total)",
            memory_usage_percent, used_mb, free_mb, total_mb
        );

        let is_healthy = memory_usage_percent < 90.0;
        if !is_healthy {
            warn!("High memory usage detected: {:.1}%", memory_usage_percent);
        }
        is_healthy
    } else {
        warn!("Could not determine memory usage");
        true
    }
}

/// Monitors disk space usage across mounted filesystems.
///
/// Checks available disk space, prioritizing the root filesystem,
/// and validates that sufficient space is available for continued
/// operation and log file generation.
///
/// # Returns
///
/// `true` if disk space is within acceptable limits, `false` otherwise
pub async fn check_disk_space() -> bool {
    info!("Checking disk space");

    let disks = Disks::new_with_refreshed_list();

    let mut root_disk = None;
    let mut largest_capacity = 0;

    for disk in &disks {
        let mount_point = disk.mount_point().to_string_lossy();
        let total_space = disk.total_space();

        if mount_point == "/" {
            root_disk = Some(disk);
            break;
        }

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

            let total_gb = total_space as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = used_space as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = available_space as f64 / (1024.0 * 1024.0 * 1024.0);

            info!(
                "Disk usage: {:.1}% (Mount: {}, Total: {:.1}GB, Used: {:.1}GB, Available: {:.1}GB)",
                used_percent,
                disk.mount_point().to_string_lossy(),
                total_gb,
                used_gb,
                available_gb
            );

            let is_healthy = used_percent < 85.0;
            if !is_healthy {
                warn!("High disk usage detected: {:.1}%", used_percent);
            }
            is_healthy
        } else {
            warn!("Could not determine disk usage - total space is 0");
            true
        }
    } else {
        warn!("No disks found for health check");
        true
    }
}

/// Validates system time accuracy and clock synchronization.
///
/// Checks that the system clock is set to a reasonable time value
/// within expected bounds to ensure proper timestamp generation
/// and time-based operations.
///
/// # Returns
///
/// `true` if system time is within acceptable range, `false` otherwise
pub async fn check_system_time() -> bool {
    info!("Checking system time");

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            // A reasonable approximation of the current year.
            let current_year = 1970 + (timestamp / (365 * 24 * 3600));

            // Timestamps for Jan 1, 2020 and Jan 1, 2035.
            // This provides a reasonable range to detect if the clock is wildly off.
            let min_timestamp = 1_577_836_800; // Jan 1, 2020
            let max_timestamp = 2_051_222_400; // Jan 1, 2035

            let is_time_valid = timestamp > min_timestamp && timestamp < max_timestamp;

            info!(
                "System time check: timestamp={}, year≈{}, valid={}",
                timestamp, current_year, is_time_valid
            );

            if !is_time_valid {
                if timestamp <= min_timestamp {
                    error!(
                        "System time appears to be set too far in the past (year ≈ {})",
                        current_year
                    );
                } else {
                    error!(
                        "System time appears to be set too far in the future (year ≈ {})",
                        current_year
                    );
                }
            }

            // Check for clock stability by taking a second reading.
            let second_reading = SystemTime::now().duration_since(UNIX_EPOCH);
            match second_reading {
                Ok(second_duration) => {
                    let time_diff = second_duration.as_secs().abs_diff(timestamp);
                    if time_diff > 5 {
                        warn!(
                            "System time appears unstable ({}s difference between readings)",
                            time_diff
                        );
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
