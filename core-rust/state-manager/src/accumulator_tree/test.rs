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

use super::storage::{TreeSlice, TreeSliceLevel};
use super::tree_builder::{AccuTree, Merklizable};
use crate::accumulator_tree::storage::{ReadableAccuTreeStore, WriteableAccuTreeStore};
use radix_engine_interface::crypto::{blake2b_256_hash, Hash};
use std::collections::HashMap;

// Simple smoke tests using the actual hashing coming from our business use-cases:

#[test]
fn degenerate_node_tree_treats_its_only_node_as_root() {
    let node = Hash([9; 32]);
    let mut store = MemoryAccuTreeStore::new();
    AccuTree::new(&mut store, 0).append(vec![node]);
    let root = store.slices.get(&1).unwrap().root();
    assert_eq!(*root, node);
}

#[test]
fn simple_tree_produces_root_using_merkle_rule() {
    let h0 = Hash([7; 32]);
    let h1 = Hash([13; 32]);
    let h2 = Hash([66; 32]);
    let h3 = Hash([8; 32]);
    let merkle_hash = blake2b_256_hash(
        [
            blake2b_256_hash([h0.0, h1.0].concat()).0,
            blake2b_256_hash([h2.0, h3.0].concat()).0,
        ]
        .concat(),
    );
    let mut store = MemoryAccuTreeStore::new();
    AccuTree::new(&mut store, 0).append(vec![h0, h1, h2, h3]);
    let root = store.slices.get(&4).unwrap().root();
    assert_eq!(*root, merkle_hash);
}

// Detailed unit tests covering corner-cases, with a "fake hashing" allowing for easy inspection.
// The "hashing" used here is actually an opposite of a hash function: each node is a direct string,
// and a "merkle hash" is simply formatted as "(${left}+${right})".

#[test]
fn builds_direct_merkle_tree_from_first_leaf_batch() {
    let mut store = MemoryAccuTreeStore::new();
    let mut tree = AccuTree::new(&mut store, 0);
    tree.append(strings(&["a", "b", "c", "d", "e"]));
    let first_slice = store.slices.get(&5).unwrap();
    assert_eq!(
        *first_slice,
        slice(&[
            level_woc(&["(((a+b)+(c+d))+((e+)+))"]),
            level_woc(&["((a+b)+(c+d))", "((e+)+)"]),
            level_woc(&["(a+b)", "(c+d)", "(e+)"]),
            level_woc(&["a", "b", "c", "d", "e"])
        ])
    );
}

#[test]
fn builds_merkle_tree_slice_from_second_leaf_batch() {
    let mut store = MemoryAccuTreeStore::new();
    let mut tree = AccuTree::new(&mut store, 0);
    tree.append(strings(&["a", "b", "c", "d", "e"]));
    tree.append(strings(&["f", "g"]));
    let second_slice = store.slices.get(&7).unwrap();
    assert_eq!(
        *second_slice,
        slice(&[
            level_woc(&["(((a+b)+(c+d))+((e+f)+(g+)))"]),
            level("((a+b)+(c+d))", &["((e+f)+(g+))"]),
            level_woc(&["(e+f)", "(g+)"]),
            level("e", &["f", "g"]),
        ])
    );
}

#[test]
fn builds_merkle_tree_slice_from_third_leaf_batch() {
    let mut store = MemoryAccuTreeStore::new();
    let mut tree = AccuTree::new(&mut store, 0);
    tree.append(strings(&["a", "b", "c", "d", "e"]));
    tree.append(strings(&["f", "g"]));
    tree.append(strings(&["h", "i", "j"]));
    let third_slice = store.slices.get(&10).unwrap();
    assert_eq!(
        *third_slice,
        slice(&[
            level_woc(&["((((a+b)+(c+d))+((e+f)+(g+h)))+(((i+j)+)+))"]),
            level_woc(&["(((a+b)+(c+d))+((e+f)+(g+h)))", "(((i+j)+)+)"]),
            level("((a+b)+(c+d))", &["((e+f)+(g+h))", "((i+j)+)"]),
            level("(e+f)", &["(g+h)", "(i+j)"]),
            level("g", &["h", "i", "j"]),
        ])
    );
}

#[test]
fn grows_height_of_merkle_tree_on_next_power_of_two() {
    let mut store = MemoryAccuTreeStore::new();
    let mut tree = AccuTree::new(&mut store, 0);
    tree.append(strings(&["a", "b", "c", "d", "e", "f", "g", "h"]));
    tree.append(strings(&["i"]));
    let second_slice = store.slices.get(&9).unwrap();
    assert_eq!(
        *second_slice,
        slice(&[
            level_woc(&["((((a+b)+(c+d))+((e+f)+(g+h)))+(((i+)+)+))"]),
            level("(((a+b)+(c+d))+((e+f)+(g+h)))", &["(((i+)+)+)"]),
            level_woc(&["((i+)+)"]),
            level_woc(&["(i+)"]),
            level_woc(&["i"]),
        ])
    );
}

#[test]
fn incrementally_built_tree_has_same_hash_as_directly_built() {
    const LEAF_COUNT: usize = 1000;
    let nodes = (0..LEAF_COUNT)
        .map(|index| index.to_string())
        .collect::<Vec<String>>();

    let mut store_1 = MemoryAccuTreeStore::new();
    let mut directly_built_tree = AccuTree::new(&mut store_1, 0);
    directly_built_tree.append(nodes.clone());
    let root_1 = store_1.slices.get(&LEAF_COUNT).unwrap().root();

    let mut store_2 = MemoryAccuTreeStore::new();
    let mut incrementally_built = AccuTree::new(&mut store_2, 0);
    for node in nodes {
        incrementally_built.append(vec![node]);
    }
    let root_2 = store_2.slices.get(&LEAF_COUNT).unwrap().root();

    assert_eq!(root_1, root_2)
}

#[test]
fn different_slicing_results_in_same_root() {
    let mut store_1 = MemoryAccuTreeStore::new();
    let mut tree_1 = AccuTree::new(&mut store_1, 0);
    tree_1.append(strings(&["a", "b", "c", "d"]));
    tree_1.append(strings(&["e", "f", "g"]));
    tree_1.append(strings(&["h", "i"]));
    let root_1 = store_1.slices.get(&9).unwrap().root();

    let mut store_2 = MemoryAccuTreeStore::new();
    let mut tree_2 = AccuTree::new(&mut store_2, 0);
    tree_2.append(strings(&["a"]));
    tree_2.append(strings(&["b", "c", "d", "e", "f"]));
    tree_2.append(strings(&["g", "h"]));
    tree_2.append(strings(&["i"]));
    let root_2 = store_2.slices.get(&9).unwrap().root();

    assert_eq!(root_1, root_2)
}

#[test]
fn tree_slice_takes_logarithmic_space() {
    const PREVIOUS_LEAF_COUNT: usize = 1000;
    const ADDED_SLICE_LEAF_COUNT: usize = 10;
    const NEW_LEAF_COUNT: usize = PREVIOUS_LEAF_COUNT + ADDED_SLICE_LEAF_COUNT;
    const LOG_NEW_LEAF_COUNT: u32 = usize::BITS - (NEW_LEAF_COUNT - 1).leading_zeros();

    let mut store = MemoryAccuTreeStore::new();
    let mut tree = AccuTree::new(&mut store, 0);
    tree.append(strings(&["x"; PREVIOUS_LEAF_COUNT]));
    tree.append(strings(&["y"; ADDED_SLICE_LEAF_COUNT]));
    let slice = store.slices.get(&NEW_LEAF_COUNT).unwrap();
    assert_eq!(slice.levels.len() as u32, LOG_NEW_LEAF_COUNT + 1);
    for level in slice.levels.iter() {
        assert!(level.nodes.len() <= ADDED_SLICE_LEAF_COUNT);
    }
}

#[test]
fn noop_when_adding_0_leaves() {
    let mut store = MemoryAccuTreeStore::new();
    AccuTree::new(&mut store, 0).append(strings(&["a", "b", "c", "d", "e", "f", "g"]));
    let root = store.slices.get(&7).unwrap().root().clone();
    AccuTree::new(&mut store, 7).append(strings(&[]));
    let noop_root = store.slices.get(&7).unwrap().root();
    assert_eq!(*noop_root, root);
    assert_eq!(store.slices.len(), 1);
}

fn slice(levels: &[TreeSliceLevel<String>]) -> TreeSlice<String> {
    TreeSlice {
        levels: levels.iter().rev().cloned().collect(),
    }
}

fn level_woc(nodes: &[&str]) -> TreeSliceLevel<String> {
    TreeSliceLevel::new(None, strings(nodes))
}

fn level(left_sibling_cache: &str, nodes: &[&str]) -> TreeSliceLevel<String> {
    TreeSliceLevel::new(Some(left_sibling_cache.to_string()), strings(nodes))
}

fn strings(str_refs: &[&str]) -> Vec<String> {
    str_refs.iter().map(|str_ref| str_ref.to_string()).collect()
}

impl Merklizable for String {
    fn zero() -> Self {
        "".to_string()
    }

    fn merge(left: &Self, right: &Self) -> Self {
        format!("({left}+{right})")
    }
}

impl Merklizable for Hash {
    fn zero() -> Self {
        Hash([0; Hash::LENGTH])
    }

    fn merge(left: &Self, right: &Self) -> Self {
        blake2b_256_hash([left.0, right.0].concat())
    }
}

struct MemoryAccuTreeStore<K, N> {
    pub slices: HashMap<K, TreeSlice<N>>,
}

impl<K, N> MemoryAccuTreeStore<K, N> {
    pub fn new() -> Self {
        Self {
            slices: HashMap::new(),
        }
    }
}

impl<K: Eq + core::hash::Hash, N: Clone> ReadableAccuTreeStore<K, N> for MemoryAccuTreeStore<K, N> {
    fn get_tree_slice(&self, key: &K) -> Option<TreeSlice<N>> {
        self.slices.get(key).cloned()
    }
}

impl<K: Eq + core::hash::Hash + Clone, N> WriteableAccuTreeStore<K, N>
    for MemoryAccuTreeStore<K, N>
{
    fn put_tree_slice(&mut self, key: &K, slice: TreeSlice<N>) {
        self.slices.insert(key.clone(), slice);
    }
}
