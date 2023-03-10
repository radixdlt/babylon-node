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

use std::collections::HashMap;
use std::hash::Hash;

/// The "read" part of an accumulator tree storage SPI.
/// Both the key type and node type are implementation-dependent.
pub trait ReadableAccuTreeStore<K, N> {
    /// Gets a vertical `TreeSlice` by the given key.
    fn get_tree_slice(&self, key: &K) -> Option<TreeSlice<N>>;
}

/// The "write" part of an accumulator tree storage SPI.
/// Both the key type and node type are implementation-dependent.
pub trait WriteableAccuTreeStore<K, N> {
    /// Puts a vertical `TreeSlice` at the given end index.
    /// This index is effectively equal to the number of leaf nodes stored in the tree by _all_ the
    /// slices _including_ the given one.
    fn put_tree_slice(&mut self, key: &K, slice: TreeSlice<N>);
}

/// A convenience read+write storage trait.
pub trait AccuTreeStore<K, N>: ReadableAccuTreeStore<K, N> + WriteableAccuTreeStore<K, N> {}
impl<T: ReadableAccuTreeStore<K, N> + WriteableAccuTreeStore<K, N>, K, N> AccuTreeStore<K, N>
    for T
{
}

/// A vertical slice of an accumulator tree.
/// This is an "incremental" persistence part of a tree representing a batch of appended leaf nodes
/// and the resulting merkle updates, propagating up to a single root.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TreeSlice<N> {
    /// The tree-levels of this slice, arranged from leafs to root.
    pub levels: Vec<TreeSliceLevel<N>>,
}

impl<N> TreeSlice<N> {
    /// Creates a vertical slice directly from levels.
    pub fn new(levels: Vec<TreeSliceLevel<N>>) -> Self {
        Self { levels }
    }

    /// Returns the root of the tree.
    pub fn root(&self) -> &N {
        self.levels
            .last()
            .expect("empty slice")
            .nodes
            .first()
            .expect("empty slice level")
    }
}

/// A single tree-level of a `TreeSlice`.
/// Effectively this means that it is a horizontal slice of a corresponding level of an entire
/// accumulator tree.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TreeSliceLevel<N> {
    /// The cached left sibling of the first node of that level slice, i.e. present if and only if
    /// the `nodes` actually start with a right child.
    /// This value was not computed (i.e. not updated) during the append to the accumulator tree,
    /// but it needed to be loaded from the storage (i.e. coming from the previous vertical slice).
    /// It is stored in this slice again (i.e. cached), because it will be needed to produce merkle
    /// proofs for the nodes contained in this slice.
    pub left_sibling_cache: Option<N>,

    /// The actual nodes computed during the append to the accumulator tree.
    /// Depending on the level, these might be exactly all the appended leafs, or their composite
    /// merkle nodes, all the way up. The highest level of each `TreeSlice` contains a single
    /// element in the `nodes`, representing the merkle root.
    pub nodes: Vec<N>,
}

impl<N> TreeSliceLevel<N> {
    /// Creates a single horizontal level slice (of some vertical tree slice).
    pub fn new(left_sibling_cache: Option<N>, nodes: Vec<N>) -> Self {
        Self {
            left_sibling_cache,
            nodes,
        }
    }
}

/// An in-memory implementation of an `AccuTreeStore`.
pub struct MemoryAccuTreeStore<K, N> {
    /// Directly stored vertical slices, keyed by their end index (exclusive).
    pub slices: HashMap<K, TreeSlice<N>>,
}

impl<K, N> MemoryAccuTreeStore<K, N> {
    /// Creates an empty in-memory storage.
    pub fn new() -> Self {
        Self {
            slices: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash, N: Clone> ReadableAccuTreeStore<K, N> for MemoryAccuTreeStore<K, N> {
    fn get_tree_slice(&self, key: &K) -> Option<TreeSlice<N>> {
        self.slices.get(key).cloned()
    }
}

impl<K: Eq + Hash + Clone, N> WriteableAccuTreeStore<K, N> for MemoryAccuTreeStore<K, N> {
    fn put_tree_slice(&mut self, key: &K, slice: TreeSlice<N>) {
        self.slices.insert(key.clone(), slice);
    }
}
