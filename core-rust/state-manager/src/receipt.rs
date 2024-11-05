use crate::prelude::*;

define_versioned! {
    #[derive(Debug, Clone, Sbor)]
    pub VersionedCommittedTransactionIdentifiers(CommittedTransactionIdentifiersVersions) {
        previous_versions: [
            1 => CommittedTransactionIdentifiersV1: { updates_to: 2 },
        ],
        latest_version: {
            2 => CommittedTransactionIdentifiers = CommittedTransactionIdentifiersV2,
        },
    },
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(
            backwards_compatible(
                bottlenose = "FILE:CF_SCHEMA_versioned_committed_transaction_identifiers_bottlenose.bin",
                cuttlefish = "FILE:CF_SCHEMA_versioned_committed_transaction_identifiers_cuttlefish.bin"
            ),
            settings(allow_name_changes),
        )]
    ]
}

#[derive(Debug, Clone, Sbor)]
pub struct CommittedTransactionIdentifiersV1 {
    pub transaction_hashes: LedgerTransactionHashesV1,
    pub resultant_ledger_hashes: LedgerHashes,
    pub proposer_timestamp_ms: i64,
}

#[derive(Debug, Clone, Sbor)]
pub struct CommittedTransactionIdentifiersV2 {
    pub transaction_hashes: LedgerTransactionHashesV2,
    pub resultant_ledger_hashes: LedgerHashes,
    pub proposer_timestamp_ms: i64,
}

impl From<CommittedTransactionIdentifiersV1> for CommittedTransactionIdentifiersV2 {
    fn from(value: CommittedTransactionIdentifiersV1) -> Self {
        let CommittedTransactionIdentifiersV1 {
            transaction_hashes,
            resultant_ledger_hashes,
            proposer_timestamp_ms,
        } = value;
        CommittedTransactionIdentifiersV2 {
            transaction_hashes: transaction_hashes.into(),
            resultant_ledger_hashes,
            proposer_timestamp_ms,
        }
    }
}

/// A "flat" representation of an entire Partition's change, suitable for merkle hash computation.
#[derive(Debug, Clone, ScryptoSbor)]
pub struct PartitionChange {
    pub node_id: NodeId,
    pub partition_num: PartitionNumber,
    pub action: PartitionChangeAction,
}

impl From<(PartitionReference, PartitionChangeAction)> for PartitionChange {
    fn from((partition_reference, action): (PartitionReference, PartitionChangeAction)) -> Self {
        let PartitionReference(node_id, partition_num) = partition_reference;
        Self {
            node_id,
            partition_num,
            action,
        }
    }
}

/// A "flat" representation of a single substate's change, suitable for merkle hash computation.
#[derive(Debug, Clone, ScryptoSbor)]
pub struct SubstateChange {
    pub node_id: NodeId,
    pub partition_num: PartitionNumber,
    pub substate_key: SubstateKey,
    pub action: SubstateChangeAction,
}

impl From<(SubstateReference, SubstateChangeAction)> for SubstateChange {
    fn from((substate_reference, action): (SubstateReference, SubstateChangeAction)) -> Self {
        let SubstateReference(node_id, partition_num, substate_key) = substate_reference;
        Self {
            node_id,
            partition_num,
            substate_key,
            action,
        }
    }
}

/// An on-ledger change of an entire partition.
#[derive(Debug, Clone, ScryptoSbor)]
pub enum PartitionChangeAction {
    /// Deletion of an entire Partition.
    /// Note: contrary to [`SubstateChangeAction`]s, the previous contents of the Partition are not
    /// captured here.
    Delete,
}

/// An on-ledger change of an individual substate.
#[derive(Debug, Clone, ScryptoSbor)]
pub enum SubstateChangeAction {
    Create {
        /// A value after the transaction.
        new: DbSubstateValue,
    },
    Update {
        /// A value after the transaction.
        new: DbSubstateValue,
        /// A value before the transaction.
        /// *Important note:* this is *not* a "value before the substate change", but "before the
        /// transaction" - it may be especially visible if a partition is deleted, and then a
        /// substate inside is created under some key that existed before the transaction:
        /// technically, this was a creation (i.e. no previous value), but in the scope of its
        /// transaction it was an update (i.e. we can return its previous value).
        previous: DbSubstateValue,
    },
    Delete {
        /// A value before the transaction.
        previous: DbSubstateValue,
    },
}

#[derive(Debug, Clone, ScryptoSbor)]
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

#[derive(Debug, Clone, ScryptoSbor)]
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

#[derive(Debug, Clone, ScryptoSbor)]
pub enum DetailedTransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(LenientRuntimeError),
}

/// A wrapper for SBOR-encoded [`RuntimeError`] which may turn out to no longer be decodable (due
/// to differences in historical error enum schema).
#[derive(Debug, Clone, ScryptoEncode, ScryptoDecode, ScryptoDescribe)]
#[sbor(transparent)]
pub struct LenientRuntimeError(ScryptoValue);

impl Categorize<ScryptoCustomValueKind> for LenientRuntimeError {
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        // We know for a fact that the `RuntimeError` was at least always an enum...
        ValueKind::Enum
    }
}

impl From<RuntimeError> for LenientRuntimeError {
    fn from(error: RuntimeError) -> Self {
        let bytes = scrypto_encode(&error).unwrap();
        Self(scrypto_decode(&bytes).unwrap())
    }
}

impl LenientRuntimeError {
    /// Performs a best-effort rendering of the wrapped error.
    /// This will either be a debug-formatted [`RuntimeError`] (if it can be successfully decoded),
    /// or `UnknownError(DecodeError(...), <hex-encoded error bytes>)` otherwise.
    pub fn render(&self) -> String {
        let bytes = scrypto_encode(&self.0).unwrap();
        scrypto_decode::<RuntimeError>(&bytes)
            .map(|original_error| format!("{:?}", original_error))
            .unwrap_or_else(|decode_error| {
                format!("UnknownError({:?}, {})", decode_error, hex::encode(&bytes))
            })
    }
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
            TransactionOutcome::Failure(error) => Self::Failure(LenientRuntimeError::from(error)),
        }
    }
}

/// A "flattened", unambiguous representation of state changes resulting from a transaction,
/// suitable for merkle hash computation (to be recorded on-ledger).
#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct LedgerStateChanges {
    /// Changes applied on Partition level, affecting all substates of a Partition *except* the
    /// ones referenced in [`substate_level_changes`].
    pub partition_level_changes: ByPartition<PartitionChangeAction>,
    /// Changes applied to individual substates.
    pub substate_level_changes: BySubstate<SubstateChangeAction>,
}

impl LedgerStateChanges {
    /// Returns hashes of all the contained changes in a predefined order, suitable for computing
    /// a merkle root hash.
    pub fn get_hashes(&self) -> Vec<StateChangeHash> {
        self.partition_level_changes
            .iter()
            .map(|(par_ref, action)| PartitionChange::from((par_ref, action.clone())))
            .map(|partition_change| StateChangeHash::from_partition_change(&partition_change))
            .chain(
                self.substate_level_changes
                    .iter()
                    .map(|(sub_ref, action)| SubstateChange::from((sub_ref, action.clone())))
                    .map(|substate_change| StateChangeHash::from_substate_change(&substate_change)),
            )
            .collect()
    }

    pub fn len(&self) -> usize {
        self.partition_level_changes.len() + self.substate_level_changes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.partition_level_changes.is_empty() && self.substate_level_changes.is_empty()
    }
}

/// A committed transaction (success or failure), extracted from the Engine's [`TransactionReceipt`]
/// of any locally-executed transaction (slightly post-processed).
///
/// It contains all the critical, deterministic pieces of the Engine's receipt, but also some of its
/// other parts - for this reason, it is very clearly split into 2 parts:
///
/// * The [`LedgerTransactionReceipt`] contains the parts of the receipt which are validated at
///   consensus (success/failure, state changes and emitted events).
/// * The [`LocalTransactionExecution`] contains other information which is useful for transaction
///   analysis and investigation.
#[derive(Debug, Clone)]
pub struct LocalTransactionReceipt {
    pub on_ledger: LedgerTransactionReceipt,
    pub local_execution: LocalTransactionExecution,
}

define_single_versioned! {
    #[derive(Debug, Clone, ScryptoSbor)]
    pub VersionedLedgerTransactionReceipt(LedgerTransactionReceiptVersions) => LedgerTransactionReceipt = LedgerTransactionReceiptV1,
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:CF_SCHEMA_versioned_ledger_transaction_receipt.bin"
        ))]
    ]
}

/// A part of the [`LocalTransactionReceipt`] which is completely stored on ledger. It contains only
/// the critical, deterministic pieces of the original Engine's `TransactionReceipt`.
///
/// All these pieces can be verified against the Receipt Root hash (found in the Ledger Proof).
///
/// Note: the [`LedgerTransactionReceipt`] is still a pretty large structure (i.e. containing entire collections,
/// like substate changes) and is not supposed to be hashed directly - it should instead go through
/// a [`ConsensusReceipt`].
#[derive(Debug, Clone, ScryptoSbor)]
pub struct LedgerTransactionReceiptV1 {
    /// A simple, high-level outcome of the transaction.
    /// Its omitted details may be found in `LocalTransactionExecution::outcome`.
    pub outcome: LedgerTransactionOutcome,
    /// The state changes resulting from the transaction.
    pub state_changes: LedgerStateChanges,
    /// The events emitted during the transaction, in the order they occurred.
    pub application_events: Vec<ApplicationEvent>,
}

define_versioned! {
    /// A computable/non-critical/non-deterministic part of the `LocalTransactionReceipt`
    /// (e.g. logs, summaries).
    /// It is not verifiable against ledger, but is still be useful for debugging.
    #[derive(Debug, Clone, ScryptoSbor)]
    pub VersionedLocalTransactionExecution(LocalTransactionExecutionVersions) {
        previous_versions: [
            1 => LocalTransactionExecutionV1: { updates_to: 2 },
        ],
        latest_version: {
            2 => LocalTransactionExecution = LocalTransactionExecutionV2,
        },
    },
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:CF_SCHEMA_versioned_local_transaction_execution.bin"
        ))]
    ]
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct LocalTransactionExecutionV1 {
    pub outcome: DetailedTransactionOutcome,
    pub fee_summary: TransactionFeeSummary,
    pub fee_source: FeeSource,
    pub fee_destination: FeeDestination,
    pub engine_costing_parameters: CostingParameters,
    pub transaction_costing_parameters: TransactionCostingParametersReceiptV1,
    pub application_logs: Vec<(crate::engine_prelude::Level, String)>,
    pub state_update_summary: StateUpdateSummary,
    pub global_balance_summary: GlobalBalanceSummary,
    pub substates_system_structure: BySubstate<SubstateSystemStructure>,
    pub events_system_structure: IndexMap<EventTypeIdentifier, EventSystemStructure>,
    pub next_epoch: Option<EpochChangeEvent>,
}

impl From<LocalTransactionExecutionV1> for LocalTransactionExecutionV2 {
    fn from(value: LocalTransactionExecutionV1) -> Self {
        LocalTransactionExecutionV2 {
            outcome: value.outcome,
            fee_summary: value.fee_summary,
            fee_source: value.fee_source,
            fee_destination: value.fee_destination,
            engine_costing_parameters: value.engine_costing_parameters,
            transaction_costing_parameters: value.transaction_costing_parameters.into(),
            application_logs: value.application_logs,
            state_update_summary: value.state_update_summary,
            global_balance_summary: value.global_balance_summary,
            substates_system_structure: value.substates_system_structure,
            events_system_structure: value.events_system_structure,
            next_epoch: value.next_epoch,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct LocalTransactionExecutionV2 {
    pub outcome: DetailedTransactionOutcome,
    pub fee_summary: TransactionFeeSummary,
    pub fee_source: FeeSource,
    pub fee_destination: FeeDestination,
    pub engine_costing_parameters: CostingParameters,
    pub transaction_costing_parameters: TransactionCostingParametersReceiptV2,
    pub application_logs: Vec<(crate::engine_prelude::Level, String)>,
    pub state_update_summary: StateUpdateSummary,
    pub global_balance_summary: GlobalBalanceSummary,
    pub substates_system_structure: BySubstate<SubstateSystemStructure>,
    pub events_system_structure: IndexMap<EventTypeIdentifier, EventSystemStructure>,
    pub next_epoch: Option<EpochChangeEvent>,
}

impl LedgerTransactionReceipt {
    pub fn get_consensus_receipt(&self) -> ConsensusReceipt {
        let LedgerTransactionReceipt {
            outcome,
            state_changes,
            application_events,
        } = self;
        ConsensusReceipt {
            outcome: outcome.clone(),
            substate_change_root: compute_merkle_root(state_changes.get_hashes()),
            event_root: compute_merkle_root(
                application_events
                    .iter()
                    .map(|application_event| application_event.get_hash())
                    .collect(),
            ),
        }
    }
}

pub struct ExecutionFeeData {
    pub fee_summary: TransactionFeeSummary,
    pub engine_costing_parameters: CostingParameters,
    pub transaction_costing_parameters: TransactionCostingParametersReceiptV2,
}

impl LocalTransactionReceipt {
    pub fn new(
        commit_result: CommitResult,
        state_changes: LedgerStateChanges,
        global_balance_summary: GlobalBalanceSummary,
        execution_fee_data: ExecutionFeeData,
    ) -> Self {
        let next_epoch = commit_result.next_epoch();
        let system_structure = commit_result.system_structure;
        Self {
            on_ledger: LedgerTransactionReceipt {
                outcome: LedgerTransactionOutcome::resolve(&commit_result.outcome),
                state_changes,
                application_events: commit_result
                    .application_events
                    .into_iter()
                    .map(|(type_id, data)| ApplicationEvent::new(type_id, data))
                    .collect(),
            },
            local_execution: LocalTransactionExecution {
                outcome: commit_result.outcome.into(),
                fee_summary: execution_fee_data.fee_summary,
                fee_source: commit_result.fee_source,
                fee_destination: commit_result.fee_destination,
                engine_costing_parameters: execution_fee_data.engine_costing_parameters,
                transaction_costing_parameters: execution_fee_data.transaction_costing_parameters,
                application_logs: commit_result.application_logs,
                state_update_summary: commit_result.state_update_summary,
                global_balance_summary,
                substates_system_structure: BySubstate::wrap(
                    system_structure.substate_system_structures,
                ),
                events_system_structure: system_structure.event_system_structures,
                next_epoch,
            },
        }
    }
}

/// A container of items associated with a specific partition.
/// This simply offers a less wasteful representation of a `Vec<(PartitionReference, T)>`, by
/// avoiding the repeated [`NodeId`]s (within [`PartitionReference`]s).
#[derive(Debug, Clone, ScryptoSbor)]
#[sbor(categorize_types = "T")]
pub struct ByPartition<T> {
    by_node_id: IndexMap<NodeId, IndexMap<PartitionNumber, T>>,
}

impl<T> ByPartition<T> {
    pub fn new() -> Self {
        Self {
            by_node_id: index_map_new(),
        }
    }

    pub fn add(&mut self, node_id: &NodeId, partition_num: &PartitionNumber, item: T) {
        self.by_node_id
            .entry(*node_id)
            .or_insert(index_map_new())
            .insert(*partition_num, item);
    }

    pub fn get(&self, node_id: &NodeId, partition_num: &PartitionNumber) -> Option<&T> {
        self.by_node_id
            .get(node_id)
            .and_then(|by_partition_num| by_partition_num.get(partition_num))
    }

    pub fn iter(&self) -> impl Iterator<Item = (PartitionReference, &T)> + '_ {
        self.by_node_id
            .iter()
            .flat_map(|(node_id, by_partition_num)| {
                by_partition_num.iter().map(|(partition_num, element)| {
                    (PartitionReference(*node_id, *partition_num), element)
                })
            })
    }

    pub fn len(&self) -> usize {
        self.by_node_id
            .values()
            .map(|by_partition_num| by_partition_num.len())
            .sum::<usize>()
    }

    pub fn is_empty(&self) -> bool {
        self.by_node_id.is_empty()
    }
}

impl<T> Default for ByPartition<T> {
    fn default() -> Self {
        Self {
            by_node_id: index_map_new(),
        }
    }
}

/// A container of items associated with a specific substate.
/// This simply offers a less wasteful representation of a `Vec<(SubstateReference, T)>`, by
/// avoiding the repeated [`NodeId`]s and [`PartitionNumber`]s (within [`SubstateReference`]s).
#[derive(Debug, Clone, ScryptoSbor)]
#[sbor(categorize_types = "T")]
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
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
        item: T,
    ) {
        self.by_node_id
            .entry(*node_id)
            .or_insert(index_map_new())
            .entry(*partition_num)
            .or_insert(index_map_new())
            .insert(substate_key.clone(), item);
    }

    pub fn get(
        &self,
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&T> {
        self.by_node_id
            .get(node_id)
            .and_then(|by_partition_num| by_partition_num.get(partition_num))
            .and_then(|by_substate_key| by_substate_key.get(substate_key))
    }

    pub fn iter(&self) -> impl Iterator<Item = (SubstateReference, &T)> + '_ {
        self.by_node_id
            .iter()
            .flat_map(|(node_id, by_partition_num)| {
                by_partition_num
                    .iter()
                    .flat_map(|(partition_num, by_substate_key)| {
                        by_substate_key.iter().map(|(substate_key, element)| {
                            (
                                SubstateReference(*node_id, *partition_num, substate_key.clone()),
                                element,
                            )
                        })
                    })
            })
    }

    pub fn iter_node_ids(&self) -> impl Iterator<Item = &NodeId> + '_ {
        self.by_node_id.keys()
    }

    pub fn len(&self) -> usize {
        self.by_node_id
            .values()
            .map(|by_partition_num| {
                by_partition_num
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
        Self {
            by_node_id: index_map_new(),
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
