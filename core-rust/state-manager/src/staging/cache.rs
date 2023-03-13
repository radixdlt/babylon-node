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
use super::RootStore;

use super::result::ProcessedResult;
use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::staging::{AccuTreeDiff, HashTreeDiff, ProcessedTransactionCommit};
use crate::{
    AccumulatorHash, CommittedTransactionIdentifiers, EpochTransactionIdentifiers,
    LedgerPayloadHash, ReceiptHash, TransactionHash,
};
use im::hashmap::HashMap as ImmutableHashMap;
use radix_engine::ledger::{OutputValue, ReadableSubstateStore};
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::api::types::{SubstateId, SubstateOffset};
use radix_engine_stores::hash_tree::tree_store::{
    NodeKey, ReNodeModulePayload, ReadableTreeStore, TreeNode,
};
use sbor::rust::collections::HashMap;
use slotmap::SecondaryMap;

pub struct ExecutionCache {
    stage_tree: StageTree<ProcessedResult, ImmutableStore>,
    root_accumulator_hash: AccumulatorHash,
    accumulator_hash_to_key: HashMap<AccumulatorHash, DerivedStageKey>,
    key_to_accumulator_hash: SecondaryMap<DerivedStageKey, AccumulatorHash>,
}

impl ExecutionCache {
    pub fn new(root_accumulator_hash: AccumulatorHash) -> Self {
        ExecutionCache {
            stage_tree: StageTree::new(),
            root_accumulator_hash,
            accumulator_hash_to_key: HashMap::new(),
            key_to_accumulator_hash: SecondaryMap::new(),
        }
    }

    pub fn execute_transaction<S: RootStore, T: FnOnce(&StagedStore<S>) -> TransactionReceipt>(
        &mut self,
        root_store: &S,
        epoch_transaction_identifiers: &EpochTransactionIdentifiers,
        parent_transaction_identifiers: &CommittedTransactionIdentifiers,
        transaction_hash: &LedgerPayloadHash,
        transaction: T,
    ) -> &ProcessedResult {
        let parent_accumulator_hash = &parent_transaction_identifiers.accumulator_hash;
        let transaction_accumulator_hash = parent_accumulator_hash.accumulate(transaction_hash);
        match self
            .accumulator_hash_to_key
            .get(&transaction_accumulator_hash)
        {
            Some(new_key) => self.stage_tree.get_delta(new_key),
            None => {
                let parent_key = self.get_existing_substore_key(parent_accumulator_hash);
                let staged_store =
                    StagedStore::new(root_store, self.stage_tree.get_accumulator(&parent_key));
                let transaction_receipt = transaction(&staged_store);
                let processed = ProcessedResult::from_processed(
                    &staged_store,
                    epoch_transaction_identifiers,
                    parent_transaction_identifiers,
                    transaction_accumulator_hash,
                    transaction_hash,
                    transaction_receipt,
                );
                let transaction_key = self.stage_tree.new_child_node(parent_key, processed);
                self.key_to_accumulator_hash
                    .insert(transaction_key, transaction_accumulator_hash);
                self.accumulator_hash_to_key
                    .insert(transaction_accumulator_hash, transaction_key);
                self.stage_tree.get_delta(&transaction_key)
            }
        }
    }

    pub fn progress_root(&mut self, new_root_hash: &AccumulatorHash) {
        let new_root_key = self.get_existing_substore_key(new_root_hash);
        let mut removed_keys = Vec::new();
        self.stage_tree
            .reparent(new_root_key, &mut |key| removed_keys.push(*key));
        for removed_key in removed_keys {
            self.remove_node(&removed_key);
        }
        self.root_accumulator_hash = *new_root_hash;
    }

    fn get_existing_substore_key(&self, accumulator_hash: &AccumulatorHash) -> StageKey {
        if *accumulator_hash == self.root_accumulator_hash {
            StageKey::Root
        } else {
            StageKey::Derived(*self.accumulator_hash_to_key.get(accumulator_hash).unwrap())
        }
    }

    fn remove_node(&mut self, key: &DerivedStageKey) {
        // Note: we don't have to remove anything from key_to_accumulator_hash.
        // Since it's a SecondaryMap, it's guaranteed to be removed once the key
        // is removed from the "primary" SlotMap.
        match self.key_to_accumulator_hash.get(*key) {
            None => {}
            Some(accumulator_hash) => {
                self.accumulator_hash_to_key.remove(accumulator_hash);
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
}

impl<'s, S: ReadableSubstateStore> ReadableSubstateStore for StagedStore<'s, S> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.overlay
            .substate_values
            .get(substate_id)
            .cloned()
            .or_else(|| self.root.get_substate(substate_id))
    }
}

impl<'s, S: ReadableTreeStore<ReNodeModulePayload>> ReadableTreeStore<ReNodeModulePayload>
    for StagedStore<'s, S>
{
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<ReNodeModulePayload>> {
        self.overlay
            .re_node_layer_nodes
            .get(key)
            .cloned()
            .or_else(|| self.root.get_node(key))
    }
}

impl<'s, S: ReadableTreeStore<SubstateOffset>> ReadableTreeStore<SubstateOffset>
    for StagedStore<'s, S>
{
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<SubstateOffset>> {
        self.overlay
            .substate_layer_nodes
            .get(key)
            .cloned()
            .or_else(|| self.root.get_node(key))
    }
}

impl<'s, S: ReadableAccuTreeStore<u64, TransactionHash>> ReadableAccuTreeStore<u64, TransactionHash>
    for StagedStore<'s, S>
{
    fn get_tree_slice(&self, key: &u64) -> Option<TreeSlice<TransactionHash>> {
        self.overlay
            .transaction_tree_slices
            .get(key)
            .cloned()
            .or_else(|| self.root.get_tree_slice(key))
    }
}

impl<'s, S: ReadableAccuTreeStore<u64, ReceiptHash>> ReadableAccuTreeStore<u64, ReceiptHash>
    for StagedStore<'s, S>
{
    fn get_tree_slice(&self, key: &u64) -> Option<TreeSlice<ReceiptHash>> {
        self.overlay
            .receipt_tree_slices
            .get(key)
            .cloned()
            .or_else(|| self.root.get_tree_slice(key))
    }
}

impl Delta for ProcessedResult {
    fn weight(&self) -> usize {
        let state_diff_weight = self.state_diff().up_substates.len();
        let processed_commit_weight = self.processed_commit.as_ref().map(|commit| commit.weight());
        state_diff_weight + processed_commit_weight.unwrap_or(0)
    }
}

impl ProcessedTransactionCommit {
    pub fn weight(&self) -> usize {
        self.state_tree_diff.weight()
            + self.transaction_tree_diff.weight()
            + self.receipt_tree_diff.weight()
    }
}

impl HashTreeDiff {
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
    substate_values: ImmutableHashMap<SubstateId, OutputValue>,
    re_node_layer_nodes: ImmutableHashMap<NodeKey, TreeNode<ReNodeModulePayload>>,
    substate_layer_nodes: ImmutableHashMap<NodeKey, TreeNode<SubstateOffset>>,
    transaction_tree_slices: ImmutableHashMap<u64, TreeSlice<TransactionHash>>,
    receipt_tree_slices: ImmutableHashMap<u64, TreeSlice<ReceiptHash>>,
}

impl Accumulator<ProcessedResult> for ImmutableStore {
    fn create_empty() -> Self {
        Self {
            substate_values: ImmutableHashMap::new(),
            re_node_layer_nodes: ImmutableHashMap::new(),
            substate_layer_nodes: ImmutableHashMap::new(),
            transaction_tree_slices: ImmutableHashMap::new(),
            receipt_tree_slices: ImmutableHashMap::new(),
        }
    }

    fn accumulate(&mut self, processed: &ProcessedResult) {
        self.substate_values.extend(
            processed
                .state_diff()
                .up_substates
                .iter()
                .map(|(id, value)| (id.clone(), value.clone())),
        );
        if let Some(processed_commit) = &processed.processed_commit {
            let state_tree_diff = &processed_commit.state_tree_diff;
            self.re_node_layer_nodes
                .extend(state_tree_diff.new_re_node_layer_nodes.iter().cloned());
            self.substate_layer_nodes
                .extend(state_tree_diff.new_substate_layer_nodes.iter().cloned());
            let transaction_tree_diff = &processed_commit.transaction_tree_diff;
            self.transaction_tree_slices.insert(
                transaction_tree_diff.key,
                transaction_tree_diff.slice.clone(),
            );
            let receipt_tree_diff = &processed_commit.receipt_tree_diff;
            self.receipt_tree_slices
                .insert(receipt_tree_diff.key, receipt_tree_diff.slice.clone());
        }
    }

    fn constant_clone(&self) -> Self {
        self.clone()
    }
}
