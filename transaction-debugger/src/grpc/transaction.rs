use tonic::{Request, Response, Status};
use crate::services::TransactionService;
use crate::dtos::DebugRequestDto;
use crate::grpc::converters::ResponseConverter;
use crate::utils::error_handling::AppError;
use crate::utils::validation::Validator;
use tracing::instrument;

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
        
        // Validate the request early
        if let Err(e) = Validator::validate_debug_request(&req.signature, &req.rpc_url) {
            return Err(Status::invalid_argument(e.to_string()));
        }
        
        let debug_request = DebugRequestDto {
            signature: req.signature,
            rpc_url: req.rpc_url,
        };
        
        match TransactionService::analyze_transaction(debug_request).await {
            Ok(analysis_result) => {
                let response = ResponseConverter::to_grpc_response(analysis_result);
                Ok(Response::new(response))
            },
            Err(e) => {
                let status = match e {
                    AppError::ValidationError(_) => Status::invalid_argument(e.to_string()),
                    AppError::NetworkError(_) => Status::unavailable(e.to_string()),
                    AppError::TransactionNotFound(_) => Status::not_found(e.to_string()),
                    AppError::RpcError(_) => Status::unavailable(e.to_string()),
                    AppError::InternalError(_) => Status::internal(e.to_string()),
                };
                Err(status)
            }
        }
    }
}
