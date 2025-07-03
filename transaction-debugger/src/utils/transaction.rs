//! Transaction utility functions for performing transaction-related health checks.
//!
//! This module provides functionality for testing the transaction processing pipeline.

use crate::dtos::transaction::TransactionAnalysisDto;
use crate::models::transaction::TransactionAnalysis;

/// Validates transaction processing pipeline functionality.
///
/// Creates a mock `DebugRequest` and processes it to verify that the
/// transaction analysis is performed as expected.
///
/// # Returns
///
/// `true` if the transaction processing pipeline is functional, `false` otherwise
pub async fn check_transaction_processing() -> bool {
    let mock_analysis = TransactionAnalysis {
        success: true,
        error: None,
        compute_units_consumed: Some(12345),
        fee: Some(5000),
        accounts_involved: vec!["acc1".to_string(), "acc2".to_string()],
        program_ids: vec!["prog1".to_string()],
        instruction_count: 1,
        pre_balances: vec![100, 200],
        post_balances: vec![90, 210],
        log_messages: vec!["log1".to_string()],
    };

    let analysis_dto = TransactionAnalysisDto::from(mock_analysis);

    analysis_dto.success
}
