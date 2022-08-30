use radix_engine::engine::ResourceChange;
use radix_engine::fee::FeeSummary;
use radix_engine::state_manager::StateDiff;
use radix_engine::transaction::{
    EntityChanges, TransactionOutcome, TransactionReceipt as EngineTransactionReceipt,
    TransactionResult,
};
use sbor::{Decode, Encode, TypeId};
use scrypto::prelude::Level;

#[derive(Debug, Decode, Encode, TypeId)]
pub enum CommittedTransactionStatus {
    Success(Vec<Vec<u8>>),
    Failure(String),
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct LedgerTransactionReceipt {
    pub status: CommittedTransactionStatus,
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
    pub state_updates: StateDiff,
    pub entity_changes: EntityChanges,
    pub resource_changes: Vec<ResourceChange>,
}

impl TryFrom<EngineTransactionReceipt> for LedgerTransactionReceipt {
    type Error = String;

    fn try_from(engine_receipt: EngineTransactionReceipt) -> Result<Self, Self::Error> {
        match engine_receipt.result {
            TransactionResult::Commit(commit_result) => Ok(LedgerTransactionReceipt {
                status: match commit_result.outcome {
                    TransactionOutcome::Success(output) => {
                        CommittedTransactionStatus::Success(output)
                    }
                    TransactionOutcome::Failure(error) => {
                        CommittedTransactionStatus::Failure(format!("{:?}", error))
                    }
                },
                fee_summary: engine_receipt.execution.fee_summary,
                application_logs: engine_receipt.execution.application_logs,
                state_updates: commit_result.state_updates,
                entity_changes: commit_result.entity_changes,
                resource_changes: commit_result.resource_changes,
            }),
            TransactionResult::Reject(_) => {
                Err("Can't create a ledger receipt for rejected txn".to_string())
            }
        }
    }
}
