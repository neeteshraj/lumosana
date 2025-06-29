use serde::{Deserialize, Serialize};
use solana_transaction_status::UiTransactionStatusMeta;

#[derive(Debug, Deserialize, Serialize)]
pub struct DebugRequest {
    pub signature: String,
    pub rpc_url: String,
}

#[derive(Debug, Serialize)]
pub struct DebugResponse {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction: serde_json::Value,
    pub meta: Option<UiTransactionStatusMeta>,
    pub analysis: TransactionAnalysis,
    // Additional comprehensive fields
    pub transaction_details: TransactionDetails,
}

#[derive(Debug, Serialize)]
pub struct TransactionDetails {
    pub version: String,
    pub recent_blockhash: Option<String>,
    pub signatures: Vec<String>,
    pub message_type: String,
    pub account_keys_count: usize,
    pub instruction_details: Vec<InstructionDetail>,
    pub inner_instructions_count: usize,
}

#[derive(Debug, Serialize)]
pub struct InstructionDetail {
    pub program_id: String,
    pub program_name: Option<String>,
    pub instruction_type: String,
    pub accounts_used: Vec<String>,
    pub data_length: usize,
}

#[derive(Debug, Serialize)]
pub struct TransactionAnalysis {
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

impl Default for TransactionAnalysis {
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
