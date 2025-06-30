use tonic::{Request, Response, Status};
use crate::debugger::analyze_transaction;
use tracing::instrument;

// Include the generated protobuf code
pub mod debugger {
    tonic::include_proto!("debugger");
}

use debugger::transaction_debugger_server::TransactionDebugger;
use debugger::{DebugRequest, DebugResponse, TransactionAnalysis, TransactionDetails, InstructionDetail};

#[derive(Debug)]
pub struct DebuggerService;

#[tonic::async_trait]
impl TransactionDebugger for DebuggerService {
    #[instrument(skip(self))]
    async fn debug_transaction(&self, request: Request<DebugRequest>) -> Result<Response<DebugResponse>, Status> {
        let req = request.into_inner();
        
        match analyze_transaction(&req.signature, &req.rpc_url).await {
            Ok(analysis_result) => {
                let analysis = TransactionAnalysis {
                    success: analysis_result.analysis.success,
                    error: analysis_result.analysis.error,
                    compute_units_consumed: analysis_result.analysis.compute_units_consumed,
                    fee: analysis_result.analysis.fee,
                    accounts_involved: analysis_result.analysis.accounts_involved,
                    program_ids: analysis_result.analysis.program_ids,
                    instruction_count: analysis_result.analysis.instruction_count as u32,
                    pre_balances: analysis_result.analysis.pre_balances,
                    post_balances: analysis_result.analysis.post_balances,
                    log_messages: analysis_result.analysis.log_messages,
                };

                // Convert transaction details to protobuf structure
                let instruction_details: Vec<InstructionDetail> = analysis_result.transaction_details.instruction_details
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
                    version: analysis_result.transaction_details.version,
                    recent_blockhash: analysis_result.transaction_details.recent_blockhash,
                    signatures: analysis_result.transaction_details.signatures,
                    message_type: analysis_result.transaction_details.message_type,
                    account_keys_count: analysis_result.transaction_details.account_keys_count as u32,
                    instruction_details,
                    inner_instructions_count: analysis_result.transaction_details.inner_instructions_count as u32,
                };

                let response = DebugResponse {
                    signature: analysis_result.signature,
                    slot: analysis_result.slot,
                    block_time: analysis_result.block_time,
                    transaction: Some(convert_transaction_to_grpc(&analysis_result.transaction, analysis_result.slot, analysis_result.block_time, &analysis_result.meta)),
                    meta: analysis_result.meta.as_ref().map(|meta| convert_meta_to_grpc(meta)),
                    analysis: Some(analysis),
                    transaction_details: Some(transaction_details),
                };
                
                Ok(Response::new(response))
            },
            Err(e) => Err(Status::internal(format!("Analysis failed: {}", e))),
        }
    }
}

fn convert_transaction_to_grpc(
    transaction_value: &serde_json::Value,
    slot: u64,
    block_time: Option<i64>,
    meta: &Option<solana_transaction_status::UiTransactionStatusMeta>
) -> debugger::Transaction {
    let mut account_keys = Vec::new();
    let mut instructions = Vec::new();
    let mut signatures = Vec::new();
    let mut recent_blockhash = String::new();

    // Extract signatures from the transaction
    if let Some(sigs) = transaction_value.get("transaction")
        .and_then(|t| t.get("signatures"))
        .and_then(|s| s.as_array()) {
        signatures = sigs.iter()
            .filter_map(|s| s.as_str())
            .map(|s| s.to_string())
            .collect();
    }

    // Extract message data
    if let Some(message) = transaction_value.get("transaction")
        .and_then(|t| t.get("message"))
        .and_then(|m| m.as_object()) {
        // Extract recent blockhash
        if let Some(blockhash) = message.get("recentBlockhash")
            .and_then(|b| b.as_str()) {
            recent_blockhash = blockhash.to_string();
        }

        // Extract account keys
        if let Some(keys) = message.get("accountKeys")
            .and_then(|k| k.as_array()) {
            for key in keys {
                if let Some(key_obj) = key.as_object() {
                    let pubkey = key_obj.get("pubkey")
                        .and_then(|p| p.as_str())
                        .unwrap_or("")
                        .to_string();
                    let signer = key_obj.get("signer")
                        .and_then(|s| s.as_bool())
                        .unwrap_or(false);
                    let source = key_obj.get("source")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    let writable = key_obj.get("writable")
                        .and_then(|w| w.as_bool())
                        .unwrap_or(false);
                    
                    account_keys.push(debugger::AccountKey {
                        pubkey,
                        signer,
                        source,
                        writable,
                    });
                } else if let Some(key_str) = key.as_str() {
                    account_keys.push(debugger::AccountKey {
                        pubkey: key_str.to_string(),
                        signer: false,
                        source: String::new(),
                        writable: false,
                    });
                }
            }
        }

        // Extract instructions
        if let Some(instrs) = message.get("instructions")
            .and_then(|i| i.as_array()) {
            for instr in instrs {
                let instruction = if let Some(instr_obj) = instr.as_object() {
                    // Check if it's a parsed instruction
                    if let Some(parsed) = instr_obj.get("parsed")
                        .and_then(|p| p.as_object()) {
                        let parsed_instruction = debugger::ParsedInstruction {
                            r#type: parsed.get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            info_json: parsed.get("info")
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                        };
                        
                        debugger::TransactionInstruction {
                            accounts: Vec::new(),
                            data: String::new(),
                            program_id: instr_obj.get("programId")
                                .and_then(|p| p.as_str())
                                .unwrap_or("")
                                .to_string(),
                            stack_height: instr_obj.get("stackHeight")
                                .and_then(|s| s.as_u64())
                                .unwrap_or(0) as u32,
                            parsed: Some(parsed_instruction),
                            program: instr_obj.get("program")
                                .and_then(|p| p.as_str())
                                .map(|s| s.to_string()),
                        }
                    } else {
                        // Handle compiled instruction
                        let accounts = instr_obj.get("accounts")
                            .and_then(|a| a.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect())
                            .unwrap_or_default();
                        let data = instr_obj.get("data")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();
                        let program_id = instr_obj.get("programId")
                            .and_then(|p| p.as_str())
                            .unwrap_or("")
                            .to_string();

                        debugger::TransactionInstruction {
                            accounts,
                            data,
                            program_id,
                            stack_height: instr_obj.get("stackHeight")
                                .and_then(|s| s.as_u64())
                                .unwrap_or(0) as u32,
                            parsed: None,
                            program: instr_obj.get("program")
                                .and_then(|p| p.as_str())
                                .map(|s| s.to_string()),
                        }
                    }
                } else {
                    debugger::TransactionInstruction {
                        accounts: Vec::new(),
                        data: String::new(),
                        program_id: String::new(),
                        stack_height: 0,
                        parsed: None,
                        program: None,
                    }
                };
                instructions.push(instruction);
            }
        }
    }

    debugger::Transaction {
        block_time: block_time.unwrap_or(0),
        meta: meta.as_ref().map(|m| convert_meta_to_grpc(m)),
        slot,
        transaction: Some(debugger::TransactionData {
            message: Some(debugger::TransactionMessage {
                account_keys,
                instructions,
                recent_blockhash,
            }),
            signatures,
        }),
    }
}

fn convert_meta_to_grpc(meta: &solana_transaction_status::UiTransactionStatusMeta) -> debugger::Meta {
    // Convert pre token balances
    let pre_token_balances = match &meta.pre_token_balances {
        solana_transaction_status::option_serializer::OptionSerializer::Some(balances) => {
            balances.iter().map(|balance| debugger::TokenBalance {
                account_index: balance.account_index as u32,
                mint: balance.mint.clone(),
                ui_token_amount: Some(debugger::UiTokenAmount {
                    ui_amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                    decimals: balance.ui_token_amount.decimals as u32,
                    amount: balance.ui_token_amount.amount.clone(),
                    ui_amount_string: balance.ui_token_amount.ui_amount_string.clone(),
                }),
                owner: match &balance.owner {
                    solana_transaction_status::option_serializer::OptionSerializer::Some(owner) => owner.clone(),
                    _ => String::new(),
                },
                program_id: match &balance.program_id {
                    solana_transaction_status::option_serializer::OptionSerializer::Some(program_id) => program_id.clone(),
                    _ => String::new(),
                },
            }).collect()
        },
        _ => Vec::new(),
    };

    // Convert post token balances
    let post_token_balances = match &meta.post_token_balances {
        solana_transaction_status::option_serializer::OptionSerializer::Some(balances) => {
            balances.iter().map(|balance| debugger::TokenBalance {
                account_index: balance.account_index as u32,
                mint: balance.mint.clone(),
                ui_token_amount: Some(debugger::UiTokenAmount {
                    ui_amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                    decimals: balance.ui_token_amount.decimals as u32,
                    amount: balance.ui_token_amount.amount.clone(),
                    ui_amount_string: balance.ui_token_amount.ui_amount_string.clone(),
                }),
                owner: match &balance.owner {
                    solana_transaction_status::option_serializer::OptionSerializer::Some(owner) => owner.clone(),
                    _ => String::new(),
                },
                program_id: match &balance.program_id {
                    solana_transaction_status::option_serializer::OptionSerializer::Some(program_id) => program_id.clone(),
                    _ => String::new(),
                },
            }).collect()
        },
        _ => Vec::new(),
    };

    // Convert inner instructions
    let inner_instructions = match &meta.inner_instructions {
        solana_transaction_status::option_serializer::OptionSerializer::Some(inner_instrs) => {
            inner_instrs.iter().map(|inner_instr| debugger::InnerInstruction {
                index: inner_instr.index as u32,
                instructions: inner_instr.instructions.iter().map(|ui_instr| {
                    match ui_instr {
                        solana_transaction_status::UiInstruction::Compiled(compiled_instr) => {
                            debugger::TransactionInstruction {
                                program_id: String::new(), // We'd need account keys to resolve this
                                accounts: compiled_instr.accounts.iter().map(|&idx| idx.to_string()).collect(),
                                data: compiled_instr.data.clone(),
                                stack_height: compiled_instr.stack_height.unwrap_or(0) as u32,
                                parsed: None,
                                program: None,
                            }
                        },
                        solana_transaction_status::UiInstruction::Parsed(parsed_instr) => {
                            let parsed = debugger::ParsedInstruction {
                                r#type: "parsed".to_string(),
                                info_json: serde_json::to_string(parsed_instr).unwrap_or_default(),
                            };
                            debugger::TransactionInstruction {
                                program_id: "parsed_instruction".to_string(),
                                accounts: Vec::new(),
                                data: String::new(),
                                stack_height: 0,
                                parsed: Some(parsed),
                                program: None,
                            }
                        }
                    }
                }).collect(),
            }).collect()
        },
        _ => Vec::new(),
    };

    // Convert rewards - simplified as strings for now
    let rewards = match &meta.rewards {
        solana_transaction_status::option_serializer::OptionSerializer::Some(reward_list) => {
            reward_list.iter().map(|reward| {
                format!("{}:{}:{}", reward.pubkey, reward.lamports, reward.post_balance)
            }).collect()
        },
        _ => Vec::new(),
    };

    debugger::Meta {
        compute_units_consumed: match &meta.compute_units_consumed {
            solana_transaction_status::option_serializer::OptionSerializer::Some(units) => *units,
            _ => 0,
        },
        err: meta.err.as_ref().map(|e| format!("{:?}", e)),
        fee: meta.fee,
        inner_instructions,
        log_messages: match &meta.log_messages {
            solana_transaction_status::option_serializer::OptionSerializer::Some(logs) => logs.clone(),
            _ => Vec::new(),
        },
        post_balances: meta.post_balances.clone(),
        post_token_balances,
        pre_balances: meta.pre_balances.clone(),
        pre_token_balances,
        rewards,
        status: Some(debugger::Status {
            ok: if meta.err.is_none() { Some(String::new()) } else { None },
        }),
    }
}
