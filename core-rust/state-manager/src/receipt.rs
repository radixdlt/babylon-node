use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_queries::typed_substate_layout::EpochChangeEvent;

use radix_engine::errors::RuntimeError;
use radix_engine::system::system_modules::costing::FeeSummary;

use radix_engine::transaction::{
    CommitResult, EventSystemStructure, StateUpdateSummary, SubstateSystemStructure,
    TransactionOutcome,
};
use radix_engine::types::*;

use radix_engine_interface::types::EventTypeIdentifier;
use radix_engine_store_interface::interface::DbSubstateValue;
use sbor::rust::collections::IndexMap;

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore};
use crate::accumulator_tree::tree_builder::{AccuTree, Merklizable};
use crate::limits::ExecutionMetrics;
use crate::transaction::PayloadIdentifiers;
use crate::{ConsensusReceipt, EventHash, LedgerHashes, SubstateChangeHash, SubstateReference};

#[derive(Debug, Clone, Sbor)]
pub struct CommittedTransactionIdentifiers {
    pub payload: PayloadIdentifiers,
    pub resultant_ledger_hashes: LedgerHashes,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SubstateChange {
    pub node_id: NodeId,
    pub partition_number: PartitionNumber,
    pub substate_key: SubstateKey,
    pub action: ChangeAction,
}

impl From<(SubstateReference, ChangeAction)> for SubstateChange {
    fn from((substate_reference, action): (SubstateReference, ChangeAction)) -> Self {
        let SubstateReference(node_id, partition_number, substate_key) = substate_reference;
        Self {
            node_id,
            partition_number,
            substate_key,
            action,
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ChangeAction {
    Create(DbSubstateValue),
    Update {
        new: DbSubstateValue,
        previous: DbSubstateValue,
    },
    Delete,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ApplicationEvent {
    pub type_id: EventTypeIdentifier,
    pub data: Vec<u8>,
}

impl ApplicationEvent {
    pub fn new(type_id: EventTypeIdentifier, data: Vec<u8>) -> Self {
        Self { type_id, data }
    }

    /// Computes a hash of this event, to be used in the events' merkle tree.
    pub fn get_hash(&self) -> EventHash {
        EventHash::from(hash(scrypto_encode(self).unwrap()))
    }
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

impl LedgerTransactionOutcome {
    fn resolve(outcome: &TransactionOutcome) -> Self {
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

/// A committed transaction (success or failure), extracted from the Engine's `TransactionReceipt`
/// of any locally-executed transaction (slightly post-processed).
/// It contains all the critical, deterministic pieces of the Engine's receipt, but also some of its
/// other parts - for this reason, it is very clearly split into 2 parts (on-ledger vs off-ledger).
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionReceipt {
    pub on_ledger: LedgerTransactionReceipt,
    pub local_execution: LocalTransactionExecution,
}

/// A part of the `LocalTransactionReceipt` which is completely stored on ledger. It contains only
/// the critical, deterministic pieces of the original Engine's `TransactionReceipt`.
/// All these pieces can be verified against the Receipt Root hash (found in the Ledger Proof).
/// Note: the Ledger Receipt is still a pretty large structure (i.e. containing entire collections,
/// like substate changes) and is not supposed to be hashed directly - it should instead go through
/// a `Consensus Receipt`.
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerTransactionReceipt {
    /// A simple, high-level outcome of the transaction.
    /// Its omitted details may be found in `LocalTransactionExecution::outcome`.
    pub outcome: LedgerTransactionOutcome,
    /// The substate changes resulting from the transaction.
    pub substate_changes: BySubstate<ChangeAction>,
    /// The events emitted during the transaction, in the order they occurred.
    pub application_events: Vec<ApplicationEvent>,
}

/// A computable/non-critical/non-deterministic part of the `LocalTransactionReceipt` (e.g. logs,
/// summaries).
/// It is not verifiable against ledger, but may still be useful for debugging.
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionExecution {
    pub outcome: DetailedTransactionOutcome,
    pub execution_metrics: ExecutionMetrics,
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
    pub state_update_summary: StateUpdateSummary,
    pub substates_system_structure: BySubstate<SubstateSystemStructure>,
    pub events_system_structure: IndexMap<EventTypeIdentifier, EventSystemStructure>,
    pub next_epoch: Option<EpochChangeEvent>,
}

impl LedgerTransactionReceipt {
    pub fn get_consensus_receipt(&self) -> ConsensusReceipt {
        let LedgerTransactionReceipt {
            outcome,
            substate_changes,
            application_events,
        } = self;
        ConsensusReceipt {
            outcome: outcome.clone(),
            substate_change_root: compute_merkle_root(
                substate_changes
                    .iter()
                    .map(|(sub_ref, action)| SubstateChange::from((sub_ref, action.clone())))
                    .map(|substate_change| {
                        SubstateChangeHash::from_substate_change(&substate_change)
                    })
                    .collect(),
            ),
            event_root: compute_merkle_root(
                application_events
                    .iter()
                    .map(|application_event| application_event.get_hash())
                    .collect(),
            ),
        }
    }
}

impl From<(CommitResult, BySubstate<ChangeAction>)> for LocalTransactionReceipt {
    fn from((commit_result, substate_changes): (CommitResult, BySubstate<ChangeAction>)) -> Self {
        let next_epoch = commit_result.next_epoch();
        let system_structure = commit_result.system_structure;
        Self {
            on_ledger: LedgerTransactionReceipt {
                outcome: LedgerTransactionOutcome::resolve(&commit_result.outcome),
                substate_changes,
                application_events: commit_result
                    .application_events
                    .into_iter()
                    .map(|(type_id, data)| ApplicationEvent::new(type_id, data))
                    .collect(),
            },
            local_execution: LocalTransactionExecution {
                execution_metrics: ExecutionMetrics::new_from_commit(&commit_result.fee_summary),
                outcome: commit_result.outcome.into(),
                fee_summary: commit_result.fee_summary,
                application_logs: commit_result.application_logs,
                state_update_summary: commit_result.state_update_summary,
                substates_system_structure: BySubstate::wrap(
                    system_structure.substate_system_structures,
                ),
                events_system_structure: system_structure.event_system_structures,
                next_epoch,
            },
        }
    }
}

/// A container of items associated with a specific substate.
/// This simply offers a less wasteful representation of a `Vec<(SubstateReference, T)>`, by
/// avoiding the repeated [`NodeId`]s and [`PartitionNumber`]s (within [`SubstateReference`]s).
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BySubstate<T> {
    by_node_id: IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, T>>>,
}

impl<T> BySubstate<T> {
    pub fn new() -> Self {
        Self::wrap(index_map_new())
    }

    pub fn add(
        &mut self,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        item: T,
    ) {
        self.by_node_id
            .entry(*node_id)
            .or_insert(index_map_new())
            .entry(*partition_number)
            .or_insert(index_map_new())
            .insert(substate_key.clone(), item);
    }

    pub fn iter(&self) -> impl Iterator<Item = (SubstateReference, &T)> + '_ {
        self.by_node_id
            .iter()
            .flat_map(|(node_id, by_partition_number)| {
                by_partition_number
                    .iter()
                    .flat_map(|(partition_number, by_substate_key)| {
                        by_substate_key.iter().map(|(substate_key, action)| {
                            (
                                SubstateReference(
                                    *node_id,
                                    *partition_number,
                                    substate_key.clone(),
                                ),
                                action,
                            )
                        })
                    })
            })
    }

    pub fn len(&self) -> usize {
        self.by_node_id
            .values()
            .map(|by_partition_number| {
                by_partition_number
                    .values()
                    .map(|by_substate_key| by_substate_key.len())
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.by_node_id.is_empty()
    }

    fn wrap(
        by_node_id: IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, T>>>,
    ) -> Self {
        Self { by_node_id }
    }
}

impl<T> Default for BySubstate<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructs a transient merkle tree on top of the given leaves, and returns its root only.
/// Returns a `Merklizable::zero()` if the tree is empty.
fn compute_merkle_root<M: Merklizable>(leaves: Vec<M>) -> M {
    if leaves.is_empty() {
        return M::zero();
    }
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
