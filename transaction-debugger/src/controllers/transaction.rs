//! Transaction debugging controller for Solana transaction analysis endpoints.
//! 
//! This module provides HTTP endpoints for analyzing and debugging Solana transactions,
//! offering detailed transaction breakdown, execution analysis, and error diagnostics.

use actix_web::{web, HttpResponse, Responder, Result};
use crate::services::TransactionService;
use crate::dtos::DebugRequestDto;
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
        match TransactionService::analyze_transaction(req.into_inner()).await {
            Ok(response) => {
                Ok(HttpResponse::Ok().json(response))
            },
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e,
                "message": "Failed to analyze transaction"
            }))),
        }
    }
}
