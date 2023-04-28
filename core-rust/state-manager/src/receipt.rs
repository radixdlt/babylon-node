use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use std::collections::BTreeMap;

use radix_engine::errors::RuntimeError;
use radix_engine::system::system_modules::costing::FeeSummary;
use radix_engine::system::system_modules::execution_trace::ResourceChange;
use radix_engine::transaction::{
    CommitResult, StateUpdateSummary, TransactionExecutionTrace, TransactionOutcome,
};
use radix_engine::types::{hash, scrypto_encode, Decimal, Hash, Level};
use radix_engine_common::types::{ComponentAddress, ModuleId, NodeId, SubstateKey};

use radix_engine_interface::types::EventTypeIdentifier;
use radix_engine_interface::*;
use sbor::rust::collections::IndexMap;

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore};
use crate::accumulator_tree::tree_builder::{AccuTree, Merklizable};
use crate::{AccumulatorHash, ConsensusReceipt, EventHash, SubstateChangeHash};

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
    pub node_id: NodeId,
    pub module_id: ModuleId,
    pub substate_key: SubstateKey,
    pub action: ChangeAction,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ChangeAction {
    Create(Vec<u8>),
    Update(Vec<u8>),
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
    pub substate_changes: Vec<SubstateChange>,
    /// The events emitted during the transaction, in the order they occurred.
    pub application_events: Vec<ApplicationEvent>,
}

/// A computable/non-critical/non-deterministic part of the `LocalTransactionReceipt` (e.g. logs,
/// summaries).
/// It is not verifiable against ledger, but may still be useful for debugging.
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionExecution {
    pub outcome: DetailedTransactionOutcome,
    // The breakdown of the fee
    pub fee_summary: FeeSummary,
    // Which vault/s paid the fee
    pub fee_payments: IndexMap<NodeId, Decimal>,
    pub application_logs: Vec<(Level, String)>,
    pub state_update_summary: StateUpdateSummary,
    // These will be removed once we have the parent_map for the toolkit to use
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub next_epoch: Option<(BTreeMap<ComponentAddress, Validator>, u64)>,
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
                    .map(|substate_change| {
                        SubstateChangeHash::from_substate_change(substate_change)
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

// TODO: also add state changes
impl From<(CommitResult, Vec<SubstateChange>, TransactionExecutionTrace)>
    for LocalTransactionReceipt
{
    fn from(
        (commit_result, substate_changes, execution_trace): (
            CommitResult,
            Vec<SubstateChange>,
            TransactionExecutionTrace,
        ),
    ) -> Self {
        let next_epoch = commit_result.next_epoch();
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
                outcome: commit_result.outcome.into(),
                fee_summary: commit_result.fee_summary,
                fee_payments: commit_result.fee_payments,
                application_logs: commit_result.application_logs,
                state_update_summary: commit_result.state_update_summary,
                resource_changes: execution_trace.resource_changes,
                next_epoch,
            },
        }
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
