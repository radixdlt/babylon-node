use std::collections::BTreeMap;

use radix_engine::engine::RuntimeError;
use radix_engine::fee::FeeSummary;
use radix_engine::ledger::OutputValue;
use radix_engine::model::ResourceChange;
use radix_engine::state_manager::StateDiff;
use radix_engine::transaction::{
    CommitResult, EntityChanges, TransactionOutcome,
    TransactionReceipt as EngineTransactionReceipt, TransactionResult,
};
use radix_engine::types::{hash, scrypto_encode, Hash, Level, SubstateId};
use radix_engine_interface::scrypto;
use sbor::*;

use crate::AccumulatorHash;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct CommittedTransactionIdentifiers {
    pub state_version: u64,
    pub accumulator_hash: AccumulatorHash,
}

impl CommittedTransactionIdentifiers {
    pub fn pre_genesis() -> Self {
        Self {
            state_version: 0,
            accumulator_hash: AccumulatorHash::pre_genesis(),
        }
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum CommittedTransactionStatus {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct SubstateChanges {
    pub created: BTreeMap<SubstateId, OutputValue>,
    pub updated: BTreeMap<SubstateId, OutputValue>,
    pub deleted: BTreeMap<SubstateId, DeletedSubstateVersion>,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct DeletedSubstateVersion {
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum LedgerTransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

impl From<TransactionOutcome> for LedgerTransactionOutcome {
    fn from(outcome: TransactionOutcome) -> Self {
        match outcome {
            TransactionOutcome::Success(output) => {
                LedgerTransactionOutcome::Success(output.into_iter().map(|o| o.as_vec()).collect())
            }
            TransactionOutcome::Failure(error) => LedgerTransactionOutcome::Failure(error),
        }
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct LedgerTransactionReceipt {
    pub outcome: LedgerTransactionOutcome,
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
    pub substate_changes: SubstateChanges,
    pub entity_changes: EntityChanges,
    pub resource_changes: Vec<ResourceChange>,
}

impl TryFrom<EngineTransactionReceipt> for LedgerTransactionReceipt {
    type Error = String;

    fn try_from(engine_receipt: EngineTransactionReceipt) -> Result<Self, Self::Error> {
        match engine_receipt.result {
            TransactionResult::Commit(commit_result) => {
                Ok((commit_result, engine_receipt.execution.fee_summary).into())
            }
            TransactionResult::Reject(error) => Err(format!(
                "Can't create a ledger receipt for rejected txn: {:?}",
                error
            )),
        }
    }
}

/// For Genesis Transaction
impl From<(CommitResult, FeeSummary)> for LedgerTransactionReceipt {
    fn from((commit_result, fee_summary): (CommitResult, FeeSummary)) -> Self {
        LedgerTransactionReceipt {
            outcome: commit_result.outcome.into(),
            fee_summary,
            application_logs: commit_result.application_logs,
            substate_changes: map_state_updates(commit_result.state_updates),
            entity_changes: commit_result.entity_changes,
            resource_changes: commit_result.resource_changes,
        }
    }
}

fn map_state_updates(state_updates: StateDiff) -> SubstateChanges {
    // As of end of August 2022, the engine's statediff erroneously includes substate reads
    // (even if the content didn't change) as ups and downs.
    // This needs fixing, but for now, we work around this here, by removing such up/down pairs.
    let mut possible_creations = state_updates.up_substates;
    let mut updated = BTreeMap::<SubstateId, OutputValue>::new();
    let mut deleted = BTreeMap::<SubstateId, DeletedSubstateVersion>::new();

    // We iterate over the downed substates, and attempt to match them with an upped substate
    // > If they match an upped substate, the down and up is erroneous, so we ignore both
    // > If it doesn't match, this is correct and is added to valid_down_substates
    for down_substate_output_id in state_updates.down_substates {
        let substate_id = down_substate_output_id.substate_id;
        let down_substate_hash = down_substate_output_id.substate_hash;
        match possible_creations.remove(&substate_id) {
            Some(up_substate_output_value) => {
                // TODO - this check can be removed when the bug is fixed
                let up_substate_hash =
                    hash(&scrypto_encode(&up_substate_output_value.substate).unwrap());
                if up_substate_hash != down_substate_hash {
                    updated.insert(substate_id, up_substate_output_value);
                } else {
                    // Do nothing - this is erroneous
                }
            }
            None => {
                deleted.insert(
                    substate_id,
                    DeletedSubstateVersion {
                        substate_hash: down_substate_hash,
                        version: down_substate_output_id.version,
                    },
                );
            }
        }
    }

    // The remaining up_substates which didn't match with a down_substate are all creates
    let created = possible_creations;

    SubstateChanges {
        created,
        updated,
        deleted,
    }
}
