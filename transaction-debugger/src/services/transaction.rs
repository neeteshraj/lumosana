//! Transaction analysis service for Solana blockchain transaction debugging.
//!
//! This module provides comprehensive transaction analysis capabilities including
//! instruction parsing, account interaction analysis, fee calculation, and
//! detailed execution metrics extraction from Solana transactions.

use crate::dtos::{
    DebugRequestDto, DebugResponseDto, InstructionDetailDto, TransactionAnalysisDto,
    TransactionDetailsDto,
};
use crate::services::ValidationService;
use crate::utils::error_handling::{AppError, AppResult};
use crate::utils::validation::Validator;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInstruction, UiTransactionEncoding,
};
use std::str::FromStr;
use tokio::task;
use tracing::{error, info, instrument};

/// Service for analyzing and debugging Solana blockchain transactions.
///
/// Provides methods to fetch transaction data from Solana RPC endpoints,
/// parse transaction instructions, analyze account interactions, calculate
/// fees and compute units, and generate comprehensive debugging reports.
pub struct TransactionService;

impl TransactionService {
    /// Analyzes a Solana transaction providing comprehensive debugging information.
    ///
    /// Fetches transaction data from the Solana RPC endpoint, performs detailed
    /// analysis of instructions, account interactions, and execution metrics.
    ///
    /// # Arguments
    ///
    /// * `request` - Debug request containing transaction signature and RPC URL
    ///
    /// # Returns
    ///
    /// Comprehensive transaction analysis including:
    /// - Transaction metadata (slot, block time, signatures)
    /// - Instruction breakdown and program interactions
    /// - Account balance changes and token movements
    /// - Execution logs and compute unit consumption
    /// - Fee analysis and success/failure status
    ///
    /// # Errors
    ///
    /// Returns error string if:
    /// - Transaction signature is invalid
    /// - RPC endpoint is unreachable
    /// - Transaction is not found on chain
    /// - Transaction data cannot be parsed
    #[instrument]
    pub async fn analyze_transaction(request: DebugRequestDto) -> AppResult<DebugResponseDto> {
        info!("Analyzing transaction: {}", request.signature);

        // Validate input
        Validator::validate_debug_request(&request.signature, &request.rpc_url)?;

        let sig_str = request.signature.clone();
        let rpc_url_str = request.rpc_url.clone();

        let transaction = task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url_str);

            let signature = Signature::from_str(&sig_str).map_err(|e| {
                AppError::ValidationError(format!("Invalid signature format: {}", e))
            })?;

            client
                .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
                .map_err(|e| {
                    error!("Failed to fetch transaction: {}", e);
                    AppError::RpcError(format!("Failed to fetch transaction: {}", e))
                })
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

        let analysis = Self::perform_analysis(&transaction);
        let transaction_details = Self::extract_transaction_details(&transaction);

        let response = DebugResponseDto {
            signature: request.signature.clone(),
            slot: transaction.slot,
            block_time: transaction.block_time,
            transaction: serde_json::to_value(&transaction).unwrap_or_default(),
            meta: transaction.transaction.meta.clone(),
            analysis,
            transaction_details,
        };

        // Validate the response before returning
        ValidationService::validate_debug_response(&response)?;

        info!("Transaction analysis completed for: {}", request.signature);
        Ok(response)
    }

    /// Performs comprehensive analysis of transaction execution and results.
    ///
    /// Analyzes transaction metadata to extract execution status, compute unit
    /// consumption, fees, account interactions, and program invocations.
    ///
    /// # Arguments
    ///
    /// * `transaction` - Confirmed transaction with metadata from Solana RPC
    ///
    /// # Returns
    ///
    /// Analysis DTO containing execution metrics and interaction details
    fn perform_analysis(
        transaction: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> TransactionAnalysisDto {
        let mut analysis = TransactionAnalysisDto::default();

        if let Some(meta) = &transaction.transaction.meta {
            analysis.success = meta.err.is_none();

            if let Some(err) = &meta.err {
                analysis.error = Some(format!("{:?}", err));
            }

            match &meta.compute_units_consumed {
                solana_transaction_status::option_serializer::OptionSerializer::Some(
                    compute_units,
                ) => {
                    analysis.compute_units_consumed = Some(*compute_units);
                }
                _ => {}
            }

            analysis.fee = Some(meta.fee);
            analysis.pre_balances = meta.pre_balances.clone();
            analysis.post_balances = meta.post_balances.clone();

            match &meta.log_messages {
                solana_transaction_status::option_serializer::OptionSerializer::Some(logs) => {
                    analysis.log_messages = logs.clone();
                }
                _ => {}
            }
        }

        let encoded_tx = &transaction.transaction.transaction;
        match encoded_tx {
            solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
                match &ui_tx.message {
                    solana_transaction_status::UiMessage::Parsed(parsed_message) => {
                        analysis.instruction_count = parsed_message.instructions.len();

                        for account in &parsed_message.account_keys {
                            // Only add valid public keys to the analysis
                            if Validator::is_valid_pubkey(&account.pubkey) {
                                analysis.accounts_involved.push(account.pubkey.clone());
                            }
                        }

                        for instruction in &parsed_message.instructions {
                            if let UiInstruction::Compiled(compiled_instruction) = instruction {
                                if let Some(account) = parsed_message
                                    .account_keys
                                    .get(compiled_instruction.program_id_index as usize)
                                {
                                    let program_id = &account.pubkey;
                                    // Only add valid program IDs
                                    if Validator::is_valid_pubkey(program_id)
                                        && !analysis.program_ids.contains(program_id)
                                    {
                                        analysis.program_ids.push(program_id.clone());
                                    }
                                }
                            }
                        }
                    }
                    solana_transaction_status::UiMessage::Raw(raw_message) => {
                        analysis.instruction_count = raw_message.instructions.len();

                        // Filter and validate account keys
                        analysis.accounts_involved = raw_message
                            .account_keys
                            .iter()
                            .filter(|key| Validator::is_valid_pubkey(key))
                            .cloned()
                            .collect();

                        for instruction in &raw_message.instructions {
                            let program_id_index = instruction.program_id_index;
                            if let Some(program_id) =
                                raw_message.account_keys.get(program_id_index as usize)
                            {
                                // Only add valid program IDs
                                if Validator::is_valid_pubkey(program_id)
                                    && !analysis.program_ids.contains(program_id)
                                {
                                    analysis.program_ids.push(program_id.clone());
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                analysis.instruction_count = 0;
            }
        }

        analysis.accounts_involved.sort();
        analysis.accounts_involved.dedup();
        analysis.program_ids.sort();
        analysis.program_ids.dedup();

        // Final validation of collected data
        Self::validate_analysis_data(&mut analysis);

        analysis
    }

    /// Validates the collected analysis data and removes any invalid entries
    fn validate_analysis_data(analysis: &mut TransactionAnalysisDto) {
        // Validate and filter accounts involved
        analysis
            .accounts_involved
            .retain(|account| Validator::is_valid_pubkey(account));

        // Validate and filter program IDs
        analysis
            .program_ids
            .retain(|program_id| Validator::is_valid_pubkey(program_id));
    }

    /// Extracts detailed transaction information including instruction breakdown,
    /// account interactions, and execution metrics.
    ///
    /// Parses the transaction to extract:
    /// - Version and type of message (parsed, raw, etc.)
    /// - Recent blockhash used in the transaction
    /// - Count of account keys involved
    /// - Instruction details including program IDs, types, and accounts used
    /// - Count of inner instructions if available
    ///
    /// # Arguments
    ///
    /// * `transaction` - Encoded confirmed transaction with status metadata
    ///
    /// # Returns
    ///
    /// Transaction details DTO containing structured information about the transaction
    /// fn extract_transaction_details(transaction: &EncodedConfirmedTransactionWithStatusMeta);
    ///
    /// # Errors
    ///
    /// Returns error string if transaction data cannot be parsed or is invalid
    fn extract_transaction_details(
        transaction: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> TransactionDetailsDto {
        let mut details = TransactionDetailsDto {
            version: "legacy".to_string(),
            recent_blockhash: None,
            signatures: Vec::new(),
            message_type: "unknown".to_string(),
            account_keys_count: 0,
            instruction_details: Vec::new(),
            inner_instructions_count: 0,
        };

        if let Some(meta) = &transaction.transaction.meta {
            match &meta.inner_instructions {
                solana_transaction_status::option_serializer::OptionSerializer::Some(
                    inner_instructions,
                ) => {
                    details.inner_instructions_count = inner_instructions.len();
                }
                _ => {}
            }
        }

        match &transaction.transaction.transaction {
            solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
                details.message_type = "parsed".to_string();

                match &ui_tx.message {
                    solana_transaction_status::UiMessage::Parsed(parsed_message) => {
                        details.account_keys_count = parsed_message.account_keys.len();
                        details.recent_blockhash = Some(parsed_message.recent_blockhash.clone());

                        for (index, instruction) in parsed_message.instructions.iter().enumerate() {
                            let instruction_detail = match instruction {
                                UiInstruction::Parsed(_parsed_inst) => InstructionDetailDto {
                                    program_id: format!("parsed_instruction_{}", index),
                                    program_name: None,
                                    instruction_type: "parsed".to_string(),
                                    accounts_used: Vec::new(),
                                    data_length: 0,
                                },
                                UiInstruction::Compiled(compiled_inst) => {
                                    let program_id = parsed_message
                                        .account_keys
                                        .get(compiled_inst.program_id_index as usize)
                                        .map(|acc| acc.pubkey.clone())
                                        .unwrap_or_else(|| {
                                            format!("Unknown-{}", compiled_inst.program_id_index)
                                        });

                                    // Only include valid accounts in the accounts_used list
                                    let accounts_used: Vec<String> = compiled_inst
                                        .accounts
                                        .iter()
                                        .filter_map(|&idx| {
                                            parsed_message.account_keys.get(idx as usize)
                                        })
                                        .map(|acc| acc.pubkey.clone())
                                        .filter(|account| Validator::is_valid_pubkey(account))
                                        .collect();

                                    InstructionDetailDto {
                                        program_id,
                                        program_name: None,
                                        instruction_type: "compiled".to_string(),
                                        accounts_used,
                                        data_length: compiled_inst.data.len(),
                                    }
                                }
                            };
                            details.instruction_details.push(instruction_detail);
                        }
                    }
                    solana_transaction_status::UiMessage::Raw(raw_message) => {
                        details.account_keys_count = raw_message.account_keys.len();
                        details.recent_blockhash = Some(raw_message.recent_blockhash.clone());

                        for instruction in &raw_message.instructions {
                            let program_id = raw_message
                                .account_keys
                                .get(instruction.program_id_index as usize)
                                .cloned()
                                .unwrap_or_else(|| {
                                    format!("Unknown-{}", instruction.program_id_index)
                                });

                            // Only include valid accounts in the accounts_used list
                            let accounts_used: Vec<String> = instruction
                                .accounts
                                .iter()
                                .filter_map(|&idx| raw_message.account_keys.get(idx as usize))
                                .cloned()
                                .filter(|account| Validator::is_valid_pubkey(account))
                                .collect();

                            let instruction_detail = InstructionDetailDto {
                                program_id,
                                program_name: None,
                                instruction_type: "raw".to_string(),
                                accounts_used,
                                data_length: instruction.data.len(),
                            };
                            details.instruction_details.push(instruction_detail);
                        }
                    }
                }
            }
            _ => {
                details.message_type = "other".to_string();
            }
        }

        details
    }
}
