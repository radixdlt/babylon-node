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

use super::stage_tree::{DerivedStageKey, StageKey};
use crate::staging::stage_tree::{Accumulator, Delta, StageTree};
use crate::{
    AccumulatorHash, LedgerHashes, LedgerPayloadHash, LedgerTransactionReceipt, ReceiptHash,
    StateHash, TransactionHash,
};
use im::hashmap::HashMap as ImmutableHashMap;
use lazy_static::lazy_static;
use radix_engine::ledger::{OutputValue, ReadableSubstateStore};
use radix_engine::state_manager::StateDiff;
use radix_engine::transaction::{TransactionReceipt, TransactionResult};
use radix_engine_interface::api::types::SubstateId;
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_stores::hash_tree::put_at_next_version;
use radix_engine_stores::hash_tree::tree_store::{
    NodeKey, ReadableTreeStore, TreeNode, Version, WriteableTreeStore,
};
use sbor::rust::collections::HashMap;
use slotmap::SecondaryMap;

pub trait RootStore: ReadableSubstateStore + ReadableTreeStore {}
impl<T: ReadableSubstateStore + ReadableTreeStore> RootStore for T {}

pub struct ExecutionCache {
    stage_tree: StageTree<ProcessedResult, ImmutableStore>,
    root_accumulator_hash: AccumulatorHash,
    accumulator_hash_to_key: HashMap<AccumulatorHash, DerivedStageKey>,
    key_to_accumulator_hash: SecondaryMap<DerivedStageKey, AccumulatorHash>,
}

pub struct ProcessedResult {
    /// Raw transaction receipt.
    receipt: TransactionReceipt,
    /// The processing results, applicable only for committed transactions (i.e. not rejected).
    committed_results: Option<(LedgerHashes, HashTreeDiff)>,
}

#[derive(Clone)]
pub struct HashTreeDiff {
    pub new_hash_tree_nodes: Vec<(NodeKey, TreeNode)>,
    pub stale_hash_tree_node_keys: Vec<NodeKey>,
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
        state_version: Option<Version>,
        parent_hash: &AccumulatorHash,
        transaction_hash: &LedgerPayloadHash,
        new_hash: &AccumulatorHash,
        transaction: T,
    ) -> &ProcessedResult {
        match self.accumulator_hash_to_key.get(new_hash) {
            Some(new_key) => self.stage_tree.get_delta(new_key),
            None => {
                let parent_key = self.get_existing_substore_key(parent_hash);
                let staged_store =
                    StagedStore::new(root_store, self.stage_tree.get_accumulator(&parent_key));
                let receipt = transaction(&staged_store);
                let processed = ProcessedResult::from_processed(
                    transaction_hash,
                    receipt,
                    state_version,
                    &staged_store,
                );
                let new_key = self.stage_tree.new_child_node(parent_key, processed);
                self.key_to_accumulator_hash.insert(new_key, *new_hash);
                self.accumulator_hash_to_key.insert(*new_hash, new_key);
                self.stage_tree.get_delta(&new_key)
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

struct CollectingTreeStore<'s, S: ReadableTreeStore> {
    readable_delegate: &'s S,
    diff: HashTreeDiff,
}

impl<'s, S: ReadableTreeStore> CollectingTreeStore<'s, S> {
    pub fn new(readable_delegate: &'s S) -> Self {
        Self {
            readable_delegate,
            diff: HashTreeDiff::new(),
        }
    }
}

impl<'s, S: ReadableTreeStore> ReadableTreeStore for CollectingTreeStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.readable_delegate.get_node(key)
    }
}

impl<'s, S: ReadableTreeStore> WriteableTreeStore for CollectingTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode) {
        self.diff.new_hash_tree_nodes.push((key, node));
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.diff.stale_hash_tree_node_keys.push(key);
    }
}

pub struct StagedStore<'s, S: RootStore> {
    root: &'s S,
    overlay: &'s ImmutableStore,
}

impl<'s, S: RootStore> StagedStore<'s, S> {
    pub fn new(root: &'s S, overlay: &'s ImmutableStore) -> Self {
        Self { root, overlay }
    }
}

impl<'s, S: RootStore> ReadableSubstateStore for StagedStore<'s, S> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.overlay
            .substate_values
            .get(substate_id)
            .cloned()
            .or_else(|| self.root.get_substate(substate_id))
    }
}

impl<'s, S: RootStore> ReadableTreeStore for StagedStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.overlay
            .hash_tree_nodes
            .get(key)
            .cloned()
            .or_else(|| ReadableTreeStore::get_node(self.root, key))
    }
}

lazy_static! {
    static ref EMPTY_STATE_DIFF: StateDiff = StateDiff::new();
}

impl ProcessedResult {
    pub fn from_processed<S: ReadableTreeStore>(
        transaction_hash: &LedgerPayloadHash,
        transaction_receipt: TransactionReceipt,
        state_version: Option<Version>,
        store: &S,
    ) -> ProcessedResult {
        let ledger_receipt = LedgerTransactionReceipt::try_from(transaction_receipt.clone()).ok();
        let committed_results = ledger_receipt.map(|ledger_receipt| {
            let state_diff = Self::extract_state_diff(&transaction_receipt);
            let (state_tree_root_hash, state_tree_diff) =
                Self::compute_state_tree_update(store, state_version, state_diff);
            let ledger_hashes = LedgerHashes {
                state_root: StateHash::from(state_tree_root_hash),
                // TODO(wip): the values below are currently placeholders; they should be computed
                // and returned together with "update slices" of their respective merkle trees.
                transaction_root: TransactionHash::from_raw_bytes(transaction_hash.into_bytes()),
                receipt_root: ReceiptHash::from_raw_bytes(ledger_receipt.get_hash().into_bytes()),
            };
            (ledger_hashes, state_tree_diff)
        });
        Self {
            receipt: transaction_receipt,
            committed_results,
        }
    }

    pub fn receipt(&self) -> &TransactionReceipt {
        &self.receipt
    }

    pub fn state_diff(&self) -> &StateDiff {
        Self::extract_state_diff(&self.receipt)
    }

    pub fn ledger_hashes(&self) -> &LedgerHashes {
        &self.committed_results().0
    }

    pub fn hash_tree_diff(&self) -> &HashTreeDiff {
        &self.committed_results().1
    }

    fn committed_results(&self) -> &(LedgerHashes, HashTreeDiff) {
        self.committed_results
            .as_ref()
            .expect("available only for committed transactions")
    }

    fn extract_state_diff(receipt: &TransactionReceipt) -> &StateDiff {
        if let TransactionResult::Commit(commit) = &receipt.result {
            &commit.state_updates
        } else {
            &EMPTY_STATE_DIFF
        }
    }

    fn compute_state_tree_update<S: ReadableTreeStore>(
        store: &S,
        state_version: Option<Version>,
        state_diff: &StateDiff,
    ) -> (Hash, HashTreeDiff) {
        // TODO: currently, only the hashes of changed (or created) substates are tracked, since
        // the hash tree wants to stay consistent with the substate store (which does not support
        // deletes yet). The underlying JMT implementation already supports deletion - and to use
        // it, we simply can include `down_substates` with `None` hashes in the vector below.
        let hash_changes = state_diff
            .up_substates
            .iter()
            .map(|(id, value)| {
                (
                    id.clone(),
                    Some(hash(scrypto_encode(&value.substate).unwrap())),
                )
            })
            .collect::<Vec<(SubstateId, Option<Hash>)>>();
        let mut collector = CollectingTreeStore::new(store);
        let root_hash = put_at_next_version(&mut collector, state_version, &hash_changes);
        (root_hash, collector.diff)
    }
}

impl Delta for ProcessedResult {
    fn weight(&self) -> usize {
        let state_diff_weight = self.state_diff().up_substates.len();
        let hash_tree_diff_weight = self
            .committed_results
            .as_ref()
            .map(|results| results.1.new_hash_tree_nodes.len());
        state_diff_weight + hash_tree_diff_weight.unwrap_or(0)
    }
}

impl HashTreeDiff {
    pub fn new() -> Self {
        Self {
            new_hash_tree_nodes: Vec::new(),
            stale_hash_tree_node_keys: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct ImmutableStore {
    substate_values: ImmutableHashMap<SubstateId, OutputValue>,
    hash_tree_nodes: ImmutableHashMap<NodeKey, TreeNode>,
}

impl Accumulator<ProcessedResult> for ImmutableStore {
    fn create_empty() -> Self {
        Self {
            substate_values: ImmutableHashMap::new(),
            hash_tree_nodes: ImmutableHashMap::new(),
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
        if let Some((_, hash_tree_diff)) = &processed.committed_results {
            self.hash_tree_nodes
                .extend(hash_tree_diff.new_hash_tree_nodes.iter().cloned());
        }
    }

    fn constant_clone(&self) -> Self {
        self.clone()
    }
}
