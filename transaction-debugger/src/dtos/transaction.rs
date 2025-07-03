use crate::models::transaction::{InstructionDetail, TransactionAnalysis, TransactionDetails};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DebugRequestDto {
    /// Solana transaction signature to analyze
    #[schema(
        example = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW"
    )]
    pub signature: String,
    /// RPC URL to use for fetching transaction data
    #[schema(example = "https://api.mainnet-beta.solana.com")]
    pub rpc_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DebugResponseDto {
    /// Transaction signature
    pub signature: String,
    /// Blockchain slot number
    pub slot: u64,
    /// Block timestamp
    pub block_time: Option<i64>,
    /// Raw transaction data
    pub transaction: serde_json::Value,
    /// Transaction metadata as JSON object
    #[schema(value_type = Object)]
    pub meta: Option<solana_transaction_status::UiTransactionStatusMeta>,
    /// Transaction analysis results
    pub analysis: TransactionAnalysisDto,
    /// Detailed transaction breakdown
    pub transaction_details: TransactionDetailsDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionDetailsDto {
    /// Transaction version (legacy or v0)
    pub version: String,
    /// Recent blockhash used in transaction
    pub recent_blockhash: Option<String>,
    /// Transaction signatures
    pub signatures: Vec<String>,
    /// Message type (parsed, raw, etc.)
    pub message_type: String,
    /// Number of account keys in transaction
    pub account_keys_count: usize,
    /// Details of each instruction
    pub instruction_details: Vec<InstructionDetailDto>,
    /// Number of inner instructions
    pub inner_instructions_count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InstructionDetailDto {
    /// Program ID that executed the instruction
    pub program_id: String,
    /// Human-readable program name (if known)
    pub program_name: Option<String>,
    /// Type of instruction (compiled, parsed, etc.)
    pub instruction_type: String,
    /// Accounts used by this instruction
    pub accounts_used: Vec<String>,
    /// Length of instruction data
    pub data_length: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionAnalysisDto {
    /// Whether the transaction executed successfully
    pub success: bool,
    /// Error message if transaction failed
    pub error: Option<String>,
    /// Compute units consumed by transaction
    pub compute_units_consumed: Option<u64>,
    /// Transaction fee in lamports
    pub fee: Option<u64>,
    /// List of accounts involved in the transaction
    pub accounts_involved: Vec<String>,
    /// List of program IDs called
    pub program_ids: Vec<String>,
    /// Number of instructions in transaction
    pub instruction_count: usize,
    /// Account balances before transaction
    pub pre_balances: Vec<u64>,
    /// Account balances after transaction
    pub post_balances: Vec<u64>,
    /// Log messages from transaction execution
    pub log_messages: Vec<String>,
}

impl Default for TransactionAnalysisDto {
    fn default() -> Self {
        Self {
            success: false,
            error: None,
            compute_units_consumed: None,
            fee: None,
            accounts_involved: Vec::new(),
            program_ids: Vec::new(),
            instruction_count: 0,
            pre_balances: Vec::new(),
            post_balances: Vec::new(),
            log_messages: Vec::new(),
        }
    }
}

impl From<TransactionAnalysis> for TransactionAnalysisDto {
    fn from(analysis: TransactionAnalysis) -> Self {
        Self {
            success: analysis.success,
            error: analysis.error,
            compute_units_consumed: analysis.compute_units_consumed,
            fee: analysis.fee,
            accounts_involved: analysis.accounts_involved,
            program_ids: analysis.program_ids,
            instruction_count: analysis.instruction_count,
            pre_balances: analysis.pre_balances,
            post_balances: analysis.post_balances,
            log_messages: analysis.log_messages,
        }
    }
}

impl From<TransactionDetails> for TransactionDetailsDto {
    fn from(details: TransactionDetails) -> Self {
        Self {
            version: details.version,
            recent_blockhash: details.recent_blockhash,
            signatures: details.signatures,
            message_type: details.message_type,
            account_keys_count: details.account_keys_count,
            instruction_details: details
                .instruction_details
                .into_iter()
                .map(Into::into)
                .collect(),
            inner_instructions_count: details.inner_instructions_count,
        }
    }
}

impl From<InstructionDetail> for InstructionDetailDto {
    fn from(detail: InstructionDetail) -> Self {
        Self {
            program_id: detail.program_id,
            program_name: detail.program_name,
            instruction_type: detail.instruction_type,
            accounts_used: detail.accounts_used,
            data_length: detail.data_length,
        }
    }
}
