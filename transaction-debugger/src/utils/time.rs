//! Time utility functions for generating timestamps.
//!
//! This module provides functionality for creating timestamps for health check responses.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Generates a current timestamp string for health check responses.
///
/// Creates a Unix timestamp string representing the current system time
/// for inclusion in health check response metadata.
///
/// # Returns
///
/// Unix timestamp as a string, or "0" if time cannot be determined
pub fn get_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
        .to_string()
}
