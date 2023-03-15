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

use super::{ReadableLayeredTreeStore, ReadableTxAndRxTreeStore};
use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore};
use crate::accumulator_tree::tree_builder::{AccuTree, Merklizable};
use crate::{
    AccumulatorHash, CommittedTransactionIdentifiers, EpochTransactionIdentifiers, LedgerHashes,
    LedgerPayloadHash, LedgerTransactionReceipt, ReceiptTreeHash, StateHash, TransactionTreeHash,
};
use lazy_static::lazy_static;
use radix_engine::state_manager::StateDiff;
use radix_engine::transaction::{TransactionReceipt, TransactionResult};
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto::scrypto_encode;
use radix_engine_stores::hash_tree::tree_store::{
    NodeKey, Payload, ReNodeModulePayload, ReadableTreeStore, TreeNode, WriteableTreeStore,
};
use radix_engine_stores::hash_tree::{put_at_next_version, SubstateHashChange};

lazy_static! {
    static ref EMPTY_STATE_DIFF: StateDiff = StateDiff::new();
}

pub struct ProcessedResult {
    /// Raw transaction receipt.
    pub transaction_receipt: TransactionReceipt,
    /// The processing results, applicable only for committed transactions (i.e. not rejected).
    pub processed_commit: Option<ProcessedTransactionCommit>,
}

impl ProcessedResult {
    pub fn from_processed<S: ReadableLayeredTreeStore + ReadableTxAndRxTreeStore>(
        store: &S,
        epoch_transaction_identifiers: &EpochTransactionIdentifiers,
        parent_transaction_identifiers: &CommittedTransactionIdentifiers,
        transaction_accumulator_hash: AccumulatorHash,
        transaction_hash: &LedgerPayloadHash,
        transaction_receipt: TransactionReceipt,
    ) -> ProcessedResult {
        let ledger_receipt = LedgerTransactionReceipt::try_from(transaction_receipt.clone()).ok();
        let processed_commit = ledger_receipt.map(|ledger_receipt| {
            let state_diff = Self::extract_state_diff(&transaction_receipt);
            let state_tree_diff = Self::compute_state_tree_update(
                store,
                parent_transaction_identifiers.state_version,
                state_diff,
            );
            let transaction_tree_diff = Self::compute_accu_tree_update::<S, TransactionTreeHash>(
                store,
                epoch_transaction_identifiers.state_version,
                &epoch_transaction_identifiers.transaction_hash,
                parent_transaction_identifiers.state_version,
                TransactionTreeHash::from_raw_bytes((*transaction_hash).into_bytes()),
            );
            let receipt_tree_diff = Self::compute_accu_tree_update::<S, ReceiptTreeHash>(
                store,
                epoch_transaction_identifiers.state_version,
                &epoch_transaction_identifiers.receipt_hash,
                parent_transaction_identifiers.state_version,
                ReceiptTreeHash::from_raw_bytes(ledger_receipt.get_hash().into_bytes()),
            );
            let ledger_hashes = LedgerHashes {
                state_root: state_tree_diff.new_root,
                transaction_root: *transaction_tree_diff.slice.root(),
                receipt_root: *receipt_tree_diff.slice.root(),
            };
            ProcessedTransactionCommit {
                transaction_accumulator_hash,
                ledger_hashes,
                state_tree_diff,
                transaction_tree_diff,
                receipt_tree_diff,
            }
        });
        Self {
            transaction_receipt,
            processed_commit,
        }
    }

    pub fn receipt(&self) -> &TransactionReceipt {
        &self.transaction_receipt
    }

    pub fn state_diff(&self) -> &StateDiff {
        Self::extract_state_diff(&self.transaction_receipt)
    }

    pub fn commit(&self) -> &ProcessedTransactionCommit {
        self.processed_commit
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

    fn compute_accu_tree_update<S: ReadableAccuTreeStore<u64, M>, M: Merklizable + Clone>(
        store: &S,
        epoch_state_version: u64,
        epoch_root: &M,
        parent_state_version: u64,
        new_leaf_hash: M,
    ) -> AccuTreeDiff<u64, M> {
        let epoch_transaction_count = parent_state_version - epoch_state_version;
        let (epoch_tree_size, appended_hashes) = if epoch_transaction_count == 0 {
            (0, vec![epoch_root.clone(), new_leaf_hash])
        } else {
            (
                usize::try_from(epoch_transaction_count).unwrap() + 1,
                vec![new_leaf_hash],
            )
        };
        let mut collector = CollectingAccuTreeStore::new(store);
        let mut epoch_scoped_store =
            EpochScopedAccuTreeStore::new(&mut collector, epoch_state_version);
        AccuTree::new(&mut epoch_scoped_store, epoch_tree_size).append(appended_hashes);
        collector.into_diff()
    }

    fn compute_state_tree_update<S: ReadableLayeredTreeStore>(
        store: &S,
        parent_state_version: u64,
        state_diff: &StateDiff,
    ) -> HashTreeDiff {
        // TODO: currently, only the hashes of changed (or created) substates are tracked, since
        // the hash tree wants to stay consistent with the substate store (which does not support
        // deletes yet). The underlying JMT implementation already supports deletion - and to use
        // it, we simply can include `down_substates` with `None` hashes in the vector below.
        let hash_changes = state_diff
            .up_substates
            .iter()
            .map(|(id, value)| {
                SubstateHashChange::new(
                    id.clone(),
                    Some(hash(scrypto_encode(&value.substate).unwrap())),
                )
            })
            .collect::<Vec<_>>();
        let mut collector = CollectingTreeStore::new(store);
        let root_hash = put_at_next_version(
            &mut collector,
            Some(parent_state_version).filter(|v| *v > 0),
            hash_changes,
        );
        collector.into_diff_with(root_hash)
    }
}

pub struct ProcessedTransactionCommit {
    pub transaction_accumulator_hash: AccumulatorHash,
    pub ledger_hashes: LedgerHashes,
    pub state_tree_diff: HashTreeDiff,
    pub transaction_tree_diff: AccuTreeDiff<u64, TransactionTreeHash>,
    pub receipt_tree_diff: AccuTreeDiff<u64, ReceiptTreeHash>,
}

pub struct HashTreeDiff {
    pub new_root: StateHash,
    pub new_re_node_layer_nodes: Vec<(NodeKey, TreeNode<ReNodeModulePayload>)>,
    pub new_substate_layer_nodes: Vec<(NodeKey, TreeNode<SubstateOffset>)>,
    pub stale_hash_tree_node_keys: Vec<NodeKey>,
}

impl HashTreeDiff {
    pub fn new() -> Self {
        Self {
            new_root: StateHash::from_raw_bytes([0; StateHash::LENGTH]),
            new_re_node_layer_nodes: Vec::new(),
            new_substate_layer_nodes: Vec::new(),
            stale_hash_tree_node_keys: Vec::new(),
        }
    }
}

pub struct AccuTreeDiff<K, N> {
    pub key: K,
    pub slice: TreeSlice<N>,
}

struct EpochScopedAccuTreeStore<'s, S> {
    forest_store: &'s mut S,
    epoch_state_version: u64,
}

impl<'s, S> EpochScopedAccuTreeStore<'s, S> {
    pub fn new(forest_store: &'s mut S, epoch_state_version: u64) -> Self {
        Self {
            forest_store,
            epoch_state_version,
        }
    }
}

impl<'s, S: ReadableAccuTreeStore<u64, N>, N> ReadableAccuTreeStore<usize, N>
    for EpochScopedAccuTreeStore<'s, S>
{
    fn get_tree_slice(&self, epoch_tree_size: &usize) -> Option<TreeSlice<N>> {
        let end_state_version = self.epoch_state_version + *epoch_tree_size as u64 - 1;
        self.forest_store.get_tree_slice(&end_state_version)
    }
}

impl<'s, S: WriteableAccuTreeStore<u64, N>, N> WriteableAccuTreeStore<usize, N>
    for EpochScopedAccuTreeStore<'s, S>
{
    fn put_tree_slice(&mut self, epoch_tree_size: usize, slice: TreeSlice<N>) {
        let end_state_version = self.epoch_state_version + epoch_tree_size as u64 - 1;
        self.forest_store.put_tree_slice(end_state_version, slice)
    }
}

struct CollectingAccuTreeStore<'s, S, K, N> {
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
    diff: HashTreeDiff,
}

impl<'s, S: ReadableLayeredTreeStore> CollectingTreeStore<'s, S> {
    pub fn new(readable_delegate: &'s S) -> Self {
        Self {
            readable_delegate,
            diff: HashTreeDiff::new(),
        }
    }

    pub fn into_diff_with(self, new_root: Hash) -> HashTreeDiff {
        let mut diff = self.diff;
        diff.new_root = StateHash::from(new_root);
        diff
    }
}

impl<'s, S: ReadableTreeStore<P>, P: Payload> ReadableTreeStore<P> for CollectingTreeStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        self.readable_delegate.get_node(key)
    }
}

impl<'s, S> WriteableTreeStore<ReNodeModulePayload> for CollectingTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<ReNodeModulePayload>) {
        self.diff.new_re_node_layer_nodes.push((key, node));
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.diff.stale_hash_tree_node_keys.push(key);
    }
}

impl<'s, S> WriteableTreeStore<SubstateOffset> for CollectingTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<SubstateOffset>) {
        self.diff.new_substate_layer_nodes.push((key, node));
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.diff.stale_hash_tree_node_keys.push(key);
    }
}
