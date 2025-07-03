//! RPC utility functions for performing RPC-related health checks.
//!
//! This module provides functionality for testing connectivity to RPC endpoints.

use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info};

/// Tests connectivity to Solana RPC endpoints.
///
/// Attempts to connect to the Solana mainnet RPC endpoint with a timeout
/// to verify network connectivity and RPC service availability.
///
/// # Returns
///
/// `true` if RPC endpoint is reachable and responsive, `false` otherwise
pub async fn check_rpc_connectivity() -> bool {
    info!("Testing RPC connectivity");

    let rpc_url = "https://api.mainnet-beta.solana.com";

    match timeout(Duration::from_secs(3), async {
        let client = reqwest::Client::new();
        client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .body(r#"{"jsonrpc":"2.0","id":1,"method":"getVersion","params":[]}"#)
            .send()
            .await
    })
    .await
    {
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
