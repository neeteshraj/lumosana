// Simple converter for now - just return the same format as the original
use crate::dtos::DebugResponseDto;
use crate::grpc::transaction::debugger::*;

pub struct ResponseConverter;

impl ResponseConverter {
    pub fn to_grpc_response(dto: DebugResponseDto) -> DebugResponse {
        let analysis = TransactionAnalysis {
            success: dto.analysis.success,
            error: dto.analysis.error,
            compute_units_consumed: dto.analysis.compute_units_consumed,
            fee: dto.analysis.fee,
            accounts_involved: dto.analysis.accounts_involved,
            program_ids: dto.analysis.program_ids,
            instruction_count: dto.analysis.instruction_count as u32,
            pre_balances: dto.analysis.pre_balances,
            post_balances: dto.analysis.post_balances,
            log_messages: dto.analysis.log_messages,
        };

        let instruction_details: Vec<InstructionDetail> = dto.transaction_details.instruction_details
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
            version: dto.transaction_details.version,
            recent_blockhash: dto.transaction_details.recent_blockhash,
            signatures: dto.transaction_details.signatures,
            message_type: dto.transaction_details.message_type,
            account_keys_count: dto.transaction_details.account_keys_count as u32,
            instruction_details,
            inner_instructions_count: dto.transaction_details.inner_instructions_count as u32,
        };

        DebugResponse {
            signature: dto.signature,
            slot: dto.slot,
            block_time: dto.block_time,
            transaction: Some(Self::convert_transaction_to_grpc(&dto.transaction, dto.slot, dto.block_time, &dto.meta)),
            meta: dto.meta.as_ref().map(|m| Self::convert_meta_to_grpc(m)),
            analysis: Some(analysis),
            transaction_details: Some(transaction_details),
        }
    }

    fn convert_transaction_to_grpc(
        transaction_value: &serde_json::Value,
        slot: u64,
        block_time: Option<i64>,
        meta: &Option<solana_transaction_status::UiTransactionStatusMeta>
    ) -> Transaction {
        // Return a simple transaction structure
        Transaction {
            block_time: block_time.unwrap_or(0),
            meta: meta.as_ref().map(|m| Self::convert_meta_to_grpc(m)),
            slot,
            transaction: Some(TransactionData {
                signatures: vec![], // Simplified for now
                message: Some(TransactionMessage {
                    account_keys: vec![], // Simplified for now
                    instructions: vec![], // Simplified for now
                    recent_blockhash: String::new(), // Simplified for now
                }),
            }),
        }
    }

    fn convert_meta_to_grpc(meta: &solana_transaction_status::UiTransactionStatusMeta) -> Meta {
        Meta {
            compute_units_consumed: 0, // Simplified for now
            err: meta.err.as_ref().map(|e| format!("{:?}", e)),
            fee: meta.fee,
            inner_instructions: vec![], // Simplified for now
            log_messages: vec![], // Simplified for now
            post_balances: meta.post_balances.clone(),
            post_token_balances: vec![], // Simplified for now
            pre_balances: meta.pre_balances.clone(),
            pre_token_balances: vec![], // Simplified for now
            rewards: vec![], // Simplified for now
            status: None, // Simplified for now
        }
    }
}
