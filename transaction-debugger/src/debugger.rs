use solana_client::rpc_client::RpcClient;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use solana_sdk::signature::Signature;
use crate::models::{DebugResponse, TransactionAnalysis};
use tracing::{info, error, instrument};
use std::str::FromStr;
use tokio::task;

#[instrument]
pub async fn analyze_transaction(sig: &str, rpc_url: &str) -> Result<DebugResponse, String> {
    info!("Analyzing transaction: {}", sig);
    
    let sig_str = sig.to_string();
    let rpc_url_str = rpc_url.to_string();
    
    // Move the blocking operation to a separate thread
    let transaction = task::spawn_blocking(move || {
        let client = RpcClient::new(rpc_url_str);
        
        // Parse the signature string
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
    
    let response = DebugResponse {
        signature: sig.to_string(),
        slot: transaction.slot,
        block_time: transaction.block_time,
        transaction: serde_json::to_value(&transaction).unwrap_or_default(),
        meta: transaction.transaction.meta.clone(),
        analysis,
    };

    info!("Transaction analysis completed for: {}", sig);
    Ok(response)
}

fn perform_analysis(transaction: &EncodedConfirmedTransactionWithStatusMeta) -> TransactionAnalysis {
    let mut analysis = TransactionAnalysis::default();

    if let Some(meta) = &transaction.transaction.meta {
        // Determine if transaction was successful
        analysis.success = meta.err.is_none();
        
        // Extract error if present
        if let Some(err) = &meta.err {
            analysis.error = Some(format!("{:?}", err));
        }

        // Extract compute units consumed if available
        match &meta.compute_units_consumed {
            solana_transaction_status::option_serializer::OptionSerializer::Some(compute_units) => {
                analysis.compute_units_consumed = Some(*compute_units);
            },
            _ => {}
        }

        // Extract fee
        analysis.fee = Some(meta.fee);

        // Extract balances
        analysis.pre_balances = meta.pre_balances.clone();
        analysis.post_balances = meta.post_balances.clone();

        // Extract log messages if available
        match &meta.log_messages {
            solana_transaction_status::option_serializer::OptionSerializer::Some(logs) => {
                analysis.log_messages = logs.clone();
            },
            _ => {}
        }
    }

    // Extract instruction and account information from the transaction
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
                    
                    // Extract account keys from message - raw messages have Vec<String>
                    analysis.accounts_involved = raw_message.account_keys.clone();

                    // For raw messages, extract program IDs from the instruction indexes
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
            // For other encoding types, we'll have limited analysis
            analysis.instruction_count = 0;
        }
    }

    // Remove duplicates from accounts_involved and program_ids
    analysis.accounts_involved.sort();
    analysis.accounts_involved.dedup();
    analysis.program_ids.sort();
    analysis.program_ids.dedup();

    analysis
}
