//! Transaction debugging controller for Solana transaction analysis endpoints.
//!
//! This module provides HTTP endpoints for analyzing and debugging Solana transactions,
//! offering detailed transaction breakdown, execution analysis, and error diagnostics.

use crate::dtos::transaction::{DebugRequestDto, DebugResponseDto};
use crate::services::TransactionService;
use crate::utils::error_handling::AppError;
use crate::utils::validation::Validator;
use actix_web::{web, HttpResponse, Responder, Result};
use opentelemetry::{global, KeyValue};
use tracing::{error, info, instrument};

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
    /// JSON response with detailed transaction analysis including
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
    #[instrument(name = "debug_transaction_handler", skip(req))]
    pub async fn debug_transaction(req: web::Json<DebugRequestDto>) -> Result<impl Responder> {
        let request = req.into_inner();

        let span = tracing::Span::current();
        span.record("rpc_url", &request.rpc_url.as_str());
        span.record("signature", &request.signature.as_str());

        let meter = global::meter("transaction_debugger");
        let counter = meter
            .u64_counter("transaction_analysis_total")
            .with_description("Total number of transaction analyses attempted")
            .build();

        if let Err(e) = Validator::validate_debug_request(&request.signature, &request.rpc_url) {
            let error_response = serde_json::json!({
            "error": "validation_error",
            "message": e.to_string()
        });

            counter.add(1, &[KeyValue::new("status", "validation_error")]);

            return Ok(HttpResponse::BadRequest().json(error_response));
        }

        match TransactionService::analyze_transaction(request).await {
            Ok(response) => {
                info!("Transaction analysis successful");
                counter.add(1, &[KeyValue::new("status", "success")]);
                Ok(HttpResponse::Ok().json(response))
            }
            Err(e) => {
                error!(error = ?e, "Transaction analysis failed");

                counter.add(1, &[KeyValue::new("status", e.variant_name())]);

                let error_response = serde_json::json!({
                "error": e.variant_name(),
                "message": e.to_string()
            });

                let response = match &e {
                    AppError::ValidationError(_) => {
                        HttpResponse::BadRequest().json(&error_response)
                    }
                    AppError::NetworkError(_) => {
                        HttpResponse::ServiceUnavailable().json(&error_response)
                    }
                    AppError::TransactionNotFound(_) => {
                        HttpResponse::NotFound().json(&error_response)
                    }
                    AppError::RpcError(_) => {
                        HttpResponse::ServiceUnavailable().json(&error_response)
                    }
                    AppError::InternalError(_) => {
                        HttpResponse::InternalServerError().json(&error_response)
                    }
                };
                Ok(response)
            }
        }
    }
}

/// Analyzes and debugs a Solana transaction providing detailed execution information.
#[utoipa::path(
    post,
    path = "/debug",
    request_body = DebugRequestDto,
    responses(
        (status = 200, description = "Transaction debug analysis completed successfully", body = DebugResponseDto),
        (status = 400, description = "Invalid request parameters"),
        (status = 404, description = "Transaction not found"),
        (status = 503, description = "Service unavailable")
    ),
    tag = "transaction"
)]
pub async fn debug_transaction_api(req: web::Json<DebugRequestDto>) -> Result<impl Responder> {
    TransactionController::debug_transaction(req).await
}
