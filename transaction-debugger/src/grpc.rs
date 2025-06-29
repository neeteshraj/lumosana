use tonic::{Request, Response, Status};
use crate::debugger::analyze_transaction;
use tracing::instrument;

// Include the generated protobuf code
pub mod debugger {
    tonic::include_proto!("debugger");
}

use debugger::transaction_debugger_server::TransactionDebugger;
use debugger::{DebugRequest, DebugResponse, TransactionAnalysis};

#[derive(Debug)]
pub struct DebuggerService;

#[tonic::async_trait]
impl TransactionDebugger for DebuggerService {
    #[instrument(skip(self))]
    async fn debug_transaction(&self, request: Request<DebugRequest>) -> Result<Response<DebugResponse>, Status> {
        let req = request.into_inner();
        
        match analyze_transaction(&req.signature, &req.rpc_url).await {
            Ok(analysis_result) => {
                // Convert our analysis to the protobuf structure
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

                let response = DebugResponse {
                    signature: analysis_result.signature,
                    slot: analysis_result.slot,
                    block_time: analysis_result.block_time,
                    transaction_json: serde_json::to_string(&analysis_result.transaction)
                        .map_err(|e| Status::internal(format!("Transaction serialization error: {}", e)))?,
                    analysis: Some(analysis),
                };
                
                Ok(Response::new(response))
            },
            Err(e) => Err(Status::internal(format!("Analysis failed: {}", e))),
        }
    }
}
