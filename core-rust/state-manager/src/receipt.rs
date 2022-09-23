use radix_engine::engine::ResourceChange;
use radix_engine::fee::FeeSummary;
use radix_engine::ledger::OutputId;
use radix_engine::state_manager::StateDiff;
use radix_engine::transaction::{
    CommitResult, EntityChanges, TransactionOutcome,
    TransactionReceipt as EngineTransactionReceipt, TransactionResult,
};
use radix_engine::types::hash;
use sbor::{Decode, Encode, TypeId};
use scrypto::buffer::scrypto_encode;
use scrypto::prelude::Level;

#[derive(Debug, Decode, Encode, TypeId)]
pub struct CommittedTransactionIdentifiers {
    pub state_version: u64,
}

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
            TransactionResult::Commit(commit_result) => Ok((
                commit_result,
                engine_receipt.execution.fee_summary,
                engine_receipt.execution.application_logs,
            )
                .into()),
            TransactionResult::Reject(error) => {
                Err(format!("Can't create a ledger receipt for rejected txn: {:?}", error))
            }
        }
    }
}

/// For Genesis Transaction
impl From<(CommitResult, FeeSummary, Vec<(Level, String)>)> for LedgerTransactionReceipt {
    fn from(
        (commit_result, fee_summary, application_logs): (
            CommitResult,
            FeeSummary,
            Vec<(Level, String)>,
        ),
    ) -> Self {
        LedgerTransactionReceipt {
            status: match commit_result.outcome {
                TransactionOutcome::Success(output) => CommittedTransactionStatus::Success(output),
                TransactionOutcome::Failure(error) => {
                    CommittedTransactionStatus::Failure(format!("{:?}", error))
                }
            },
            fee_summary,
            application_logs,
            state_updates: filter_state_updates(commit_result.state_updates),
            entity_changes: commit_result.entity_changes,
            resource_changes: commit_result.resource_changes,
        }
    }
}

/// As of end of August 2022, the engine's statediff erroneously includes substate reads
/// (even if the content didn't change) as ups and downs.
///
/// This needs fixing, but for now, we work around this here, by removing such up/down pairs.
fn filter_state_updates(state_updates: StateDiff) -> StateDiff {
    let mut possible_up_substates = state_updates.up_substates;
    let mut valid_down_substates: Vec<OutputId> = Vec::new();

    // We iterate over the downed substates, and attempt to match them with an upped substate
    // > If they match an upped substate, the down and up is erroneous, so we ignore both
    // > If it doesn't match, this is correct and is added to valid_down_substates
    for down_substate_output_id in state_updates.down_substates {
        match possible_up_substates.get(&down_substate_output_id.substate_id) {
            Some(up_substate_output_value) => {
                let up_substate_hash = hash(&scrypto_encode(&up_substate_output_value.substate));
                if up_substate_hash == down_substate_output_id.substate_hash {
                    possible_up_substates.remove(&down_substate_output_id.substate_id);
                } else {
                    valid_down_substates.push(down_substate_output_id);
                }
            }
            None => {
                valid_down_substates.push(down_substate_output_id);
            }
        }
    }

    StateDiff {
        down_virtual_substates: state_updates.down_virtual_substates,
        up_substates: possible_up_substates,
        down_substates: valid_down_substates,
        new_roots: state_updates.new_roots,
    }
}
