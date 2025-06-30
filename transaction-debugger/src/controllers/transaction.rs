//! Transaction debugging controller for Solana transaction analysis endpoints.
//! 
//! This module provides HTTP endpoints for analyzing and debugging Solana transactions,
//! offering detailed transaction breakdown, execution analysis, and error diagnostics.

use actix_web::{web, HttpResponse, Responder, Result};
use crate::services::TransactionService;
use crate::dtos::DebugRequestDto;
use crate::utils::error_handling::AppError;
use crate::utils::validation::Validator;
use tracing::instrument;

/// HTTP controller for Solana transaction debugging and analysis.
/// 
/// Provides RESTful endpoints for analyzing transaction execution,
/// extracting instruction details, account interactions, and identifying
/// potential issues or optimization opportunities in Solana transactions.
pub struct TransactionController;

impl TransactionController {
    /// Analyzes and debugs a Solana transaction providing detailed execution information.
    /// 
    /// This endpoint accepts a transaction signature and performs comprehensive analysis
    /// including instruction breakdown, account interactions, program execution details,
    /// compute unit consumption, and potential error diagnosis.
    /// 
    /// # Arguments
    /// 
    /// * `req` - JSON request containing the transaction signature to analyze
    /// 
    /// # Returns
    /// 
    /// JSON response with detailed transaction analysis including:
    /// - Transaction metadata (signatures, slot, block time)
    /// - Instruction details and program interactions
    /// - Account balance changes and token movements
    /// - Execution logs and error information
    /// - Compute unit consumption and fees
    /// 
    /// # Errors
    /// 
    /// Returns HTTP 500 with error details if:
    /// - Transaction signature is invalid or not found
    /// - RPC connection fails
    /// - Transaction data cannot be parsed
    /// 
    /// # Example Request Body
    /// 
    /// ```json
    /// {
    ///   "signature": "5j7s88kF9E...transaction_signature_here"
    /// }
    /// ```
    #[instrument]
    pub async fn debug_transaction(req: web::Json<DebugRequestDto>) -> Result<impl Responder> {
        let request = req.into_inner();
        
        // Early validation
        if let Err(e) = Validator::validate_debug_request(&request.signature, &request.rpc_url) {
            let error_response = serde_json::json!({
                "error": "validation_error",
                "message": e.to_string()
            });
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
        
        match TransactionService::analyze_transaction(request).await {
            Ok(response) => {
                Ok(HttpResponse::Ok().json(response))
            },
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": match e {
                        AppError::ValidationError(_) => "validation_error",
                        AppError::NetworkError(_) => "network_error",
                        AppError::TransactionNotFound(_) => "transaction_not_found",
                        AppError::RpcError(_) => "rpc_error",
                        AppError::InternalError(_) => "internal_error",
                    },
                    "message": e.to_string()
                });
                
                match e {
                    AppError::ValidationError(_) => Ok(HttpResponse::BadRequest().json(error_response)),
                    AppError::NetworkError(_) => Ok(HttpResponse::ServiceUnavailable().json(error_response)),
                    AppError::TransactionNotFound(_) => Ok(HttpResponse::NotFound().json(error_response)),
                    AppError::RpcError(_) => Ok(HttpResponse::ServiceUnavailable().json(error_response)),
                    AppError::InternalError(_) => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            },
        }
    }
}
