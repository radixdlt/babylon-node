use radix_engine::engine::ResourceChange;
use radix_engine::fee::FeeSummary;
use radix_engine::ledger::{OutputId, OutputValue};
use radix_engine::state_manager::StateDiff;
use radix_engine::transaction::{
    EntityChanges, TransactionOutcome, TransactionReceipt as EngineTransactionReceipt,
    TransactionResult,
};
use sbor::{Decode, Encode, TypeId};
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::hash;
use scrypto::engine::types::SubstateId;
use scrypto::prelude::Level;
use std::collections::BTreeMap;

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
                state_updates: filter_state_updates(commit_result.state_updates),
                entity_changes: commit_result.entity_changes,
                resource_changes: commit_result.resource_changes,
            }),
            TransactionResult::Reject(_) => {
                Err("Can't create a ledger receipt for rejected txn".to_string())
            }
        }
    }
}

fn filter_state_updates(state_updates: StateDiff) -> StateDiff {
    // This is a temporary filter that removes any substates
    // that have been downed and then upped unchanged.
    let mut filtered_up_substates: BTreeMap<SubstateId, OutputValue> = BTreeMap::new();
    let mut filtered_down_substates: Vec<OutputId> = state_updates.down_substates;

    for (substate_id, output_value) in state_updates.up_substates {
        if output_value.version == 0 {
            // This is a new substate, all good, keep it
            filtered_up_substates.insert(substate_id, output_value);
        } else {
            // Check if a previous version of this substate was downed
            // but it had the exact same data (same hash)
            let expected_down_output_id = OutputId {
                substate_id: substate_id.clone(),
                substate_hash: hash(&scrypto_encode(&output_value.substate)),
                version: output_value.version - 1,
            };

            match filtered_down_substates
                .iter()
                .position(|x| x == &expected_down_output_id)
            {
                Some(pos) => {
                    // If so, this was just a "read":
                    //   remove down substate from the receipt
                    //   and do NOT add the up substate
                    filtered_down_substates.remove(pos);
                }
                None => {
                    // This is a legit substate update:
                    //   add to up substates and do NOT remove the down substate
                    filtered_up_substates.insert(substate_id, output_value);
                }
            }
        }
    }

    StateDiff {
        down_virtual_substates: state_updates.down_virtual_substates,
        up_substates: filtered_up_substates,
        down_substates: filtered_down_substates,
        new_roots: state_updates.new_roots,
    }
}
