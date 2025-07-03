use crate::utils::error_handling::{AppError, AppResult};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;

#[allow(dead_code)]
pub struct Validator;

impl Validator {
    // Validate Solana transaction signature format
    pub fn is_valid_signature(signature: &str) -> bool {
        // Solana signatures are typically 87-88 characters long (base58 encoded)
        if signature.len() < 87 || signature.len() > 88 {
            return false;
        }

        // Check if it contains only valid base58 characters
        signature
            .chars()
            .all(|c| c.is_ascii_alphanumeric() && !matches!(c, '0' | 'O' | 'I' | 'l'))
    }

    // Validate RPC URL format
    pub fn is_valid_rpc_url(url: &str) -> bool {
        if url.is_empty() {
            return false;
        }

        // Basic URL validation
        url.starts_with("http://") || url.starts_with("https://")
    }

    // Validate Solana public key format
    pub fn is_valid_pubkey(pubkey: &str) -> bool {
        // Solana pubkeys are typically 44 characters long
        if pubkey.len() != 44 {
            return false;
        }

        // Check if it contains only valid base58 characters
        pubkey
            .chars()
            .all(|c| c.is_ascii_alphanumeric() && !matches!(c, '0' | 'O' | 'I' | 'l'))
    }

    // Validate Solana transaction signature format
    pub fn validate_signature(signature: &str) -> AppResult<()> {
        Signature::from_str(signature).map_err(|_| {
            AppError::ValidationError(
                "Invalid signature format. Must be a valid Solana transaction signature"
                    .to_string(),
            )
        })?;
        Ok(())
    }

    // Validate RPC URL format
    pub fn validate_rpc_url(url: &str) -> AppResult<()> {
        if !Self::is_valid_rpc_url(url) {
            return Err(AppError::ValidationError(
                "Invalid RPC URL. Must start with http:// or https://".to_string(),
            ));
        }
        Ok(())
    }

    // Validate Solana public key format
    pub fn validate_pubkey(pubkey: &str) -> AppResult<()> {
        Pubkey::from_str(pubkey).map_err(|_| {
            AppError::ValidationError(
                "Invalid public key format. Must be a valid Solana public key".to_string(),
            )
        })?;
        Ok(())
    }

    /// Validates multiple public keys at once
    pub fn validate_pubkeys(pubkeys: &[String]) -> AppResult<()> {
        for pubkey in pubkeys {
            Self::validate_pubkey(pubkey)?;
        }
        Ok(())
    }

    /// Validates a request with signature and RPC URL
    pub fn validate_debug_request(signature: &str, rpc_url: &str) -> AppResult<()> {
        Self::validate_signature(signature)?;
        Self::validate_rpc_url(rpc_url)?;
        Ok(())
    }
}
