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
    debugger::Transaction {
        block_time: block_time.unwrap_or(0),
        meta: meta.as_ref().map(|m| convert_meta_to_grpc(m)),
        slot,
        transaction: Some(debugger::TransactionData {
            message: Some(debugger::TransactionMessage {
                account_keys: Vec::new(), 
                instructions: Vec::new(),   
                recent_blockhash: String::new(), 
            }),
            signatures: Vec::new(), 
        }),
    }
}

fn convert_meta_to_grpc(meta: &solana_transaction_status::UiTransactionStatusMeta) -> debugger::Meta {
    debugger::Meta {
        compute_units_consumed: match &meta.compute_units_consumed {
            solana_transaction_status::option_serializer::OptionSerializer::Some(units) => *units,
            _ => 0,
        },
        err: meta.err.as_ref().map(|e| format!("{:?}", e)),
        fee: meta.fee,
        inner_instructions: Vec::new(), 
        log_messages: match &meta.log_messages {
            solana_transaction_status::option_serializer::OptionSerializer::Some(logs) => logs.clone(),
            _ => Vec::new(),
        },
        post_balances: meta.post_balances.clone(),
        post_token_balances: Vec::new(), 
        pre_balances: meta.pre_balances.clone(),
        pre_token_balances: Vec::new(), 
        rewards: Vec::new(), 
        status: Some(debugger::Status {
            ok: if meta.err.is_none() { Some(String::new()) } else { None },
        }),
    }
}
