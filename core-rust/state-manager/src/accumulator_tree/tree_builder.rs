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

use super::storage::{AccuTreeStore, TreeSlice, TreeSliceLevel};
use std::marker::PhantomData;

/// An element that a merkle tree can be built from (i.e. both a leaf and an internal node).
pub trait Merklizable {
    /// A "zero" element, to be used as a right sibling to `merge()` with, when the actual right
    /// sibling does not exist.
    fn zero() -> Self;

    /// A merging function (i.e. giving virtually-unique results for any ordered pair).
    fn merge(left: &Self, right: &Self) -> Self;
}

/// An accumulator tree with persistence backed by an external storage.
/// The storage key (for `AccuTreeStore`) used by this implementation is simply an integer
/// representing the number of leaf nodes stored in the tree by _all_ the slices _including_ the
/// loaded/saved one. This approach allows to easily access the last slice (needed for `append()`
/// operation) by knowing only the current tree size.
/// For convenience, this structure tracks the current tree size internally between calls.
pub struct AccuTree<'s, S: AccuTreeStore<usize, N>, N> {
    store: &'s mut S,
    current_len: usize,
    phantom_data: PhantomData<N>,
}

impl<'s, S: AccuTreeStore<usize, M>, M: Merklizable> AccuTree<'s, S, M> {
    /// Creates a new accumulator tree backed by the given store, which is assumed to currently hold
    /// the given number of leaves.
    pub fn new(store: &'s mut S, current_len: usize) -> Self {
        Self {
            store,
            current_len,
            phantom_data: PhantomData,
        }
    }

    /// Appends the given batch of new leaves to the tree.
    pub fn append(&mut self, leaves: Vec<M>) {
        let new_leaf_count = leaves.len();
        if new_leaf_count == 0 {
            return;
        }

        let mut previous_slice_levels = if self.current_len == 0 {
            vec![].into_iter()
        } else {
            let previous_slice = self
                .store
                .get_tree_slice(&self.current_len)
                .expect("no slice ending at given version found");
            previous_slice.levels.into_iter()
        };

        let target_length = self.current_len + new_leaf_count;

        let mut from = self.current_len;
        let mut higher_level_nodes = leaves;

        let mut levels = Vec::new();
        let target_height = usize::BITS - (target_length - 1).leading_zeros();
        for _ in 0..target_height {
            let previous_slice_level = previous_slice_levels.next();
            let left_sibling_cache = if from % 2 == 0 {
                None
            } else {
                let previous_slice_level = previous_slice_level.unwrap();
                Some(
                    RelativeNodeAccess::for_successor_offset(previous_slice_level, from)
                        .into_node(from - 1),
                )
            };
            let lower_level_access = RelativeNodeAccess::for_offset(
                from,
                TreeSliceLevel::new(left_sibling_cache, higher_level_nodes),
            );

            let to = (from + lower_level_access.slice_level.nodes.len() + 1) / 2;
            from /= 2;
            higher_level_nodes = (from..to)
                .map(|level_index| level_index * 2)
                .map(|lower_level_index| {
                    M::merge(
                        lower_level_access.get(lower_level_index),
                        lower_level_access.get(lower_level_index + 1),
                    )
                })
                .collect();

            levels.push(lower_level_access.into_level());
        }
        levels.push(TreeSliceLevel::new(None, higher_level_nodes));

        self.current_len = target_length;
        self.store
            .put_tree_slice(self.current_len, TreeSlice::new(levels));
    }
}

enum RelativeIndex {
    CachedLeftSibling,
    ActualStoredNode(usize),
    ZeroRightSibling,
}

struct RelativeNodeAccess<M: Merklizable> {
    offset: usize,
    slice_level: TreeSliceLevel<M>,
    zero: M, // only present here because it is the easiest way of returning a `&M` from `get()`
}

impl<M: Merklizable> RelativeNodeAccess<M> {
    pub fn for_offset(offset: usize, slice_level: TreeSliceLevel<M>) -> Self {
        Self {
            offset,
            slice_level,
            zero: M::zero(),
        }
    }

    pub fn for_successor_offset(slice_level: TreeSliceLevel<M>, successor_offset: usize) -> Self {
        let computed_node_count = slice_level.nodes.len();
        let cached_node_count = slice_level.left_sibling_cache.iter().count();
        let cumulative_parity = (successor_offset + computed_node_count + cached_node_count) & 1;
        Self::for_offset(
            cumulative_parity + successor_offset - computed_node_count,
            slice_level,
        )
    }

    pub fn get(&self, index: usize) -> &M {
        match self.to_relative_index(index) {
            RelativeIndex::CachedLeftSibling => {
                self.slice_level.left_sibling_cache.as_ref().unwrap()
            }
            RelativeIndex::ActualStoredNode(internal_index) => {
                &self.slice_level.nodes[internal_index]
            }
            RelativeIndex::ZeroRightSibling => &self.zero,
        }
    }

    pub fn into_node(mut self, index: usize) -> M {
        match self.to_relative_index(index) {
            RelativeIndex::CachedLeftSibling => self.slice_level.left_sibling_cache.unwrap(),
            RelativeIndex::ActualStoredNode(internal_index) => {
                std::mem::replace(&mut self.slice_level.nodes[internal_index], M::zero())
            }
            RelativeIndex::ZeroRightSibling => self.zero,
        }
    }

    pub fn into_level(self) -> TreeSliceLevel<M> {
        self.slice_level
    }

    fn to_relative_index(&self, index: usize) -> RelativeIndex {
        if index < self.offset {
            if index + 1 == self.offset {
                if self.slice_level.left_sibling_cache.is_none() {
                    panic!("wrongfully requested left sibling at index {index}");
                }
                return RelativeIndex::CachedLeftSibling;
            }
            panic!("index {} requested when offset {}", index, self.offset);
        }
        let internal_index = index - self.offset;
        if internal_index < self.slice_level.nodes.len() {
            return RelativeIndex::ActualStoredNode(internal_index);
        }
        if internal_index == self.slice_level.nodes.len() {
            return RelativeIndex::ZeroRightSibling;
        }
        panic!(
            "node of index {} requested from a level holding {}[{}; {})",
            index,
            self.slice_level
                .left_sibling_cache
                .as_ref()
                .map(|_| "cache + ")
                .unwrap_or(""),
            self.offset,
            self.offset + self.slice_level.nodes.len()
        );
    }
}
