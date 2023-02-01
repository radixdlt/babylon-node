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

use radix_engine::types::*;

use sbor::rust::vec::Vec;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    /// A key to a derived stage.
    /// "Derived" here means "obtained by applying deltas to the root stage".
    pub struct DerivedStageKey;
}

/// A key to a (root or derived) stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageKey {
    Root,
    Derived(DerivedStageKey),
}

/// A change to the accumulated state.
pub trait Delta {
    fn weight(&self) -> usize;
}

/// An accumulated state coming from multiple [`Delta`]s.
pub trait Accumulator<D: Delta>: Default + Clone {
    fn accumulate(&mut self, delta: &D);
}

/// The [`children_keys`] are needed in order to traverse the tree.
/// The [`delta`] applied to the parent's [`accumulator`] results in this node's [`accumulator`].
/// We need to keep the [`delta`] to both recompute the stores when doing
/// the data reconstruction/"garbage collection" but also to retrieve it
/// when committing.
pub struct DerivedStageNode<D: Delta, A: Accumulator<D>> {
    child_keys: Vec<DerivedStageKey>,
    delta: D,
    accumulator: A,
}

impl<D: Delta, A: Accumulator<D>> DerivedStageNode<D, A> {
    fn new(delta: D, accumulator: A) -> Self {
        DerivedStageNode {
            child_keys: Vec::new(),
            delta,
            accumulator,
        }
    }
}

/// Stage tree (a manager of the staging nodes).
pub struct StageTree<D: Delta, A: Accumulator<D>> {
    empty_accumulator: A,
    nodes: SlotMap<DerivedStageKey, DerivedStageNode<D, A>>,
    child_keys: Vec<DerivedStageKey>,
    dead_weight: usize,
    total_weight: usize,
}

impl<D: Delta, A: Accumulator<D>> StageTree<D, A> {
    pub fn new() -> Self {
        StageTree {
            empty_accumulator: A::default(),
            nodes: SlotMap::with_capacity_and_key(1000),
            child_keys: Vec::new(),
            dead_weight: 0,
            total_weight: 0,
        }
    }

    pub fn new_child_node(&mut self, parent_key: StageKey, delta: D) -> DerivedStageKey {
        self.total_weight += delta.weight();
        let mut new_accumulator = match parent_key {
            StageKey::Root => A::default(),
            StageKey::Derived(parent_key) => {
                self.nodes.get_mut(parent_key).unwrap().accumulator.clone()
            }
        };
        new_accumulator.accumulate(&delta);
        let new_node = DerivedStageNode::new(delta, new_accumulator);
        let new_node_key = self.nodes.insert(new_node);
        let child_keys = match parent_key {
            StageKey::Root => &mut self.child_keys,
            StageKey::Derived(parent_key) => {
                &mut self.nodes.get_mut(parent_key).unwrap().child_keys
            }
        };
        child_keys.push(new_node_key);
        new_node_key
    }

    pub fn get_accumulator(&self, key: &StageKey) -> &A {
        match key {
            StageKey::Root => &self.empty_accumulator,
            StageKey::Derived(key) => &self.nodes.get(*key).unwrap().accumulator,
        }
    }

    pub fn get_delta(&self, key: &DerivedStageKey) -> &D {
        &self.nodes.get(*key).unwrap().delta
    }

    fn recompute_data(&mut self) {
        self.dead_weight = 0;

        // NOTE: check [`self.delete`] note for more details why this is implemented iteratively instead of recursively
        let mut stack: Vec<(A, Vec<DerivedStageKey>)> = Vec::new();

        stack.push((self.empty_accumulator.clone(), self.child_keys.clone()));
        while let Some((accumulator, child_keys)) = stack.pop() {
            for child_key in child_keys.iter() {
                let child_node = self.nodes.get_mut(*child_key).unwrap();
                child_node.accumulator = accumulator.clone();
                child_node.accumulator.accumulate(&child_node.delta);
                stack.push((
                    child_node.accumulator.clone(),
                    child_node.child_keys.clone(),
                ));
            }
        }
    }

    fn remove_node<CB: FnMut(&DerivedStageKey)>(
        nodes: &mut SlotMap<DerivedStageKey, DerivedStageNode<D, A>>,
        total_weight: &mut usize,
        callback: &mut CB,
        node_key: &DerivedStageKey,
    ) -> DerivedStageNode<D, A> {
        let removed = nodes.remove(*node_key).unwrap();
        *total_weight -= removed.delta.weight();
        callback(node_key);
        removed
    }

    /// Iteratively deletes all nodes that are not in new_root_key subtree and returns the
    /// sum of weights from current root to new_root_key. Updates to [`Accumulator`]s on this
    /// path will persist even after deleting the nodes.
    fn delete<CB: FnMut(&DerivedStageKey)>(
        nodes: &mut SlotMap<DerivedStageKey, DerivedStageNode<D, A>>,
        total_weight: &mut usize,
        new_root_key: &DerivedStageKey,
        callback: &mut CB,
        node_key: &DerivedStageKey,
    ) -> usize {
        // WARNING: This method was originally written recursively, however this caused a SEGFAULT due to stack overflow.
        // The tree has a depth equal to the transaction depth of staging, which is normally quite small during consensus, but
        // is much larger during ledger sync. We were getting a SEGFAULT after depths of roughly 800 transactions, presumably
        // because a large amount of data was placed on the stack in each stack frame somehow by rustc.
        let mut stack = Vec::new();
        stack.push((nodes.get(*node_key).unwrap().delta.weight(), *node_key));

        let mut dead_weight = 0;
        while let Some((weight, node_key)) = stack.pop() {
            if node_key == *new_root_key {
                dead_weight = weight;
                continue;
            }
            let child_keys = nodes.get(node_key).unwrap().child_keys.clone();
            for child_key in child_keys.iter() {
                stack.push((
                    weight + nodes.get(*child_key).unwrap().delta.weight(),
                    *child_key,
                ));
            }
            Self::remove_node(nodes, total_weight, callback, &node_key);
        }

        dead_weight
    }

    /// Each node created via [`new_child_node`] represents one store state. At some point (e.g
    /// in `commit` step) after creating multiple versions (e.g in `prepare` step), we want to
    /// move the chain of state changes from the staging store into the real store.
    /// While the changes to the real store are out of scope for this structure and done
    /// separately, we still need to inform the staging store about what current version the
    /// root store is pointing to in order for it to be able to drop no longer relevant branches.
    /// Note that because retroactive deletion for a history of persistent/immutable data
    /// structure is not possible, it is not guaranteed that the chain of state changes
    /// ([`Accumulators`]s. [`DerivedStageNode`]s however, are always deleted) committed
    /// to the real store are discarded (every time `reparent` is called).
    /// This does not really matter from a correctness perspective (the staging store
    /// will act as a cache for the real store) but as an memory overhead. The memory
    /// is freed when [`recompute_data`] is called (which is called so that the overall
    /// cost is amortized).
    /// To better understand please check:
    /// Diagram here: https://whimsical.com/persistent-staged-store-amortized-reparenting-Lyc6gRgVXVzLdqWvwVT3v4
    /// And `test_complicated_reparent` unit test
    pub fn reparent<CB: FnMut(&DerivedStageKey)>(
        &mut self,
        new_root_key: StageKey,
        callback: &mut CB,
    ) {
        match new_root_key {
            StageKey::Root => {}
            StageKey::Derived(new_root_key) => {
                // Delete all nodes that are not in new_root_key subtree
                for node_key in self.child_keys.iter() {
                    self.dead_weight += Self::delete(
                        &mut self.nodes,
                        &mut self.total_weight,
                        &new_root_key,
                        callback,
                        node_key,
                    );
                }

                let new_root = Self::remove_node(
                    &mut self.nodes,
                    &mut self.total_weight,
                    callback,
                    &new_root_key,
                );
                self.child_keys = new_root.child_keys;

                // If the number of state changes that overlap with the self.root (dead_weight) store is greater
                // than the number of state changes applied on top of it (total_weight), we recalculate the
                // ImmutableStores in order to free up memory.
                if self.dead_weight > self.total_weight {
                    self.recompute_data();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_engine::types::rust::iter::zip;
    use sbor::rust::collections::HashMap;
    use sbor::rust::vec::Vec;

    struct TestDelta(Vec<(u8, u32)>);

    impl Delta for TestDelta {
        fn weight(&self) -> usize {
            self.0.len()
        }
    }

    #[derive(Clone, Default)]
    struct TestAccumulator(HashMap<u8, u32>);

    impl Accumulator<TestDelta> for TestAccumulator {
        fn accumulate(&mut self, delta: &TestDelta) {
            for (id, value) in delta.0.iter() {
                self.0.insert(*id, *value);
            }
        }
    }

    #[derive(Clone)]
    struct TestNodeData {
        parent_id: usize,
        updates: Vec<(u8, u32)>,
    }

    #[test]
    fn test_complicated_reparent() {
        // Arrange
        let mut manager = StageTree::<TestDelta, TestAccumulator>::new();

        let node_test_data = [
            TestNodeData {
                // child_node[1]
                parent_id: 0, // root
                updates: [(0, 1)].to_vec(),
            },
            TestNodeData {
                // child_node[2]
                parent_id: 1,
                updates: [(0, 2), (2, 0)].to_vec(),
            },
            TestNodeData {
                // child_node[3]
                parent_id: 2,
                updates: [(3, 1), (4, 3), (0, 3)].to_vec(),
            },
            TestNodeData {
                // child_node[4]
                parent_id: 3,
                updates: [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)].to_vec(),
            },
            TestNodeData {
                // child_node[5]
                parent_id: 4,
                updates: [(2, 1), (0, 3)].to_vec(),
            },
            TestNodeData {
                // child_node[6]
                parent_id: 5,
                updates: [(2, 2), (3, 4)].to_vec(),
            },
            TestNodeData {
                // child_node[7]
                parent_id: 0, // root
                updates: [(2, 2)].to_vec(),
            },
            TestNodeData {
                // child_node[8]
                parent_id: 7,
                updates: [(2, 1)].to_vec(),
            },
            TestNodeData {
                // child_node[9]
                parent_id: 6,
                updates: [(2, 3), (4, 4)].to_vec(),
            },
            TestNodeData {
                // child_node[10]
                parent_id: 9,
                updates: [(2, 0)].to_vec(),
            },
        ]
        .to_vec();

        let mut expected_total_weight = 0;
        let mut child_keys = [StageKey::Root].to_vec();
        let mut expected_accumulators = [TestAccumulator(HashMap::new())].to_vec();
        let mut expected_weights = [0].to_vec();
        for node_data in node_test_data.iter() {
            let new_child_key = manager.new_child_node(
                child_keys[node_data.parent_id],
                TestDelta(node_data.updates.clone()),
            );
            child_keys.push(StageKey::Derived(new_child_key));

            let mut expected_accumulator = expected_accumulators[node_data.parent_id].clone();
            expected_accumulator.0.extend(
                node_data
                    .updates
                    .iter()
                    .map(|(key, value)| (*key, *value))
                    .collect::<HashMap<u8, u32>>(),
            );

            expected_accumulators.push(expected_accumulator);
            expected_weights.push(node_data.updates.len());
            expected_total_weight += node_data.updates.len();

            // check that all stores have the expected state
            for (child_key, expected_accumulator) in
                zip(child_keys.iter(), expected_accumulators.iter())
            {
                for (id, value) in expected_accumulator.0.iter() {
                    assert_eq!(manager.get_accumulator(child_key).0.get(id), Some(value));
                }
            }

            assert_eq!(manager.total_weight, expected_total_weight);
            assert_eq!(manager.dead_weight, 0);
        }

        // State tree layout:
        // root -> 1 -> 2 -> 3 -> 4
        //      │            └──> 5 -> 6 -> 9 -> 10
        //      └> 7 -> 8
        // After reparenting to 3: 7 and 8 are discarded completely. 1, 2 and 3 discarded but leave dead weight behind
        manager.reparent(child_keys[3], &mut |_| {});
        let expected_dead_weight = [1, 2, 3]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        expected_total_weight -= [1, 2, 3, 7, 8]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, expected_dead_weight);
        assert_eq!(manager.nodes.len(), 5);

        // After reparenting to 5: node 4 gets discarded completely. Node 5 is discarded and added to the dead weight.
        // This should trigger the recomputation/garbage collection.
        manager.reparent(child_keys[5], &mut |_| {});
        expected_total_weight -= [4, 5]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, 0);
        assert_eq!(manager.nodes.len(), 3);
    }
}
