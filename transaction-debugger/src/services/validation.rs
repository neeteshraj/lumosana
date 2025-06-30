//! Validation service for comprehensive data validation.
//! 
//! This module provides high-level validation services that use the validation
//! utilities to ensure data integrity throughout the application.

use crate::utils::error_handling::{AppError, AppResult};
use crate::utils::validation::Validator;
use crate::dtos::{DebugResponseDto};
use tracing::{info, warn};

/// Service for comprehensive validation of transaction data and requests
pub struct ValidationService;

impl ValidationService {
    
    /// Validates a debug response ensuring all public keys are valid
    ///
    /// # Arguments
    /// 
    /// * `response` - The debug response to validate
    /// 
    /// # Returns
    /// 
    /// Result indicating validation success or specific validation errors
    pub fn validate_debug_response(response: &DebugResponseDto) -> AppResult<()> {
        info!("Validating debug response data integrity");
        
        // Validate signature in response
        if !Validator::is_valid_signature(&response.signature) {
            return Err(AppError::ValidationError(
                "Response contains invalid signature".to_string()
            ));
        }
        
        // Validate all accounts involved
        if let Err(e) = Validator::validate_pubkeys(&response.analysis.accounts_involved) {
            warn!("Invalid accounts found in response: {}", e);
            return Err(AppError::ValidationError(
                format!("Response contains invalid account keys: {}", e)
            ));
        }
        
        // Validate all program IDs
        if let Err(e) = Validator::validate_pubkeys(&response.analysis.program_ids) {
            warn!("Invalid program IDs found in response: {}", e);
            return Err(AppError::ValidationError(
                format!("Response contains invalid program IDs: {}", e)
            ));
        }
        
        // Validate accounts in transaction details
        for instruction in &response.transaction_details.instruction_details {
            if !instruction.program_id.starts_with("Unknown-") && 
               !instruction.program_id.starts_with("parsed_instruction_") &&
               !Validator::is_valid_pubkey(&instruction.program_id) {
                return Err(AppError::ValidationError(
                    format!("Invalid program ID in instruction: {}", instruction.program_id)
                ));
            }
            
            if let Err(e) = Validator::validate_pubkeys(&instruction.accounts_used) {
                warn!("Invalid accounts in instruction: {}", e);
                return Err(AppError::ValidationError(
                    format!("Instruction contains invalid account keys: {}", e)
                ));
            }
        }
        
        info!("Debug response validation completed successfully");
        Ok(())
    }
}
