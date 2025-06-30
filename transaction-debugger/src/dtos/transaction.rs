use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DebugRequestDto {
    pub signature: String,
    pub rpc_url: String,
}

#[derive(Debug, Serialize)]
pub struct DebugResponseDto {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction: serde_json::Value,
    pub meta: Option<solana_transaction_status::UiTransactionStatusMeta>,
    pub analysis: TransactionAnalysisDto,
    pub transaction_details: TransactionDetailsDto,
}

#[derive(Debug, Serialize)]
pub struct TransactionDetailsDto {
    pub version: String,
    pub recent_blockhash: Option<String>,
    pub signatures: Vec<String>,
    pub message_type: String,
    pub account_keys_count: usize,
    pub instruction_details: Vec<InstructionDetailDto>,
    pub inner_instructions_count: usize,
}

#[derive(Debug, Serialize)]
pub struct InstructionDetailDto {
    pub program_id: String,
    pub program_name: Option<String>,
    pub instruction_type: String,
    pub accounts_used: Vec<String>,
    pub data_length: usize,
}

#[derive(Debug, Serialize)]
pub struct TransactionAnalysisDto {
    pub success: bool,
    pub error: Option<String>,
    pub compute_units_consumed: Option<u64>,
    pub fee: Option<u64>,
    pub accounts_involved: Vec<String>,
    pub program_ids: Vec<String>,
    pub instruction_count: usize,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
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
