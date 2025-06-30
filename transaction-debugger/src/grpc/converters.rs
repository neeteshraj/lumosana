//! Response converter utilities for converting between HTTP DTOs and gRPC protobuf messages.
//! 
//! This module provides functionality to convert Solana transaction debug responses
//! from internal DTO format to gRPC protobuf format, ensuring compatibility between
//! HTTP and gRPC endpoints.

use crate::dtos::DebugResponseDto;
use crate::grpc::transaction::debugger::*;

/// Converter utility for transforming debug response DTOs to gRPC protobuf messages.
/// 
/// This struct provides methods to convert Solana transaction debug data from the internal
/// DTO representation used by HTTP endpoints to the protobuf message format required
/// by gRPC endpoints, ensuring both endpoints return identical structured data.
pub struct ResponseConverter;

impl ResponseConverter {
    /// Converts a debug response DTO to a gRPC DebugResponse message.
    /// 
    /// # Arguments
    /// 
    /// * `dto` - The debug response DTO containing transaction analysis and details
    /// 
    /// # Returns
    /// 
    /// A fully populated gRPC DebugResponse with transaction data, metadata, and analysis
    /// Converts a debug response DTO to a gRPC DebugResponse message.
    /// 
    /// # Arguments
    /// 
    /// * `dto` - The debug response DTO containing transaction analysis and details
    /// 
    /// # Returns
    /// 
    /// A fully populated gRPC DebugResponse with transaction data, metadata, and analysis
    pub fn to_grpc_response(dto: DebugResponseDto) -> DebugResponse {
        let analysis = TransactionAnalysis {
            success: dto.analysis.success,
            error: dto.analysis.error,
            compute_units_consumed: dto.analysis.compute_units_consumed,
            fee: dto.analysis.fee,
            accounts_involved: dto.analysis.accounts_involved,
            program_ids: dto.analysis.program_ids,
            instruction_count: dto.analysis.instruction_count as u32,
            pre_balances: dto.analysis.pre_balances,
            post_balances: dto.analysis.post_balances,
            log_messages: dto.analysis.log_messages,
        };

        let instruction_details: Vec<InstructionDetail> = dto.transaction_details.instruction_details
            .into_iter()
            .map(|detail| InstructionDetail {
                program_id: detail.program_id,
                program_name: detail.program_name,
                instruction_type: detail.instruction_type,
                accounts_used: detail.accounts_used,
                data_length: detail.data_length as u32,
            })
            .collect();

        let transaction_details = TransactionDetails {
            version: dto.transaction_details.version,
            recent_blockhash: dto.transaction_details.recent_blockhash,
            signatures: dto.transaction_details.signatures,
            message_type: dto.transaction_details.message_type,
            account_keys_count: dto.transaction_details.account_keys_count as u32,
            instruction_details,
            inner_instructions_count: dto.transaction_details.inner_instructions_count as u32,
        };

        DebugResponse {
            signature: dto.signature,
            slot: dto.slot,
            block_time: dto.block_time,
            transaction: Some(Self::convert_transaction_to_grpc(&dto.transaction, dto.slot, dto.block_time, &dto.meta)),
            meta: dto.meta.as_ref().map(|m| Self::convert_meta_to_grpc(m)),
            analysis: Some(analysis),
            transaction_details: Some(transaction_details),
        }
    }

    /// Converts Solana transaction JSON data to gRPC Transaction message format.
    /// 
    /// Extracts and transforms transaction signatures, account keys, instructions,
    /// and metadata from the raw JSON transaction data into structured gRPC format.
    /// 
    /// # Arguments
    /// 
    /// * `transaction_value` - Raw JSON transaction data from Solana RPC
    /// * `slot` - Blockchain slot number where the transaction was processed
    /// * `block_time` - Unix timestamp when the block was processed
    /// * `meta` - Optional transaction metadata containing execution details
    /// 
    /// # Returns
    /// 
    /// A gRPC Transaction message with all available transaction data
    fn convert_transaction_to_grpc(
        transaction_value: &serde_json::Value,
        slot: u64,
        block_time: Option<i64>,
        meta: &Option<solana_transaction_status::UiTransactionStatusMeta>
    ) -> Transaction {
        let mut signatures = vec![];
        if let Some(tx_obj) = transaction_value.get("transaction") {
            if let Some(sigs) = tx_obj.get("signatures").and_then(|s| s.as_array()) {
                signatures = sigs.iter()
                    .filter_map(|s| s.as_str())
                    .map(|s| s.to_string())
                    .collect();
            }
        }

        let mut account_keys = vec![];
        let mut instructions = vec![];
        let mut recent_blockhash = String::new();

        if let Some(tx_obj) = transaction_value.get("transaction") {
            if let Some(message) = tx_obj.get("message") {
                if let Some(keys) = message.get("accountKeys").and_then(|k| k.as_array()) {
                    account_keys = keys.iter()
                        .filter_map(|key| {
                            if let Some(pubkey) = key.get("pubkey").and_then(|p| p.as_str()) {
                                let signer = key.get("signer").and_then(|s| s.as_bool()).unwrap_or(false);
                                let writable = key.get("writable").and_then(|w| w.as_bool()).unwrap_or(false);
                                let source = key.get("source").and_then(|s| s.as_str()).unwrap_or("transaction");
                                
                                Some(AccountKey {
                                    pubkey: pubkey.to_string(),
                                    signer,
                                    source: source.to_string(),
                                    writable,
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                }

                if let Some(blockhash) = message.get("recentBlockhash").and_then(|b| b.as_str()) {
                    recent_blockhash = blockhash.to_string();
                }                    if let Some(instrs) = message.get("instructions").and_then(|i| i.as_array()) {
                        instructions = instrs.iter()
                            .filter_map(|instr| {
                                if instr.get("parsed").is_some() {
                                    let program = instr.get("program").and_then(|p| p.as_str()).unwrap_or("unknown");
                                    let program_id = instr.get("programId").and_then(|p| p.as_str()).unwrap_or("unknown");
                                    let stack_height = instr.get("stackHeight").and_then(|s| s.as_u64()).unwrap_or(1) as u32;
                                    
                                    let parsed = if let Some(parsed_obj) = instr.get("parsed") {
                                        Some(ParsedInstruction {
                                            r#type: parsed_obj.get("type").and_then(|t| t.as_str()).unwrap_or("unknown").to_string(),
                                            info_json: serde_json::to_string(parsed_obj.get("info").unwrap_or(&serde_json::Value::Null)).unwrap_or_default(),
                                        })
                                    } else {
                                        None
                                    };
                                    
                                    Some(TransactionInstruction {
                                        accounts: vec![],
                                        data: String::new(),
                                        program_id: program_id.to_string(),
                                        stack_height,
                                        parsed,
                                        program: Some(program.to_string()),
                                    })
                                } else if let Some(program_id) = instr.get("programId").and_then(|p| p.as_str()) {
                                    let data = instr.get("data").and_then(|d| d.as_str()).unwrap_or("");
                                    let stack_height = instr.get("stackHeight").and_then(|s| s.as_u64()).unwrap_or(1) as u32;
                                    
                                    let accounts = if let Some(acc_array) = instr.get("accounts").and_then(|a| a.as_array()) {
                                        acc_array.iter()
                                            .filter_map(|a| a.as_str())
                                            .map(|s| s.to_string())
                                            .collect()
                                    } else {
                                        vec![]
                                    };
                                    
                                    Some(TransactionInstruction {
                                        accounts,
                                        data: data.to_string(),
                                        program_id: program_id.to_string(),
                                        stack_height,
                                        parsed: None,
                                        program: None,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
            }
        }

        Transaction {
            block_time: block_time.unwrap_or(0),
            meta: meta.as_ref().map(|m| Self::convert_meta_to_grpc(m)),
            slot,
            transaction: Some(TransactionData {
                signatures,
                message: Some(TransactionMessage {
                    account_keys,
                    instructions,
                    recent_blockhash,
                }),
            }),
        }
    }

    /// Converts Solana transaction metadata to gRPC Meta message format.
    /// 
    /// Transforms execution metadata including compute units, fees, logs, token balances,
    /// and inner instructions from Solana's native format to gRPC protobuf structure.
    /// 
    /// # Arguments
    /// 
    /// * `meta` - Solana transaction metadata containing execution details
    /// 
    /// # Returns
    /// 
    /// A gRPC Meta message with comprehensive transaction execution information
    fn convert_meta_to_grpc(meta: &solana_transaction_status::UiTransactionStatusMeta) -> Meta {
        use solana_transaction_status::option_serializer::OptionSerializer;
        
        let compute_units_consumed = match &meta.compute_units_consumed {
            OptionSerializer::Some(units) => *units,
            _ => 0,
        };

        let inner_instructions = match &meta.inner_instructions {
            OptionSerializer::Some(inner_instrs) => {
                inner_instrs.iter().map(|inner| {
                    let instructions = inner.instructions.iter().map(|instr| {
                        let accounts = match instr {
                            solana_transaction_status::UiInstruction::Compiled(compiled) => {
                                compiled.accounts.iter().map(|&idx| idx.to_string()).collect()
                            },
                            _ => vec![],
                        };
                        
                        let (data, program_id, stack_height, parsed, program) = match instr {
                            solana_transaction_status::UiInstruction::Compiled(compiled) => {
                                (compiled.data.clone(), format!("program_{}", compiled.program_id_index), 1u32, None, None)
                            },
                            solana_transaction_status::UiInstruction::Parsed(parsed) => {
                                let parsed_instr = Some(ParsedInstruction {
                                    r#type: "parsed".to_string(),
                                    info_json: serde_json::to_string(&parsed).unwrap_or_default(),
                                });
                                (String::new(), "parsed_instruction".to_string(), 1u32, parsed_instr, None)
                            },
                        };

                        TransactionInstruction {
                            accounts,
                            data,
                            program_id,
                            stack_height,
                            parsed,
                            program,
                        }
                    }).collect();

                    InnerInstruction {
                        index: inner.index as u32,
                        instructions,
                    }
                }).collect()
            },
            _ => vec![],
        };

        let log_messages = match &meta.log_messages {
            OptionSerializer::Some(logs) => logs.clone(),
            _ => vec![],
        };

        let pre_token_balances = match &meta.pre_token_balances {
            OptionSerializer::Some(balances) => {
                balances.iter().map(|balance| {
                    let owner = match &balance.owner {
                        OptionSerializer::Some(o) => o.clone(),
                        _ => String::new(),
                    };
                    let program_id = match &balance.program_id {
                        OptionSerializer::Some(p) => p.clone(),
                        _ => String::new(),
                    };
                    
                    TokenBalance {
                        account_index: balance.account_index as u32,
                        mint: balance.mint.clone(),
                        owner,
                        program_id,
                        ui_token_amount: Some(UiTokenAmount {
                            amount: balance.ui_token_amount.amount.clone(),
                            decimals: balance.ui_token_amount.decimals as u32,
                            ui_amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                            ui_amount_string: balance.ui_token_amount.ui_amount_string.clone(),
                        }),
                    }
                }).collect()
            },
            _ => vec![],
        };

        let post_token_balances = match &meta.post_token_balances {
            OptionSerializer::Some(balances) => {
                balances.iter().map(|balance| {
                    let owner = match &balance.owner {
                        OptionSerializer::Some(o) => o.clone(),
                        _ => String::new(),
                    };
                    let program_id = match &balance.program_id {
                        OptionSerializer::Some(p) => p.clone(),
                        _ => String::new(),
                    };
                    
                    TokenBalance {
                        account_index: balance.account_index as u32,
                        mint: balance.mint.clone(),
                        owner,
                        program_id,
                        ui_token_amount: Some(UiTokenAmount {
                            amount: balance.ui_token_amount.amount.clone(),
                            decimals: balance.ui_token_amount.decimals as u32,
                            ui_amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                            ui_amount_string: balance.ui_token_amount.ui_amount_string.clone(),
                        }),
                    }
                }).collect()
            },
            _ => vec![],
        };

        Meta {
            compute_units_consumed,
            err: meta.err.as_ref().map(|e| format!("{:?}", e)),
            fee: meta.fee,
            inner_instructions,
            log_messages,
            post_balances: meta.post_balances.clone(),
            post_token_balances,
            pre_balances: meta.pre_balances.clone(),
            pre_token_balances,
            rewards: vec![],
            status: Some(Status {
                ok: if meta.err.is_none() { Some("".to_string()) } else { None },
            }),
        }
    }
}
