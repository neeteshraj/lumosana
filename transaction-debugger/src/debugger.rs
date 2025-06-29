use solana_client::rpc_client::RpcClient;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use solana_sdk::signature::Signature;
use crate::models::{DebugResponse, TransactionAnalysis, TransactionDetails, InstructionDetail};
use tracing::{info, error, instrument};
use std::str::FromStr;
use tokio::task;

#[instrument]
pub async fn analyze_transaction(sig: &str, rpc_url: &str) -> Result<DebugResponse, String> {
    info!("Analyzing transaction: {}", sig);
    
    let sig_str = sig.to_string();
    let rpc_url_str = rpc_url.to_string();
    
    let transaction = task::spawn_blocking(move || {
        let client = RpcClient::new(rpc_url_str);
        
        let signature = Signature::from_str(&sig_str)
            .map_err(|e| format!("Invalid signature format: {}", e))?;
        
        client
            .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
            .map_err(|e| {
                error!("Failed to fetch transaction: {}", e);
                format!("Failed to fetch transaction: {}", e)
            })
    }).await.map_err(|e| format!("Task join error: {}", e))??;

    let analysis = perform_analysis(&transaction);
    let transaction_details = extract_transaction_details(&transaction);
    
    let response = DebugResponse {
        signature: sig.to_string(),
        slot: transaction.slot,
        block_time: transaction.block_time,
        transaction: serde_json::to_value(&transaction).unwrap_or_default(),
        meta: transaction.transaction.meta.clone(),
        analysis,
        transaction_details,
    };

    info!("Transaction analysis completed for: {}", sig);
    Ok(response)
}

fn perform_analysis(transaction: &EncodedConfirmedTransactionWithStatusMeta) -> TransactionAnalysis {
    let mut analysis = TransactionAnalysis::default();

    if let Some(meta) = &transaction.transaction.meta {
        analysis.success = meta.err.is_none();
        
        if let Some(err) = &meta.err {
            analysis.error = Some(format!("{:?}", err));
        }

        match &meta.compute_units_consumed {
            solana_transaction_status::option_serializer::OptionSerializer::Some(compute_units) => {
                analysis.compute_units_consumed = Some(*compute_units);
            },
            _ => {}
        }

        analysis.fee = Some(meta.fee);

        analysis.pre_balances = meta.pre_balances.clone();
        analysis.post_balances = meta.post_balances.clone();

        match &meta.log_messages {
            solana_transaction_status::option_serializer::OptionSerializer::Some(logs) => {
                analysis.log_messages = logs.clone();
            },
            _ => {}
        }
    }

    let encoded_tx = &transaction.transaction.transaction;
    match encoded_tx {
        solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
            match &ui_tx.message {
                solana_transaction_status::UiMessage::Parsed(parsed_message) => {
                    analysis.instruction_count = parsed_message.instructions.len();
                    
                    // Extract account keys from parsed message - convert ParsedAccount to String
                    for account in &parsed_message.account_keys {
                        analysis.accounts_involved.push(account.pubkey.clone());
                    }

                    // For parsed instructions, extract program IDs from compiled instructions
                    for instruction in &parsed_message.instructions {
                        if let solana_transaction_status::UiInstruction::Compiled(compiled_instruction) = instruction {
                            // For compiled instructions, we need to look up the program ID by index
                            if let Some(account) = parsed_message.account_keys.get(compiled_instruction.program_id_index as usize) {
                                let program_id = &account.pubkey;
                                if !analysis.program_ids.contains(program_id) {
                                    analysis.program_ids.push(program_id.clone());
                                }
                            }
                        }
                    }
                },
                solana_transaction_status::UiMessage::Raw(raw_message) => {
                    analysis.instruction_count = raw_message.instructions.len();
                    
                    analysis.accounts_involved = raw_message.account_keys.clone();

                    for instruction in &raw_message.instructions {
                        let program_id_index = instruction.program_id_index;
                        if let Some(program_id) = raw_message.account_keys.get(program_id_index as usize) {
                            if !analysis.program_ids.contains(program_id) {
                                analysis.program_ids.push(program_id.clone());
                            }
                        }
                    }
                }
            }
        },
        _ => {
            analysis.instruction_count = 0;
        }
    }

    analysis.accounts_involved.sort();
    analysis.accounts_involved.dedup();
    analysis.program_ids.sort();
    analysis.program_ids.dedup();

    analysis
}

fn extract_transaction_details(transaction: &EncodedConfirmedTransactionWithStatusMeta) -> TransactionDetails {
    let mut details = TransactionDetails {
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
            solana_transaction_status::option_serializer::OptionSerializer::Some(inner_instructions) => {
                details.inner_instructions_count = inner_instructions.len();
            },
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
                            solana_transaction_status::UiInstruction::Parsed(_parsed_inst) => {
                                InstructionDetail {
                                    program_id: format!("parsed_instruction_{}", index),
                                    program_name: None,
                                    instruction_type: "parsed".to_string(),
                                    accounts_used: Vec::new(),
                                    data_length: 0,
                                }
                            },
                            solana_transaction_status::UiInstruction::Compiled(compiled_inst) => {
                                let program_id = parsed_message.account_keys
                                    .get(compiled_inst.program_id_index as usize)
                                    .map(|acc| acc.pubkey.clone())
                                    .unwrap_or_else(|| format!("Unknown-{}", compiled_inst.program_id_index));
                                
                                InstructionDetail {
                                    program_id,
                                    program_name: None,
                                    instruction_type: "compiled".to_string(),
                                    accounts_used: compiled_inst.accounts.iter()
                                        .filter_map(|&idx| parsed_message.account_keys.get(idx as usize))
                                        .map(|acc| acc.pubkey.clone())
                                        .collect(),
                                    data_length: compiled_inst.data.len(),
                                }
                            }
                        };
                        details.instruction_details.push(instruction_detail);
                    }
                },
                solana_transaction_status::UiMessage::Raw(raw_message) => {
                    details.account_keys_count = raw_message.account_keys.len();
                    details.recent_blockhash = Some(raw_message.recent_blockhash.clone());

                    for instruction in &raw_message.instructions {
                        let program_id = raw_message.account_keys
                            .get(instruction.program_id_index as usize)
                            .cloned()
                            .unwrap_or_else(|| format!("Unknown-{}", instruction.program_id_index));
                        
                        let instruction_detail = InstructionDetail {
                            program_id,
                            program_name: None,
                            instruction_type: "raw".to_string(),
                            accounts_used: instruction.accounts.iter()
                                .filter_map(|&idx| raw_message.account_keys.get(idx as usize))
                                .cloned()
                                .collect(),
                            data_length: instruction.data.len(),
                        };
                        details.instruction_details.push(instruction_detail);
                    }
                }
            }
        },
        _ => {
            details.message_type = "other".to_string();
        }
    }

    details
}
