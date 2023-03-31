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

use super::{ReadableHashStructuresStore, ReadableStateTreeStore};
use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore};
use crate::accumulator_tree::tree_builder::{AccuTree, Merklizable};
use crate::{
    AccumulatorHash, CommittedTransactionIdentifiers, DetailedTransactionOutcome,
    EpochTransactionIdentifiers, LedgerHashes, LedgerPayloadHash, LocalTransactionReceipt,
    NextEpoch, ReceiptTreeHash, StateHash, SubstateChange, TransactionTreeHash,
};
use radix_engine::transaction::{
    AbortResult, CommitResult, RejectResult, TransactionExecutionTrace, TransactionReceipt,
    TransactionResult,
};
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::crypto::Hash;

use crate::staging::epoch_handling::AccuTreeEpochHandler;
use radix_engine_stores::hash_tree::tree_store::{
    NodeKey, Payload, ReNodeModulePayload, ReadableTreeStore, TreeNode, WriteableTreeStore,
};
use radix_engine_stores::hash_tree::{put_at_next_version, SubstateHashChange};

pub enum ProcessedTransactionReceipt {
    Commit(ProcessedCommitResult),
    Reject(RejectResult),
    Abort(AbortResult),
}

pub struct ProcessedCommitResult {
    pub local_receipt: LocalTransactionReceipt,
    pub hash_structures_diff: HashStructuresDiff,
}

pub struct HashUpdateContext<'s, S> {
    pub store: &'s S,
    pub epoch_transaction_identifiers: &'s EpochTransactionIdentifiers,
    pub parent_transaction_identifiers: &'s CommittedTransactionIdentifiers,
    pub transaction_hash: &'s LedgerPayloadHash,
}

impl ProcessedTransactionReceipt {
    pub fn process<S: ReadableHashStructuresStore>(
        hash_update_context: HashUpdateContext<S>,
        transaction_receipt: TransactionReceipt,
    ) -> Self {
        match transaction_receipt.result {
            TransactionResult::Commit(commit) => {
                ProcessedTransactionReceipt::Commit(ProcessedCommitResult::process(
                    hash_update_context,
                    commit,
                    transaction_receipt.execution_trace,
                ))
            }
            TransactionResult::Reject(reject) => ProcessedTransactionReceipt::Reject(reject),
            TransactionResult::Abort(abort) => ProcessedTransactionReceipt::Abort(abort),
        }
    }

    pub fn expect_commit(&self, description: &str) -> &ProcessedCommitResult {
        match self {
            ProcessedTransactionReceipt::Commit(commit) => commit,
            ProcessedTransactionReceipt::Reject(reject) => {
                panic!("Transaction ({description}) was rejected: {reject:?}")
            }
            ProcessedTransactionReceipt::Abort(abort) => {
                panic!("Transaction ({description}) was aborted: {abort:?}")
            }
        }
    }

    pub fn expect_commit_or_reject(
        &self,
        description: &str,
    ) -> Result<&ProcessedCommitResult, &RejectResult> {
        match self {
            ProcessedTransactionReceipt::Commit(commit) => Ok(commit),
            ProcessedTransactionReceipt::Reject(reject) => Err(reject),
            ProcessedTransactionReceipt::Abort(abort) => {
                panic!("Transaction ({description}) was aborted: {abort:?}")
            }
        }
    }
}

impl ProcessedCommitResult {
    pub fn process<S: ReadableHashStructuresStore>(
        hash_update_context: HashUpdateContext<S>,
        commit_result: CommitResult,
        execution_trace: TransactionExecutionTrace,
    ) -> Self {
        let epoch_transaction_identifiers = hash_update_context.epoch_transaction_identifiers;
        let parent_transaction_identifiers = hash_update_context.parent_transaction_identifiers;
        let transaction_hash = hash_update_context.transaction_hash;
        let transaction_accumulator_hash = parent_transaction_identifiers
            .accumulator_hash
            .accumulate(transaction_hash);
        let local_receipt = LocalTransactionReceipt::from((commit_result, execution_trace));
        let store = hash_update_context.store;
        let state_hash_tree_diff = Self::compute_state_tree_update(
            store,
            parent_transaction_identifiers.state_version,
            &local_receipt.on_ledger.substate_changes,
        );
        let transaction_tree_diff = Self::compute_accu_tree_update::<S, TransactionTreeHash>(
            store,
            epoch_transaction_identifiers.state_version,
            epoch_transaction_identifiers.transaction_hash,
            parent_transaction_identifiers.state_version,
            TransactionTreeHash::from_raw_bytes(transaction_hash.into_bytes()),
        );
        let consensus_receipt = local_receipt.on_ledger.get_consensus_receipt();
        let receipt_tree_diff = Self::compute_accu_tree_update::<S, ReceiptTreeHash>(
            store,
            epoch_transaction_identifiers.state_version,
            epoch_transaction_identifiers.receipt_hash,
            parent_transaction_identifiers.state_version,
            ReceiptTreeHash::from_raw_bytes(consensus_receipt.get_hash().into_bytes()),
        );
        let ledger_hashes = LedgerHashes {
            state_root: state_hash_tree_diff.new_root,
            transaction_root: *transaction_tree_diff.slice.root(),
            receipt_root: *receipt_tree_diff.slice.root(),
        };
        Self {
            local_receipt,
            hash_structures_diff: HashStructuresDiff {
                transaction_accumulator_hash,
                ledger_hashes,
                state_hash_tree_diff,
                transaction_tree_diff,
                receipt_tree_diff,
            },
        }
    }

    pub fn check_success(&self, description: &str) {
        let local_detailed_outcome = &self.local_receipt.local_execution.outcome;
        if let DetailedTransactionOutcome::Failure(error) = local_detailed_outcome {
            panic!("Transaction ({description}) was failed: {error:?}")
        }
    }

    pub fn next_epoch(&self) -> Option<NextEpoch> {
        self.local_receipt
            .local_execution
            .next_epoch
            .as_ref()
            .map(|next_epoch_result| NextEpoch::from(next_epoch_result.clone()))
    }

    fn compute_accu_tree_update<S: ReadableAccuTreeStore<u64, M>, M: Merklizable + Clone>(
        store: &S,
        epoch_state_version: u64,
        epoch_root: M,
        parent_state_version: u64,
        new_leaf_hash: M,
    ) -> AccuTreeDiff<u64, M> {
        let mut collector = CollectingAccuTreeStore::new(store);
        let mut epoch_scoped_store =
            EpochScopedAccuTreeStore::new(&mut collector, epoch_state_version);
        let epoch_handler = AccuTreeEpochHandler::new(epoch_state_version, parent_state_version);
        let epoch_tree_len = epoch_handler.current_accu_tree_len();
        let appended_hashes = epoch_handler.adjust_next_batch(epoch_root, vec![new_leaf_hash]);
        AccuTree::new(&mut epoch_scoped_store, epoch_tree_len).append(appended_hashes);
        collector.into_diff()
    }

    fn compute_state_tree_update<S: ReadableStateTreeStore>(
        store: &S,
        parent_state_version: u64,
        substate_changes: &[SubstateChange],
    ) -> StateHashTreeDiff {
        let hash_changes = substate_changes
            .iter()
            .map(|substate_change| {
                SubstateHashChange::new(
                    substate_change.substate_id.clone(),
                    substate_change.action.get_new_value_hash(),
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

pub struct HashStructuresDiff {
    pub transaction_accumulator_hash: AccumulatorHash,
    pub ledger_hashes: LedgerHashes,
    pub state_hash_tree_diff: StateHashTreeDiff,
    pub transaction_tree_diff: AccuTreeDiff<u64, TransactionTreeHash>,
    pub receipt_tree_diff: AccuTreeDiff<u64, ReceiptTreeHash>,
}

pub struct StateHashTreeDiff {
    pub new_root: StateHash,
    pub new_re_node_layer_nodes: Vec<(NodeKey, TreeNode<ReNodeModulePayload>)>,
    pub new_substate_layer_nodes: Vec<(NodeKey, TreeNode<SubstateOffset>)>,
    pub stale_hash_tree_node_keys: Vec<NodeKey>,
}

impl StateHashTreeDiff {
    pub fn new() -> Self {
        Self {
            new_root: StateHash::from_raw_bytes([0; StateHash::LENGTH]),
            new_re_node_layer_nodes: Vec::new(),
            new_substate_layer_nodes: Vec::new(),
            stale_hash_tree_node_keys: Vec::new(),
        }
    }
}

impl Default for StateHashTreeDiff {
    fn default() -> Self {
        Self::new()
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
