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

use super::stage_tree::{Accumulator, Delta, DerivedStageKey, StageKey, StageTree};
use super::ReadableStore;
use std::collections::Bound::{Included, Unbounded};

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::staging::{
    AccuTreeDiff, HashStructuresDiff, HashUpdateContext, ProcessedTransactionReceipt,
    StateHashTreeDiff,
};
use crate::{EpochTransactionIdentifiers, ReceiptTreeHash, StateVersion, TransactionTreeHash};
use im::hashmap::HashMap as ImmutableHashMap;

use im::ordmap::OrdMap as ImmutableOrdMap;

use radix_engine::track::db_key_mapper::SpreadPrefixKeyMapper;

use crate::staging::substate_overlay_iterator::SubstateOverlayIterator;
use crate::transaction::{LedgerTransactionHash, TransactionLogic};
use radix_engine_store_interface::interface::{
    DatabaseUpdate, DbPartitionKey, DbSortKey, DbSubstateValue, PartitionEntry, SubstateDatabase,
};
use radix_engine_stores::hash_tree::tree_store::{
    NodeKey, PartitionPayload, ReadableTreeStore, TreeNode,
};

use sbor::rust::collections::HashMap;
use slotmap::SecondaryMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
struct TransactionPlacement {
    parent_transaction_root: TransactionTreeHash,
    ledger_transaction_hash: LedgerTransactionHash,
}

impl TransactionPlacement {
    fn new(
        parent_transaction_root: &TransactionTreeHash,
        ledger_transaction_hash: &LedgerTransactionHash,
    ) -> Self {
        Self {
            parent_transaction_root: *parent_transaction_root,
            ledger_transaction_hash: *ledger_transaction_hash,
        }
    }
}

struct InternalTransactionIds {
    transaction_placement: TransactionPlacement,
    committed_transaction_root: Option<TransactionTreeHash>,
}

/// A cached tree of transactions, representing a potentially non-linear ledger evolution.
///
/// Important implementation note on an efficient identification of transactions within the tree
/// (i.e. without re-computation of transaction merkle tree updates on cache-hits):
///
/// A transaction may be unambiguously identified by 2 different business keys here:
///
/// - By a [`TransactionPlacement`]:
/// A "transaction placement" of transaction X is technically a tuple `{X's parent's transaction
/// root, X's payload hash}`. It can be produced for any "candidate" transaction (i.e. even for ones
/// that would be rejected).
///
/// - By a transaction root:
/// This means just a regular transaction tree root (i.e. our replacement of accumulator hash).
/// There is a gotcha though: transaction root comes from transaction's [`LedgerHashes`], and we do
/// not compute them for transactions that are rejected (technically we could compute just the
/// transaction root alone, but in our current impl we do not - for simplicity and performance).
/// Yet, in this cache, we want to reference rejected transactions as well.
///
/// For the above reasons, we need two [`HashMap`]s leading to [`DerivedStageKey`]s (we identify the
/// candidate transactions by their placement, and we identify their parents by transaction root).
pub struct ExecutionCache {
    stage_tree: StageTree<ProcessedTransactionReceipt, ImmutableStore>,
    base_transaction_root: TransactionTreeHash,
    transaction_placement_to_key: HashMap<TransactionPlacement, DerivedStageKey>,
    transaction_root_to_key: HashMap<TransactionTreeHash, DerivedStageKey>,
    key_to_internal_transaction_ids: SecondaryMap<DerivedStageKey, InternalTransactionIds>,
}

impl ExecutionCache {
    pub fn new(base_transaction_root: TransactionTreeHash) -> Self {
        ExecutionCache {
            stage_tree: StageTree::new(),
            base_transaction_root,
            transaction_placement_to_key: HashMap::new(),
            transaction_root_to_key: HashMap::new(),
            key_to_internal_transaction_ids: SecondaryMap::new(),
        }
    }

    pub fn execute_transaction<
        S: ReadableStore,
        T: for<'s> TransactionLogic<StagedStore<'s, S>>,
    >(
        &mut self,
        root_store: &S,
        epoch_transaction_identifiers: &EpochTransactionIdentifiers,
        parent_state_version: StateVersion,
        parent_transaction_root: &TransactionTreeHash,
        ledger_transaction_hash: &LedgerTransactionHash,
        executable: T,
    ) -> &ProcessedTransactionReceipt {
        let transaction_placement =
            TransactionPlacement::new(parent_transaction_root, ledger_transaction_hash);
        let transaction_key = self
            .transaction_placement_to_key
            .get(&transaction_placement);
        match transaction_key {
            Some(new_key) => self.stage_tree.get_delta(new_key),
            None => {
                let parent_key = self.get_existing_stage_key(parent_transaction_root);
                let staged_store =
                    StagedStore::new(root_store, self.stage_tree.get_accumulator(&parent_key));
                let transaction_receipt = executable.execute_on(&staged_store);

                let processed = ProcessedTransactionReceipt::process::<_, SpreadPrefixKeyMapper>(
                    HashUpdateContext {
                        store: &staged_store,
                        epoch_transaction_identifiers,
                        parent_state_version,
                        ledger_transaction_hash,
                    },
                    transaction_receipt,
                );

                let internal_transaction_ids = InternalTransactionIds {
                    transaction_placement,
                    committed_transaction_root: processed.get_committed_transaction_root(),
                };
                let transaction_key = self.stage_tree.new_child_node(parent_key, processed);
                self.add_node(transaction_key, internal_transaction_ids);

                self.stage_tree.get_delta(&transaction_key)
            }
        }
    }

    pub fn progress_base(&mut self, new_base_transaction_root: &TransactionTreeHash) {
        let new_base_key = self.get_existing_stage_key(new_base_transaction_root);
        let mut removed_keys = Vec::new();
        self.stage_tree
            .reparent(new_base_key, &mut |key| removed_keys.push(*key));
        for removed_key in removed_keys {
            self.remove_node(&removed_key);
        }
        self.base_transaction_root = *new_base_transaction_root;
    }

    fn get_existing_stage_key(&self, transaction_root: &TransactionTreeHash) -> StageKey {
        if *transaction_root == self.base_transaction_root {
            StageKey::Root
        } else {
            StageKey::Derived(*self.transaction_root_to_key.get(transaction_root).unwrap())
        }
    }

    fn add_node(
        &mut self,
        transaction_key: DerivedStageKey,
        internal_transaction_ids: InternalTransactionIds,
    ) {
        self.transaction_placement_to_key.insert(
            internal_transaction_ids.transaction_placement,
            transaction_key,
        );
        if let Some(transaction_root) = internal_transaction_ids.committed_transaction_root {
            // We purposefully store the transaction root of only committed transactions
            // (since only they can be parents of future transactions).
            self.transaction_root_to_key
                .insert(transaction_root, transaction_key);
        }
        self.key_to_internal_transaction_ids
            .insert(transaction_key, internal_transaction_ids);
    }

    fn remove_node(&mut self, transaction_key: &DerivedStageKey) {
        // Note: we don't have to remove anything from `key_to_transaction_placement`.
        // Since it's a `SecondaryMap`, it's guaranteed to be removed once the key is removed from
        // the "primary" `SlotMap` (which is `stage_tree.nodes` in our case).
        match self.key_to_internal_transaction_ids.get(*transaction_key) {
            None => {}
            Some(internal_transaction_ids) => {
                self.transaction_placement_to_key
                    .remove(&internal_transaction_ids.transaction_placement);
                if let Some(transaction_root) =
                    internal_transaction_ids.committed_transaction_root.as_ref()
                {
                    self.transaction_root_to_key.remove(transaction_root);
                }
            }
        };
    }
}

pub struct StagedStore<'s, S> {
    root: &'s S,
    overlay: &'s ImmutableStore,
}

impl<'s, S> StagedStore<'s, S> {
    pub fn new(root: &'s S, overlay: &'s ImmutableStore) -> Self {
        Self { root, overlay }
    }

    pub fn list_overlayed_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = (DbSortKey, DatabaseUpdate)> + '_> {
        let partition_key = partition_key.clone();
        Box::new(
            self.overlay
                .substate_updates
                .range((
                    Included(&(partition_key.clone(), DbSortKey(vec![]))),
                    Unbounded,
                ))
                .take_while(move |((next_partition_key, _), _)| {
                    next_partition_key.eq(&partition_key)
                })
                .map(|((_, sort_key), database_update)| {
                    (sort_key.clone(), database_update.clone())
                }),
        )
    }
}

impl<'s, S: SubstateDatabase> SubstateDatabase for StagedStore<'s, S> {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let substate_key = (partition_key.clone(), sort_key.clone());
        if let Some(overlay_update) = self.overlay.substate_updates.get(&substate_key) {
            match overlay_update {
                DatabaseUpdate::Set(value) => Some(value.clone()),
                DatabaseUpdate::Delete => None,
            }
        } else {
            self.root.get_substate(partition_key, sort_key)
        }
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let root_iter = self.root.list_entries(partition_key);
        let overlay_iter = self.list_overlayed_entries(partition_key);
        Box::new(SubstateOverlayIterator::new(
            root_iter.peekable(),
            overlay_iter.peekable(),
        ))
    }
}

impl<'s, S: ReadableTreeStore<PartitionPayload>> ReadableTreeStore<PartitionPayload>
    for StagedStore<'s, S>
{
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<PartitionPayload>> {
        self.overlay
            .re_node_layer_nodes
            .get(key)
            .cloned()
            .or_else(|| self.root.get_node(key))
    }
}

impl<'s, S: ReadableTreeStore<()>> ReadableTreeStore<()> for StagedStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<()>> {
        self.overlay
            .substate_layer_nodes
            .get(key)
            .cloned()
            .or_else(|| self.root.get_node(key))
    }
}

impl<'s, S: ReadableAccuTreeStore<StateVersion, TransactionTreeHash>>
    ReadableAccuTreeStore<StateVersion, TransactionTreeHash> for StagedStore<'s, S>
{
    fn get_tree_slice(&self, key: &StateVersion) -> Option<TreeSlice<TransactionTreeHash>> {
        self.overlay
            .transaction_tree_slices
            .get(key)
            .cloned()
            .or_else(|| self.root.get_tree_slice(key))
    }
}

impl<'s, S: ReadableAccuTreeStore<StateVersion, ReceiptTreeHash>>
    ReadableAccuTreeStore<StateVersion, ReceiptTreeHash> for StagedStore<'s, S>
{
    fn get_tree_slice(&self, key: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.overlay
            .receipt_tree_slices
            .get(key)
            .cloned()
            .or_else(|| self.root.get_tree_slice(key))
    }
}

impl Delta for ProcessedTransactionReceipt {
    fn weight(&self) -> usize {
        match self {
            ProcessedTransactionReceipt::Commit(commit) => {
                let ledger_receipt = &commit.local_receipt.on_ledger;
                ledger_receipt.substate_changes.len()
                    + ledger_receipt.application_events.len()
                    + commit.hash_structures_diff.weight()
            }
            ProcessedTransactionReceipt::Reject(_) | ProcessedTransactionReceipt::Abort(_) => 0,
        }
    }
}

impl HashStructuresDiff {
    pub fn weight(&self) -> usize {
        self.state_hash_tree_diff.weight()
            + self.transaction_tree_diff.weight()
            + self.receipt_tree_diff.weight()
    }
}

impl StateHashTreeDiff {
    pub fn weight(&self) -> usize {
        self.new_re_node_layer_nodes.len() + self.new_substate_layer_nodes.len()
    }
}

impl<K, N> AccuTreeDiff<K, N> {
    pub fn weight(&self) -> usize {
        let leaf_count = self
            .slice
            .levels
            .first()
            .map(|leaf_level| leaf_level.nodes.len())
            .unwrap_or(0);
        leaf_count * self.slice.levels.len()
    }
}

#[derive(Clone)]
pub struct ImmutableStore {
    substate_updates: ImmutableOrdMap<(DbPartitionKey, DbSortKey), DatabaseUpdate>,
    re_node_layer_nodes: ImmutableHashMap<NodeKey, TreeNode<PartitionPayload>>,
    substate_layer_nodes: ImmutableHashMap<NodeKey, TreeNode<()>>,
    transaction_tree_slices: ImmutableHashMap<StateVersion, TreeSlice<TransactionTreeHash>>,
    receipt_tree_slices: ImmutableHashMap<StateVersion, TreeSlice<ReceiptTreeHash>>,
}

impl Accumulator<ProcessedTransactionReceipt> for ImmutableStore {
    fn create_empty() -> Self {
        Self {
            substate_updates: ImmutableOrdMap::new(),
            re_node_layer_nodes: ImmutableHashMap::new(),
            substate_layer_nodes: ImmutableHashMap::new(),
            transaction_tree_slices: ImmutableHashMap::new(),
            receipt_tree_slices: ImmutableHashMap::new(),
        }
    }

    fn accumulate(&mut self, processed: &ProcessedTransactionReceipt) {
        if let ProcessedTransactionReceipt::Commit(commit) = processed {
            for (db_partition_key, partition_updates) in &commit.database_updates {
                for (db_sort_key, database_update) in partition_updates {
                    let db_substate_key = (db_partition_key.clone(), db_sort_key.clone());
                    self.substate_updates
                        .insert(db_substate_key, database_update.clone());
                }
            }
            let hash_structures_diff = &commit.hash_structures_diff;
            let state_tree_diff = &hash_structures_diff.state_hash_tree_diff;
            self.re_node_layer_nodes
                .extend(state_tree_diff.new_re_node_layer_nodes.iter().cloned());
            self.substate_layer_nodes
                .extend(state_tree_diff.new_substate_layer_nodes.iter().cloned());
            let transaction_tree_diff = &hash_structures_diff.transaction_tree_diff;
            self.transaction_tree_slices.insert(
                transaction_tree_diff.key,
                transaction_tree_diff.slice.clone(),
            );
            let receipt_tree_diff = &hash_structures_diff.receipt_tree_diff;
            self.receipt_tree_slices
                .insert(receipt_tree_diff.key, receipt_tree_diff.slice.clone());
        }
    }

    fn constant_clone(&self) -> Self {
        self.clone()
    }
}
