use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine::blueprints::transaction_processor::InstructionOutput;
use std::collections::BTreeMap;

use radix_engine::errors::RuntimeError;
use radix_engine::ledger::OutputValue;
use radix_engine::state_manager::StateDiff;
use radix_engine::system::kernel_modules::costing::FeeSummary;
use radix_engine::system::kernel_modules::execution_trace::ResourceChange;
use radix_engine::transaction::{
    CommitResult, EntityChanges, TransactionOutcome,
    TransactionReceipt as EngineTransactionReceipt, TransactionResult,
};
use radix_engine::types::{hash, scrypto_encode, Hash, SubstateId};
use radix_engine_common::crypto::blake2b_256_hash;
use radix_engine_interface::blueprints::logger::Level;
use radix_engine_interface::data::scrypto::model::ComponentAddress;
use radix_engine_interface::*;
use sbor::rust::collections::IndexMap;

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore};
use crate::accumulator_tree::tree_builder::{AccuTree, Merklizable};
use crate::{AccumulatorHash, ConsensusReceipt, SubstateChangeHash};

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SubstateChange {
    pub substate_id: SubstateId,
    pub action: ChangeAction,
}

impl SubstateChange {
    pub fn new(substate_id: SubstateId, action: ChangeAction) -> Self {
        Self {
            substate_id,
            action,
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ChangeAction {
    Create(OutputValue),
    Update(OutputValue),
    Delete(DeletedSubstateVersion),
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct DeletedSubstateVersion {
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum LedgerTransactionOutcome {
    Success,
    Failure,
}

impl From<&TransactionOutcome> for LedgerTransactionOutcome {
    fn from(outcome: &TransactionOutcome) -> Self {
        match outcome {
            TransactionOutcome::Success(_) => LedgerTransactionOutcome::Success,
            TransactionOutcome::Failure(_) => LedgerTransactionOutcome::Failure,
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum DetailedTransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

impl From<TransactionOutcome> for DetailedTransactionOutcome {
    fn from(outcome: TransactionOutcome) -> Self {
        match outcome {
            TransactionOutcome::Success(output) => {
                Self::Success(
                    output
                        .into_iter()
                        .map(|o| {
                            // TODO: Clean this up
                            match o {
                                InstructionOutput::None => scrypto_encode(&()).unwrap(),
                                InstructionOutput::CallReturn(v) => v,
                            }
                        })
                        .collect(),
                )
            }
            TransactionOutcome::Failure(error) => Self::Failure(error),
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionReceipt {
    pub on_ledger: LedgerTransactionReceipt,
    pub local_execution: LocalTransactionExecution,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerTransactionReceipt {
    pub outcome: LedgerTransactionOutcome,
    pub substate_changes: Vec<SubstateChange>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionExecution {
    pub outcome: DetailedTransactionOutcome,
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
    pub entity_changes: EntityChanges,
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub next_epoch: Option<(BTreeMap<ComponentAddress, Validator>, u64)>,
}

impl LedgerTransactionReceipt {
    pub fn get_consensus_receipt(&self) -> ConsensusReceipt {
        let substate_change_hashes = self
            .substate_changes
            .iter()
            .map(|substate_change| scrypto_encode(substate_change).unwrap())
            .map(|change_bytes| SubstateChangeHash::from(blake2b_256_hash(change_bytes)))
            .collect::<Vec<_>>();
        ConsensusReceipt {
            outcome: self.outcome.clone(),
            substate_change_root: compute_merkle_root(substate_change_hashes),
        }
    }
}

impl TryFrom<EngineTransactionReceipt> for LocalTransactionReceipt {
    type Error = String;

    fn try_from(engine_receipt: EngineTransactionReceipt) -> Result<Self, Self::Error> {
        match engine_receipt.result {
            TransactionResult::Commit(commit_result) => {
                Ok((commit_result, engine_receipt.execution.fee_summary).into())
            }
            TransactionResult::Reject(error) => Err(format!(
                "Can't create a ledger receipt for rejected txn: {error:?}"
            )),
            TransactionResult::Abort(result) => Err(format!(
                "Can't create a ledger receipt for aborted txn: {result:?}"
            )),
        }
    }
}

/// For Genesis Transaction
impl From<(CommitResult, FeeSummary)> for LocalTransactionReceipt {
    fn from((commit_result, fee_summary): (CommitResult, FeeSummary)) -> Self {
        Self {
            on_ledger: LedgerTransactionReceipt {
                outcome: LedgerTransactionOutcome::from(&commit_result.outcome),
                substate_changes: map_state_updates(commit_result.state_updates),
            },
            local_execution: LocalTransactionExecution {
                outcome: commit_result.outcome.into(),
                fee_summary,
                application_logs: commit_result.application_logs,
                entity_changes: commit_result.entity_changes,
                resource_changes: commit_result.resource_changes,
                next_epoch: commit_result.next_epoch,
            },
        }
    }
}

fn map_state_updates(state_updates: StateDiff) -> Vec<SubstateChange> {
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
                    hash(scrypto_encode(&up_substate_output_value.substate).unwrap());
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

    into_change_list(created, updated, deleted)
}

/// Turns the sets of changes (of different kind) into a flat list of `SubstateChange`s, ordered by
/// `SubstateId` (i.e. suitable for merklization).
fn into_change_list(
    created: BTreeMap<SubstateId, OutputValue>,
    updated: BTreeMap<SubstateId, OutputValue>,
    deleted: BTreeMap<SubstateId, DeletedSubstateVersion>,
) -> Vec<SubstateChange> {
    let mut changes = created
        .into_iter()
        .map(|(id, value)| SubstateChange::new(id, ChangeAction::Create(value)))
        .chain(
            updated
                .into_iter()
                .map(|(id, value)| SubstateChange::new(id, ChangeAction::Update(value))),
        )
        .chain(
            deleted
                .into_iter()
                .map(|(id, version)| SubstateChange::new(id, ChangeAction::Delete(version))),
        )
        .collect::<Vec<_>>();
    changes.sort_by(|left, right| left.substate_id.cmp(&right.substate_id));
    changes
}

fn compute_merkle_root<M: Merklizable>(leaves: Vec<M>) -> M {
    let mut store = RootCapturingAccuTreeStore::default();
    let mut tree = AccuTree::new(&mut store, 0);
    tree.append(leaves);
    store.into_captured_root()
}

struct RootCapturingAccuTreeStore<M> {
    captured: Option<M>,
}

impl<M> RootCapturingAccuTreeStore<M> {
    pub fn into_captured_root(self) -> M {
        self.captured.expect("not captured yet")
    }
}

impl<M> Default for RootCapturingAccuTreeStore<M> {
    fn default() -> Self {
        Self { captured: None }
    }
}

impl<M: Merklizable> ReadableAccuTreeStore<usize, M> for RootCapturingAccuTreeStore<M> {
    fn get_tree_slice(&self, key: &usize) -> Option<TreeSlice<M>> {
        panic!("unexpected get of slice {key}, since the build should be one-shot")
    }
}

impl<M: Merklizable> WriteableAccuTreeStore<usize, M> for RootCapturingAccuTreeStore<M> {
    fn put_tree_slice(&mut self, _key: usize, slice: TreeSlice<M>) {
        if self.captured.is_some() {
            panic!("unexpected repeated put, since the build should be one-shot")
        }
        self.captured = Some(
            slice
                .levels
                .into_iter()
                .next_back()
                .unwrap()
                .nodes
                .into_iter()
                .next()
                .unwrap(),
        )
    }
}
