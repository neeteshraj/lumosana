use tonic::{Request, Response, Status};
use crate::services::TransactionService;
use crate::dtos::DebugRequestDto;
use crate::grpc::converters::ResponseConverter;
use tracing::instrument;

// Include the generated protobuf code
pub mod debugger {
    tonic::include_proto!("debugger");
}

use debugger::transaction_debugger_server::TransactionDebugger;
use debugger::{DebugRequest, DebugResponse};

#[derive(Debug)]
pub struct TransactionDebuggerService;

#[tonic::async_trait]
impl TransactionDebugger for TransactionDebuggerService {
    #[instrument(skip(self))]
    async fn debug_transaction(&self, request: Request<DebugRequest>) -> Result<Response<DebugResponse>, Status> {
        let req = request.into_inner();
        
        let debug_request = DebugRequestDto {
            signature: req.signature,
            rpc_url: req.rpc_url,
        };
        
        match TransactionService::analyze_transaction(debug_request).await {
            Ok(analysis_result) => {
                let response = ResponseConverter::to_grpc_response(analysis_result);
                Ok(Response::new(response))
            },
            Err(e) => Err(Status::internal(format!("Analysis failed: {}", e))),
        }
    }
}
