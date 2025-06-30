use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    ValidationError(String),
    NetworkError(String),
    TransactionNotFound(String),
    RpcError(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AppError::TransactionNotFound(msg) => write!(f, "Transaction not found: {}", msg),
            AppError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;
