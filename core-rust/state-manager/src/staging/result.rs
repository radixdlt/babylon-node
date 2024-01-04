/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

use super::ReadableStateTreeStore;
use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore};

use crate::staging::epoch_handling::EpochAwareAccuTreeFactory;
use crate::transaction::LedgerTransactionHash;
use crate::{
    compute_new_protocol_state, ActiveValidatorInfo, ByPartition, BySubstate,
    DetailedTransactionOutcome, EpochTransactionIdentifiers, LedgerHashes, LedgerStateChanges,
    LocalTransactionReceipt, NextEpoch, PartitionChangeAction, ProtocolState, ReceiptTreeHash,
    StateHash, StateVersion, SubstateChangeAction, SubstateReference, TransactionTreeHash,
};
use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::blueprints::resource::{FungibleVaultBalanceFieldSubstate, FungibleVaultField};
use radix_engine::transaction::{
    AbortResult, BalanceChange, CommitResult, CostingParameters, RejectResult,
    TransactionFeeSummary, TransactionReceipt, TransactionResult,
};
use radix_engine_interface::prelude::*;

use crate::staging::ReadableStore;

use radix_engine::track::{
    BatchPartitionStateUpdate, NodeStateUpdates, PartitionStateUpdates, StateUpdates,
};

use crate::staging::node_ancestry_resolver::NodeAncestryResolver;
use crate::staging::overlays::{MapSubstateNodeAncestryStore, StagedSubstateNodeAncestryStore};
use crate::store::traits::{KeyedSubstateNodeAncestryRecord, SubstateNodeAncestryStore};
use node_common::utils::IsAccountExt;
use radix_engine_store_interface::db_key_mapper::*;
use radix_engine_store_interface::interface::*;
use radix_engine_stores::hash_tree::tree_store::*;
use radix_engine_stores::hash_tree::*;
use transaction::prelude::TransactionCostingParameters;

pub enum ProcessedTransactionReceipt {
    Commit(ProcessedCommitResult),
    Reject(ProcessedRejectResult),
    Abort(AbortResult),
}

#[derive(Clone, Debug)]
pub struct ProcessedRejectResult {
    pub result: RejectResult,
    pub fee_summary: TransactionFeeSummary,
}

#[derive(Clone, Debug)]
pub struct ProcessedCommitResult {
    pub local_receipt: LocalTransactionReceipt,
    pub hash_structures_diff: HashStructuresDiff,
    pub database_updates: DatabaseUpdates,
    pub new_substate_node_ancestry_records: Vec<KeyedSubstateNodeAncestryRecord>,
    pub new_protocol_state: ProtocolState,
    pub next_protocol_version: Option<String>,
}

pub struct HashUpdateContext<'s, S> {
    pub store: &'s S,
    pub epoch_transaction_identifiers: &'s EpochTransactionIdentifiers,
    pub parent_state_version: StateVersion,
    pub ledger_transaction_hash: &'s LedgerTransactionHash,
}

pub struct ExecutionFeeData {
    pub fee_summary: TransactionFeeSummary,
    pub engine_costing_parameters: CostingParameters,
    pub transaction_costing_parameters: TransactionCostingParameters,
}

impl ProcessedTransactionReceipt {
    pub fn process<S: ReadableStore, D: DatabaseKeyMapper>(
        hash_update_context: HashUpdateContext<S>,
        receipt: TransactionReceipt,
        parent_protocol_state: &ProtocolState,
    ) -> Self {
        match receipt.result {
            TransactionResult::Commit(commit) => {
                ProcessedTransactionReceipt::Commit(ProcessedCommitResult::process::<_, D>(
                    hash_update_context,
                    commit,
                    ExecutionFeeData {
                        fee_summary: receipt.fee_summary,
                        engine_costing_parameters: receipt.costing_parameters,
                        transaction_costing_parameters: receipt.transaction_costing_parameters,
                    },
                    parent_protocol_state,
                ))
            }
            TransactionResult::Reject(reject) => {
                ProcessedTransactionReceipt::Reject(ProcessedRejectResult {
                    result: reject,
                    fee_summary: receipt.fee_summary,
                })
            }
            TransactionResult::Abort(abort) => ProcessedTransactionReceipt::Abort(abort),
        }
    }

    pub fn expect_commit(&self, description: &impl Display) -> &ProcessedCommitResult {
        match self {
            ProcessedTransactionReceipt::Commit(commit) => commit,
            ProcessedTransactionReceipt::Reject(reject) => {
                panic!("Transaction ({}) was rejected: {:?}", description, reject)
            }
            ProcessedTransactionReceipt::Abort(abort) => {
                panic!("Transaction ({}) was aborted: {:?}", description, abort)
            }
        }
    }

    pub fn expect_commit_or_reject(
        &self,
        description: &impl Display,
    ) -> Result<&ProcessedCommitResult, ProcessedRejectResult> {
        match self {
            ProcessedTransactionReceipt::Commit(commit) => Ok(commit),
            ProcessedTransactionReceipt::Reject(reject) => Err(reject.clone()),
            ProcessedTransactionReceipt::Abort(abort) => {
                panic!("Transaction {} was aborted: {:?}", description, abort)
            }
        }
    }

    pub fn get_committed_transaction_root(&self) -> Option<TransactionTreeHash> {
        if let ProcessedTransactionReceipt::Commit(commit) = self {
            Some(commit.hash_structures_diff.ledger_hashes.transaction_root)
        } else {
            None
        }
    }
}

impl ProcessedCommitResult {
    pub fn process<S: ReadableStore, D: DatabaseKeyMapper>(
        hash_update_context: HashUpdateContext<S>,
        commit_result: CommitResult,
        execution_fee_data: ExecutionFeeData,
        parent_protocol_state: &ProtocolState,
    ) -> Self {
        let epoch_identifiers = hash_update_context.epoch_transaction_identifiers;
        let parent_state_version = hash_update_context.parent_state_version;
        let ledger_transaction_hash = *hash_update_context.ledger_transaction_hash;
        let store = hash_update_context.store;

        let state_changes =
            Self::compute_ledger_state_changes::<S, D>(store, &commit_result.state_updates);

        let database_updates = commit_result.state_updates.create_database_updates::<D>();

        let global_balance_update = Self::compute_global_balance_update(
            store,
            &state_changes,
            &commit_result.state_update_summary.vault_balance_changes,
        );

        let state_hash_tree_diff =
            Self::compute_state_tree_update(store, parent_state_version, &database_updates);

        let epoch_accu_trees =
            EpochAwareAccuTreeFactory::new(epoch_identifiers.state_version, parent_state_version);

        let transaction_tree_diff = epoch_accu_trees.compute_tree_diff(
            epoch_identifiers.transaction_hash,
            store,
            vec![TransactionTreeHash::from(ledger_transaction_hash)],
        );

        let local_receipt = LocalTransactionReceipt::new(
            commit_result,
            state_changes,
            global_balance_update.global_balance_summary,
            execution_fee_data,
        );
        let consensus_receipt = local_receipt.on_ledger.get_consensus_receipt();

        let receipt_tree_diff = epoch_accu_trees.compute_tree_diff(
            epoch_identifiers.receipt_hash,
            store,
            vec![ReceiptTreeHash::from(consensus_receipt.get_hash())],
        );

        let ledger_hashes = LedgerHashes {
            state_root: state_hash_tree_diff.new_root,
            transaction_root: *transaction_tree_diff.slice.root(),
            receipt_root: *receipt_tree_diff.slice.root(),
        };

        let (new_protocol_state, next_protocol_version) = compute_new_protocol_state(
            parent_protocol_state,
            &local_receipt,
            parent_state_version.next().expect("State version overflow"),
        );

        Self {
            local_receipt,
            hash_structures_diff: HashStructuresDiff {
                ledger_hashes,
                state_hash_tree_diff,
                transaction_tree_diff,
                receipt_tree_diff,
            },
            database_updates,
            new_substate_node_ancestry_records: global_balance_update
                .new_substate_node_ancestry_records,
            new_protocol_state,
            next_protocol_version,
        }
    }

    pub fn expect_success(self, description: impl Display) -> Self {
        if let DetailedTransactionOutcome::Failure(error) =
            &self.local_receipt.local_execution.outcome
        {
            panic!(
                "{} (ledger hash: {}) failed: {:?}",
                description, self.hash_structures_diff.ledger_hashes.transaction_root, error
            );
        }
        self
    }

    pub fn next_epoch(&self) -> Option<NextEpoch> {
        self.local_receipt
            .local_execution
            .next_epoch
            .as_ref()
            .map(|next_epoch_result| NextEpoch::from(next_epoch_result.clone()))
    }

    // TODO(after RCnet-v3): Extract the `pub fn`s below (re-used by preview) to an isolated helper.

    pub fn compute_global_balance_update<S: SubstateNodeAncestryStore>(
        store: &S,
        state_changes: &LedgerStateChanges,
        vault_balance_changes: &IndexMap<NodeId, (ResourceAddress, BalanceChange)>,
    ) -> GlobalBalanceUpdate {
        // We need a fresh (!) view of a node ancestry store to group Vaults by their Global* roots.
        let new_substate_node_ancestry_records =
            NodeAncestryResolver::batch_resolve(store, state_changes.substate_level_changes.iter())
                .collect::<Vec<_>>();

        // Hence, we must prepare a "staged" ancestry store (with an overlay of the newest records).
        let map = new_substate_node_ancestry_records
            .iter()
            .flat_map(|(node_ids, record)| {
                node_ids.iter().map(|node_id| (*node_id, record.clone()))
            })
            .collect::<NonIterMap<_, _>>();
        let overlay = MapSubstateNodeAncestryStore::wrap(&map);
        let staged = StagedSubstateNodeAncestryStore::new(store, &overlay);

        // Call the group-by logic and return the results together with the ancestry store update.
        let global_balance_summary =
            GlobalBalanceSummary::compute_from(&staged, vault_balance_changes, state_changes);
        GlobalBalanceUpdate {
            global_balance_summary,
            new_substate_node_ancestry_records,
        }
    }

    pub fn compute_ledger_state_changes<S: SubstateDatabase, D: DatabaseKeyMapper>(
        store: &S,
        state_updates: &StateUpdates,
    ) -> LedgerStateChanges {
        let mut partition_level_changes = ByPartition::default();
        let mut substate_level_changes = BySubstate::default();
        for (node_id, node_updates) in &state_updates.by_node {
            let by_partition_updates = match node_updates {
                NodeStateUpdates::Delta { by_partition } => by_partition,
            };
            for (partition_num, partition_updates) in by_partition_updates {
                let substate_updates = match partition_updates {
                    PartitionStateUpdates::Delta { by_substate } => Cow::Borrowed(by_substate),
                    PartitionStateUpdates::Batch(batch) => match batch {
                        BatchPartitionStateUpdate::Reset {
                            new_substate_values,
                        } => {
                            partition_level_changes.add(
                                node_id,
                                partition_num,
                                PartitionChangeAction::Delete,
                            );
                            Cow::Owned(
                                new_substate_values
                                    .iter()
                                    .map(|(substate_key, value)| {
                                        (substate_key.clone(), DatabaseUpdate::Set(value.clone()))
                                    })
                                    .collect::<IndexMap<_, _>>(),
                            )
                        }
                    },
                };
                for (substate_key, update) in substate_updates.as_ref() {
                    let partition_key = D::to_db_partition_key(node_id, *partition_num);
                    let sort_key = D::to_db_sort_key(substate_key);

                    let previous_opt = store.get_substate(&partition_key, &sort_key);
                    let change_action_opt = match (update, previous_opt) {
                        (DatabaseUpdate::Set(new), Some(previous)) if previous != *new => {
                            Some(SubstateChangeAction::Update {
                                new: new.clone(),
                                previous,
                            })
                        }
                        (DatabaseUpdate::Set(_new), Some(_previous)) => None, // Same value as before (i.e. not really updated), ignore
                        (DatabaseUpdate::Set(value), None) => {
                            Some(SubstateChangeAction::Create { new: value.clone() })
                        }
                        (DatabaseUpdate::Delete, Some(previous)) => {
                            Some(SubstateChangeAction::Delete { previous })
                        }
                        (DatabaseUpdate::Delete, None) => None, // No value before (i.e. not really deleted), ignore
                    };

                    if let Some(change_action) = change_action_opt {
                        substate_level_changes.add(
                            node_id,
                            partition_num,
                            substate_key,
                            change_action,
                        );
                    }
                }
            }
        }
        LedgerStateChanges {
            partition_level_changes,
            substate_level_changes,
        }
    }

    fn compute_state_tree_update<S: ReadableStateTreeStore>(
        store: &S,
        parent_state_version: StateVersion,
        database_updates: &DatabaseUpdates,
    ) -> StateHashTreeDiff {
        let mut collector = CollectingTreeStore::new(store);
        let root_hash = put_at_next_version(
            &mut collector,
            Some(parent_state_version.number()).filter(|v| *v > 0),
            database_updates,
        );
        collector.into_diff_with(root_hash)
    }
}

/// A summary of vault balances per global root entity.
#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct GlobalBalanceSummary {
    /// A cumulative balance change of each global root entity owning one or more vaults from the
    /// input "vault balance changes".
    /// These entries are not pruned, i.e. a fungible delta of `0.0`, or a non-fungible delta of
    /// `{added: #1#, removed: #1#}` may be encountered here (and it simply means that resources
    /// were only moved around different vaults of the same global root entity).
    pub global_balance_changes: IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,

    /// A resultant balances of fungible resources held by global accounts.
    /// Note: aggregating a resultant balance of *any* global root entity is, in principle, not
    /// possible using only the "vault balance changes" input (because there may be vaults whose
    /// balance was not changed). We "aggregate" it for accounts using an assumption that an account
    /// has at most one vault for each resource.
    pub resultant_fungible_account_balances:
        IndexMap<GlobalAddress, IndexMap<ResourceAddress, Decimal>>,
}

impl GlobalBalanceSummary {
    /// Computes a [`GlobalBalanceSummary`] from the given vault balance changes.
    /// This uses the ancestry information from the given [`SubstateNodeAncestryStore`] to resolve
    /// the owning global entity, and parses the substate changes to collect the resulting balances
    /// of fungible vaults.
    /// Note: this function deliberately ignores the [`LedgerStateChanges::partition_level_changes`]
    /// (assuming that vault partitions cannot be deleted). In fact, it also panics upon
    /// encountering a delete of any vault's balance field.
    pub fn compute_from<S: SubstateNodeAncestryStore>(
        store: &S,
        vault_balance_changes: &IndexMap<NodeId, (ResourceAddress, BalanceChange)>,
        state_changes: &LedgerStateChanges,
    ) -> Self {
        let ancestries = store.batch_get_ancestry(vault_balance_changes.keys());
        let mut global_balance_changes = index_map_new();
        let mut resultant_fungible_account_balances = index_map_new();
        for (vault_entry, ancestry) in vault_balance_changes.iter().zip(ancestries) {
            let (vault_id, (resource_address, balance_change)) = vault_entry;

            let Some(ancestry) = ancestry else {
                panic!("No ancestry found for vault {:?}", vault_id);
            };
            let SubstateReference(root_node_id, root_partition, _) = ancestry.root;

            let Ok(root_address) = GlobalAddress::try_from(root_node_id) else {
                panic!(
                    "Root {:?} resolved for vault {:?} is not global",
                    root_node_id, vault_id
                );
            };

            // Aggregate (i.e. sum) balance changes for every global root entity.
            global_balance_changes
                .entry(root_address)
                .or_insert_with(index_map_new::<ResourceAddress, BalanceChange>)
                .entry(*resource_address)
                .and_modify(|existing| *existing += balance_change.clone())
                .or_insert_with(|| balance_change.clone());

            // Collect (i.e. not sum) resultant balances for fungible resources of global accounts.
            if vault_id.is_internal_fungible_vault()
                && root_address.is_account()
                && root_partition
                    == AccountPartitionOffset::ResourceVaultKeyValue.as_main_partition()
            {
                if ancestry.root != ancestry.parent {
                    // By the time we've got here, we know we are a fungible vault, under a global account, under the ResourceVaultKeyValue partition.
                    // The vault should also be directly owned by this substate (ie we have 1 layer, and parent == root)
                    panic!("Global account vault in resource vault partition has a parent substate which isn't equal to its root substate")
                }
                let substate_change = state_changes
                    .substate_level_changes
                    .get(
                        vault_id,
                        &FungibleVaultPartitionOffset::Field.as_main_partition(),
                        &FungibleVaultField::Balance.into(),
                    )
                    .expect(
                        "broken invariant: vault's balance changed without its substate change",
                    );
                let resultant_balance_substate = match substate_change {
                    SubstateChangeAction::Create { new } => new,
                    SubstateChangeAction::Update { new, .. } => new,
                    SubstateChangeAction::Delete { .. } => {
                        panic!("broken invariant: vault {:?} deleted", vault_id)
                    }
                };
                let resultant_balance =
                    scrypto_decode::<FungibleVaultBalanceFieldSubstate>(resultant_balance_substate)
                        .expect("cannot decode vault balance substate")
                        .into_payload()
                        .into_latest()
                        .amount();
                let balance_existed = resultant_fungible_account_balances
                    .entry(root_address)
                    .or_insert_with(index_map_new::<ResourceAddress, Decimal>)
                    .insert(*resource_address, resultant_balance)
                    .is_some();
                if balance_existed {
                    panic!(
                        "broken invariant: multiple vaults of resource {:?} exist for account {:?}",
                        resource_address, root_address
                    )
                }
            }
        }
        Self {
            global_balance_changes,
            resultant_fungible_account_balances,
        }
    }
}

impl From<EpochChangeEvent> for NextEpoch {
    fn from(epoch_change_event: EpochChangeEvent) -> Self {
        NextEpoch {
            validator_set: epoch_change_event
                .validator_set
                .validators_by_stake_desc
                .into_iter()
                .map(|(address, validator)| ActiveValidatorInfo {
                    address,
                    key: validator.key,
                    stake: validator.stake,
                })
                .collect(),
            epoch: epoch_change_event.epoch,
        }
    }
}
#[derive(Clone, Debug)]
pub struct HashStructuresDiff {
    pub ledger_hashes: LedgerHashes,
    pub state_hash_tree_diff: StateHashTreeDiff,
    pub transaction_tree_diff: AccuTreeDiff<StateVersion, TransactionTreeHash>,
    pub receipt_tree_diff: AccuTreeDiff<StateVersion, ReceiptTreeHash>,
}

#[derive(Clone, Debug)]
pub struct StateHashTreeDiff {
    pub new_root: StateHash,
    pub new_nodes: Vec<(NodeKey, TreeNode)>,
    pub stale_tree_parts: Vec<StaleTreePart>,
}

impl StateHashTreeDiff {
    pub fn new() -> Self {
        Self {
            new_root: StateHash::from(Hash([0; Hash::LENGTH])),
            new_nodes: Vec::new(),
            stale_tree_parts: Vec::new(),
        }
    }
}

impl Default for StateHashTreeDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct GlobalBalanceUpdate {
    pub global_balance_summary: GlobalBalanceSummary,
    pub new_substate_node_ancestry_records: Vec<KeyedSubstateNodeAncestryRecord>,
}

#[derive(Clone, Debug)]
pub struct AccuTreeDiff<K, N> {
    pub key: K,
    pub slice: TreeSlice<N>,
}

pub struct CollectingAccuTreeStore<'s, S, K, N> {
    readable_delegate: &'s S,
    diff: Option<AccuTreeDiff<K, N>>,
}

impl<'s, S, K, N> CollectingAccuTreeStore<'s, S, K, N> {
    pub fn new(readable_delegate: &'s S) -> Self {
        Self {
            readable_delegate,
            diff: None,
        }
    }

    pub fn into_diff(self) -> AccuTreeDiff<K, N> {
        self.diff.expect("slice not collected")
    }
}

impl<'s, S: ReadableAccuTreeStore<K, N>, K, N> ReadableAccuTreeStore<K, N>
    for CollectingAccuTreeStore<'s, S, K, N>
{
    fn get_tree_slice(&self, key: &K) -> Option<TreeSlice<N>> {
        self.readable_delegate.get_tree_slice(key)
    }
}

impl<'s, S, K, N> WriteableAccuTreeStore<K, N> for CollectingAccuTreeStore<'s, S, K, N> {
    fn put_tree_slice(&mut self, key: K, slice: TreeSlice<N>) {
        if self.diff.is_some() {
            panic!("slice already collected")
        }
        self.diff = Some(AccuTreeDiff { key, slice });
    }
}

struct CollectingTreeStore<'s, S> {
    readable_delegate: &'s S,
    diff: StateHashTreeDiff,
}

impl<'s, S: ReadableStateTreeStore> CollectingTreeStore<'s, S> {
    pub fn new(readable_delegate: &'s S) -> Self {
        Self {
            readable_delegate,
            diff: StateHashTreeDiff::new(),
        }
    }

    pub fn into_diff_with(self, new_root: Hash) -> StateHashTreeDiff {
        let mut diff = self.diff;
        diff.new_root = StateHash::from(new_root);
        diff
    }
}

impl<'s, S: ReadableTreeStore> ReadableTreeStore for CollectingTreeStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.readable_delegate.get_node(key)
    }
}

impl<'s, S> WriteableTreeStore for CollectingTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode) {
        self.diff.new_nodes.push((key, node));
    }

    fn record_stale_tree_part(&mut self, part: StaleTreePart) {
        self.diff.stale_tree_parts.push(part);
    }
}
