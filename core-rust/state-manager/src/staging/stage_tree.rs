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

use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine::{ledger::*, transaction::TransactionResult};

use im::hashmap::HashMap as ImmutableHashMap;
use sbor::rust::vec::Vec;
use slotmap::{new_key_type, SlotMap};

/// An immutable/persistent store (i.e a store built from a [`parent`] store
/// shares data with it). Note: while from the abstract representation point
/// of view you can delete keys, the underlying data is freed only and only
/// if there are no references to it (there are no paths from any root reference
/// to the node representing that (key, value)). For this reason, extra steps
/// are taken to free up (key, value) pairs accumulating over time.
/// This is intended as an wrapper/abstraction layer, so that changes to the
/// ReadableSubstateStore/WriteableSubstateStore traits are easier to maintain.
#[derive(Clone)]
struct ImmutableStore {
    outputs: ImmutableHashMap<SubstateId, OutputValue>,
}

impl ImmutableStore {
    fn new() -> Self {
        ImmutableStore {
            outputs: ImmutableHashMap::new(),
        }
    }

    fn from_parent(parent: &ImmutableStore) -> Self {
        ImmutableStore {
            // Note: this clone is O(1), only the root node is actually cloned
            // Check im::collections::HashMap for details
            outputs: parent.outputs.clone(),
        }
    }
}

impl WriteableSubstateStore for ImmutableStore {
    fn put_substate(&mut self, substate_id: SubstateId, output: OutputValue) {
        self.outputs.insert(substate_id, output);
    }
}

impl ReadableSubstateStore for ImmutableStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.outputs.get(substate_id).cloned()
    }
}

new_key_type! {
    pub struct StagedSubstateStoreNodeKey;
}

/// Because the root store (which eventually is saved on disk) is not an
/// StagedSubstateStoreNode/ImmutableStore we need to be able to
/// distinguish it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagedSubstateStoreKey {
    RootStoreKey,
    InternalNodeStoreKey(StagedSubstateStoreNodeKey),
}

/// [`children_keys`] are needed in order to traverse the tree.
/// [`receipt`] applied to the parent's store, results in this node store.
/// We need to keep the [`receipt`] to both recompute the stores when doing
/// the data reconstruction/"garbage collection" but also to retrieve it
/// when caching.
pub struct StagedSubstateStoreNode {
    children_keys: Vec<StagedSubstateStoreNodeKey>,
    receipt: TransactionReceipt,
    store: ImmutableStore,
}

impl StagedSubstateStoreNode {
    fn new(receipt: TransactionReceipt, mut store: ImmutableStore) -> Self {
        if let TransactionResult::Commit(commit) = &receipt.result {
            commit.state_updates.commit(&mut store);
        }
        StagedSubstateStoreNode {
            children_keys: Vec::new(),
            receipt,
            store,
        }
    }

    /// Weight is defined as the number of changes to the ImmutableStore
    /// done exclusively by this node.
    fn weight(&self) -> usize {
        match &self.receipt.result {
            // NOTE for future Substate delete support: add down_substates.len()
            // to the weight as well.
            TransactionResult::Commit(commit) => commit.state_updates.up_substates.len(),
            TransactionResult::Reject(_) => 0,
            TransactionResult::Abort(_) => 0,
        }
    }
}

/// Structure which manages the staged store tree
pub struct StagedSubstateStoreManager<S: ReadableSubstateStore> {
    pub root: S,
    nodes: SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
    children_keys: Vec<StagedSubstateStoreNodeKey>,
    dead_weight: usize,
    total_weight: usize,
}

impl<S: ReadableSubstateStore> StagedSubstateStoreManager<S> {
    pub fn new(root: S) -> Self {
        StagedSubstateStoreManager {
            root,
            nodes: SlotMap::with_capacity_and_key(1000),
            children_keys: Vec::new(),
            dead_weight: 0,
            total_weight: 0,
        }
    }

    pub fn new_child_node(
        &mut self,
        parent_key: StagedSubstateStoreKey,
        receipt: TransactionReceipt,
    ) -> StagedSubstateStoreNodeKey {
        let store = match parent_key {
            StagedSubstateStoreKey::RootStoreKey => ImmutableStore::new(),
            StagedSubstateStoreKey::InternalNodeStoreKey(parent_key) => {
                ImmutableStore::from_parent(&self.nodes.get(parent_key).unwrap().store)
            }
        };

        // Build new node by applying the receipt to the parent store
        let new_node = StagedSubstateStoreNode::new(receipt, store);

        // Update the `total_weight` of the tree
        self.total_weight += new_node.weight();
        let new_node_key = self.nodes.insert(new_node);
        match parent_key {
            StagedSubstateStoreKey::RootStoreKey => {
                self.children_keys.push(new_node_key);
            }
            StagedSubstateStoreKey::InternalNodeStoreKey(parent_key) => {
                let parent_node = self.nodes.get_mut(parent_key).unwrap();
                parent_node.children_keys.push(new_node_key);
            }
        }
        new_node_key
    }

    pub fn get_store(&self, key: StagedSubstateStoreKey) -> StagedSubstateStore<S> {
        StagedSubstateStore { manager: self, key }
    }

    pub fn get_receipt(&self, key: &StagedSubstateStoreNodeKey) -> &TransactionReceipt {
        &self.nodes.get(*key).unwrap().receipt
    }

    fn recompute_data_recursive(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        node_key: StagedSubstateStoreNodeKey,
    ) {
        let parent_store = ImmutableStore::from_parent(&nodes.get(node_key).unwrap().store);

        let children_keys = nodes.get(node_key).unwrap().children_keys.clone();
        for child_key in children_keys.iter() {
            let child_node = nodes.get_mut(*child_key).unwrap();
            child_node.store = parent_store.clone();
            if let TransactionResult::Commit(commit) = &child_node.receipt.result {
                commit.state_updates.commit(&mut child_node.store);
            }
            Self::recompute_data_recursive(nodes, *child_key);
        }
    }

    /// Rebuilds ImmutableStores by starting from the root with new, empty ones
    /// and recursively reapplies the [`receipt`]s.
    fn recompute_data(&mut self) {
        // Reset the [`dead_weight`]
        self.dead_weight = 0;

        for node_key in self.children_keys.iter() {
            let node = self.nodes.get_mut(*node_key).unwrap();
            node.store = ImmutableStore::new();
            if let TransactionResult::Commit(commit) = &node.receipt.result {
                commit.state_updates.commit(&mut node.store);
            }
            Self::recompute_data_recursive(&mut self.nodes, *node_key);
        }
    }

    fn remove_node<CB>(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        total_weight: &mut usize,
        callback: &mut CB,
        node_key: &StagedSubstateStoreNodeKey,
    ) -> StagedSubstateStoreNode
    where
        CB: FnMut(&StagedSubstateStoreNodeKey),
    {
        let removed = nodes.remove(*node_key).unwrap();
        *total_weight -= removed.weight();
        callback(node_key);
        removed
    }

    /// Iteratively  deletes all nodes that are not in new_root_key subtree and returns the
    /// sum of weights from current root to new_root_key. Updates to ImmutableStore on this
    /// path will persist even after deleting the nodes.
    fn delete<CB>(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        total_weight: &mut usize,
        new_root_key: &StagedSubstateStoreNodeKey,
        callback: &mut CB,
        node_key: &StagedSubstateStoreNodeKey,
    ) -> usize
    where
        CB: FnMut(&StagedSubstateStoreNodeKey),
    {
        let mut stack = Vec::new();
        stack.push((nodes.get(*node_key).unwrap().weight(), *node_key));

        let mut dead_weight = 0;
        while let Some((weight, node_key)) = stack.pop() {
            if node_key == *new_root_key {
                dead_weight = weight;
                continue;
            } else {
                let children_keys = nodes.get(node_key).unwrap().children_keys.clone();
                for child_key in children_keys.iter() {
                    stack.push((weight + nodes.get(*child_key).unwrap().weight(), *child_key));
                }
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
    /// ([`ImmutableStore`]s. [`StagedSubstateStoreNode`]s however, are always deleted) committed
    /// to the real store are discarded (every time `reparent` is called).
    /// This does not really matter from a correctness perspective (the staging store
    /// will act as a cache for the real store) but as an memory overhead. The memory
    /// is freed when [`recompute_data`] is called (which is called so that the overall
    /// cost is amortized).
    /// To better understand please check:
    /// Diagram here: https://whimsical.com/persistent-staged-store-amortized-reparenting-Lyc6gRgVXVzLdqWvwVT3v4
    /// And `test_complicated_reparent` unit test
    pub fn reparent<CB>(&mut self, new_root_key: StagedSubstateStoreKey, callback: &mut CB)
    where
        CB: FnMut(&StagedSubstateStoreNodeKey),
    {
        match new_root_key {
            StagedSubstateStoreKey::RootStoreKey => {}
            StagedSubstateStoreKey::InternalNodeStoreKey(new_root_key) => {
                // Delete all nodes that are not in new_root_key subtree
                for node_key in self.children_keys.iter() {
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
                self.children_keys = new_root.children_keys;

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

pub struct StagedSubstateStore<'t, S: ReadableSubstateStore> {
    manager: &'t StagedSubstateStoreManager<S>,
    key: StagedSubstateStoreKey,
}

impl<'t, S: ReadableSubstateStore> ReadableSubstateStore for StagedSubstateStore<'t, S> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        match self.key {
            StagedSubstateStoreKey::RootStoreKey => self.manager.root.get_substate(substate_id),
            StagedSubstateStoreKey::InternalNodeStoreKey(key) => {
                // NOTE for future Substate delete support: in order to properly reflect
                // deleted keys, a Sentinel/Tombstone value should be stored instead of
                // actually removing the key. When querying here, convert the Tombstone back
                // into a None (Option can be used as the Tombstone).
                match self
                    .manager
                    .nodes
                    .get(key)
                    .unwrap()
                    .store
                    .get_substate(substate_id)
                {
                    Some(output_value) => Some(output_value),
                    None => self.manager.root.get_substate(substate_id),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_engine::engine::ScryptoInterpreter;
    use radix_engine::fee::FeeSummary;
    use radix_engine::ledger::{OutputValue, ReadableSubstateStore, TypedInMemorySubstateStore};
    use radix_engine::model::{PersistedSubstate, Resource, VaultSubstate};
    use radix_engine::state_manager::StateDiff;
    use radix_engine::transaction::{
        CommitResult, EntityChanges, TransactionExecution, TransactionOutcome, TransactionReceipt,
        TransactionResult,
    };
    use radix_engine::types::rust::iter::zip;
    use radix_engine::wasm::DefaultWasmEngine;
    use radix_engine_interface::api::types::{RENodeId, SubstateId, SubstateOffset, VaultOffset};
    use radix_engine_interface::math::Decimal;
    use radix_engine_interface::model::ResourceAddress;
    use sbor::rust::collections::BTreeMap;
    use sbor::rust::collections::HashMap;
    use sbor::rust::vec::Vec;

    fn build_transaction_receipt_from_state_diff(state_diff: StateDiff) -> TransactionReceipt {
        TransactionReceipt {
            execution: TransactionExecution {
                fee_summary: FeeSummary {
                    cost_unit_price: Decimal::default(),
                    tip_percentage: 0,
                    cost_unit_limit: 10,
                    cost_unit_consumed: 1,
                    total_execution_cost_xrd: Decimal::default(),
                    total_royalty_cost_xrd: Decimal::default(),
                    bad_debt_xrd: Decimal::default(),
                    vault_locks: Vec::new(),
                    vault_payments_xrd: None,
                    execution_cost_unit_breakdown: HashMap::new(),
                    royalty_cost_unit_breakdown: HashMap::new(),
                },
                events: Vec::new(),
            },
            result: TransactionResult::Commit(CommitResult {
                application_logs: Vec::new(),
                next_epoch: None,
                outcome: TransactionOutcome::Success(Vec::new()),
                state_updates: state_diff,
                entity_changes: EntityChanges {
                    new_component_addresses: Vec::new(),
                    new_package_addresses: Vec::new(),
                    new_resource_addresses: Vec::new(),
                },
                resource_changes: Vec::new(),
            }),
        }
    }

    fn build_dummy_substate_id(id: [u8; 36]) -> SubstateId {
        SubstateId(
            RENodeId::Vault(id),
            SubstateOffset::Vault(VaultOffset::Vault),
        )
    }

    fn build_dummy_output_value(version: u32) -> OutputValue {
        OutputValue {
            substate: PersistedSubstate::Vault(VaultSubstate(Resource::Fungible {
                resource_address: ResourceAddress::Normal([2u8; 26]),
                divisibility: 56,
                amount: Decimal::one(),
            })),
            version,
        }
    }

    #[derive(Clone)]
    struct TestNodeData {
        parent_id: usize,
        updates: Vec<(usize, usize)>,
    }

    #[test]
    fn test_complicated_reparent() {
        // Arrange
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let store = TypedInMemorySubstateStore::with_bootstrap(&scrypto_interpreter);
        let mut manager = StagedSubstateStoreManager::new(store);

        let substate_ids: Vec<SubstateId> = (0u8..5u8)
            .into_iter()
            .map(|id| build_dummy_substate_id([id; 36]))
            .collect();
        let output_values: Vec<OutputValue> = (0u32..5u32)
            .into_iter()
            .map(build_dummy_output_value)
            .collect();

        let node_test_data = [
            TestNodeData {
                // child_node[1]
                parent_id: 0, // root
                updates: [
                    (0, 1), // manager.get_store(child_node[1]).get_substate(substate_ids[0]) == output_values[1]
                ]
                .to_vec(),
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
        let mut child_node = [StagedSubstateStoreKey::RootStoreKey].to_vec();
        let mut expected_node_states = [BTreeMap::new()].to_vec();
        let mut expected_weights = [0].to_vec();
        for node_data in node_test_data.iter() {
            let up_substates: BTreeMap<SubstateId, OutputValue> = node_data
                .updates
                .iter()
                .map(|(substate_id, output_id)| {
                    (
                        substate_ids[*substate_id].clone(),
                        output_values[*output_id].clone(),
                    )
                })
                .collect();
            let state_diff = StateDiff {
                up_substates: up_substates.clone(),
                down_substates: Vec::new(),
            };
            let new_child_node = manager.new_child_node(
                child_node[node_data.parent_id],
                build_transaction_receipt_from_state_diff(state_diff.clone()),
            );
            child_node.push(StagedSubstateStoreKey::InternalNodeStoreKey(new_child_node));

            let mut expected_node_state = expected_node_states[node_data.parent_id].clone();
            expected_node_state.extend(up_substates);

            expected_node_states.push(expected_node_state);
            expected_weights.push(node_data.updates.len());
            expected_total_weight += node_data.updates.len();

            // check that all stores have the expected state
            for (child_node, expected_node_state) in
                zip(child_node.iter(), expected_node_states.iter())
            {
                let store = manager.get_store(*child_node);
                for (substate_id, output_value) in expected_node_state.iter() {
                    assert_eq!(
                        store.get_substate(substate_id),
                        Some((*output_value).clone())
                    );
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
        manager.reparent(child_node[3], &mut |_| {});
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
        manager.reparent(child_node[5], &mut |_| {});
        expected_total_weight -= [4, 5]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, 0);
        assert_eq!(manager.nodes.len(), 3);
    }
}
