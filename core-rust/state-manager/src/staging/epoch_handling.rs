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

use crate::accumulator_tree::slice_merger::AccuTreeSliceMerger;
use crate::accumulator_tree::storage::{
    AccuTreeStore, ReadableAccuTreeStore, TreeSlice, WriteableAccuTreeStore,
};
use crate::accumulator_tree::tree_builder::AccuTree;
use crate::accumulator_tree::IsMerklizableHash;
use crate::StateVersion;

/// A factory of accu tree utilities operating under the "accu tree per epoch" approach (where the
/// first leaf of the next epoch's tree is an auto-inserted root of the previous epoch's tree).
pub struct EpochAwareAccuTreeFactory {
    epoch_state_version: StateVersion,
    epoch_version_count: usize,
}

impl EpochAwareAccuTreeFactory {
    /// Creates a factory scoped at a particular epoch, based on 2 state versions: the state version
    /// of the transaction which started that epoch, and the state version of the last committed
    /// transaction.
    pub fn new(epoch_state_version: StateVersion, current_state_version: StateVersion) -> Self {
        let epoch_version_count =
            StateVersion::calculate_progress(epoch_state_version, current_state_version)
                .and_then(usize::try_from)
                .unwrap();
        Self {
            epoch_state_version,
            epoch_version_count,
        }
    }

    /// Creates an accu tree builder which can continue the build of the scoped epoch's accu tree
    /// at the current version (i.e. declared during this factory's construction).
    /// The [`previous_epoch_root`] will be used in case we are at the very beginning of the epoch.
    pub fn create_builder<'s, S: AccuTreeStore<StateVersion, N>, N: IsMerklizableHash>(
        &'s self,
        previous_epoch_root: N,
        store: &'s mut S,
    ) -> EpochAccuTreeBuilder<S, N> {
        EpochAccuTreeBuilder::new(
            store,
            self.epoch_state_version,
            previous_epoch_root,
            self.current_accu_tree_len(),
        )
    }

    /// Creates an accu tree merger which can merge the next slice(s) of the scoped epoch's accu
    /// tree at the current version (i.e. declared during this factory's construction).
    pub fn create_merger<N>(&self) -> AccuTreeSliceMerger<N> {
        AccuTreeSliceMerger::new(self.current_accu_tree_len())
    }

    /// Returns the actual number of leaves in the epoch's accu tree.
    /// This takes into account the extra first leaf (i.e. previous epoch's tree root), and handles
    /// the "fresh epoch" (i.e. empty accu tree) case.
    fn current_accu_tree_len(&self) -> usize {
        if self.epoch_version_count == 0 {
            0
        } else {
            self.epoch_version_count + 1
        }
    }
}

/// An [`AccuTree`] wrapper which adds the proper "accu tree per epoch" handling.
pub struct EpochAccuTreeBuilder<'s, S, N> {
    previous_epoch_root: Option<N>,
    epoch_tree_len: usize,
    epoch_scoped_store: EpochScopedAccuTreeStore<'s, S>,
}

impl<'s, S: AccuTreeStore<StateVersion, N>, N: IsMerklizableHash> EpochAccuTreeBuilder<'s, S, N> {
    fn new(
        store: &'s mut S,
        epoch_state_version: StateVersion,
        previous_epoch_root: N,
        epoch_tree_len: usize,
    ) -> Self {
        Self {
            previous_epoch_root: Some(previous_epoch_root),
            epoch_tree_len,
            epoch_scoped_store: EpochScopedAccuTreeStore {
                store,
                epoch_state_version,
            },
        }
    }

    /// Appends the next leaf to the epoch's accu tree.
    /// This method will handle a special case of an epoch that was just started (which requires
    /// inserting the previous epoch's root as the first leaf of this epoch's tree).
    pub fn append(&mut self, new_leaf_hash: N) {
        self.append_batch(vec![new_leaf_hash]);
    }

    /// A batch variant of the [`append()`] method.
    pub fn append_batch(&mut self, mut new_leaf_hashes: Vec<N>) {
        if self.epoch_tree_len == 0 {
            new_leaf_hashes.insert(0, self.previous_epoch_root.take().unwrap());
        }
        let appended_len = new_leaf_hashes.len();
        AccuTree::new(&mut self.epoch_scoped_store, self.epoch_tree_len).append(new_leaf_hashes);
        self.epoch_tree_len += appended_len;
    }
}

struct EpochScopedAccuTreeStore<'s, S> {
    store: &'s mut S,
    epoch_state_version: StateVersion,
}

impl<'s, S: ReadableAccuTreeStore<StateVersion, N>, N> ReadableAccuTreeStore<usize, N>
    for EpochScopedAccuTreeStore<'s, S>
{
    fn get_tree_slice(&self, epoch_tree_size: &usize) -> Option<TreeSlice<N>> {
        let end_state_version = self
            .epoch_state_version
            .relative(*epoch_tree_size as u64 - 1);
        self.store.get_tree_slice(&end_state_version)
    }
}

impl<'s, S: WriteableAccuTreeStore<StateVersion, N>, N> WriteableAccuTreeStore<usize, N>
    for EpochScopedAccuTreeStore<'s, S>
{
    fn put_tree_slice(&mut self, epoch_tree_size: usize, slice: TreeSlice<N>) {
        let end_state_version = self
            .epoch_state_version
            .relative(epoch_tree_size as u64 - 1);
        self.store.put_tree_slice(end_state_version, slice)
    }
}
