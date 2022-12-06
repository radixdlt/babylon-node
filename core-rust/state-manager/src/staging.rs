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

use im::hashmap::HashMap as PersistentHashMap;
use radix_engine::engine::ScryptoInterpreter;
use radix_engine::wasm::WasmEngine;
use slotmap::{new_key_type, SlotMap};

use radix_engine::ledger::*;
use radix_engine::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt, TransactionResult,
};
use radix_engine::types::*;

use scrypto::engine::types::SubstateId;
use transaction::model::Executable;

use crate::query::TransactionIdentifierLoader;
use crate::store::traits::{QueryableProofStore, TransactionIndex};
use crate::{AccumulatorHash, LedgerPayloadHash};

new_key_type! {
    struct StagedSubstateStoreNodeKey;
}

#[derive(Clone, Copy)]
enum StagedSubstateStoreKey {
    RootStoreKey,
    InternalNodeStoreKey(StagedSubstateStoreNodeKey),
}

pub struct StagedSubstateStoreNode {
    parent_key: StagedSubstateStoreKey,
    next_keys: Vec<StagedSubstateStoreNodeKey>,
    pub accumulator_hash: AccumulatorHash,
    pub receipt: TransactionReceipt,
    data: PersistentHashMap<SubstateId, OutputValue>,
}

impl StagedSubstateStoreNode {
    fn new(
        parent_key: StagedSubstateStoreKey,
        accumulator_hash: AccumulatorHash,
        receipt: TransactionReceipt,
        data: PersistentHashMap<SubstateId, OutputValue>,
    ) -> Self {
        StagedSubstateStoreNode {
            parent_key,
            next_keys: Vec::new(),
            accumulator_hash,
            receipt,
            data,
        }
    }

    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.data.get(substate_id).cloned()
    }
}

pub struct StagedSubstateStoreManager<S: ReadableSubstateStore> {
    pub root: S,
    nodes: SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
    accumulator_hash_to_node: HashMap<AccumulatorHash, StagedSubstateStoreNodeKey>,
    first_layer: Vec<StagedSubstateStoreNodeKey>,
    dead_weight: u32,
}

fn recompute_data_recursive(
    nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
    node_key: StagedSubstateStoreNodeKey,
) {
    let parent_data = nodes.get(node_key).unwrap().data.clone();

    let next_keys = nodes.get(node_key).unwrap().next_keys.clone();
    for next_key in next_keys.iter() {
        let next_node = nodes.get_mut(*next_key).unwrap();
        next_node.data = parent_data.clone();
        if let TransactionResult::Commit(commit) = &next_node.receipt.result {
            for (substate_id, output_value) in &commit.state_updates.up_substates {
                next_node
                    .data
                    .insert(substate_id.clone(), output_value.clone());
            }
        }
        recompute_data_recursive(nodes, *next_key);
    }
}

fn delete_recursive(
    nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
    accumulator_hash_to_node: &mut HashMap<AccumulatorHash, StagedSubstateStoreNodeKey>,
    node_key: &StagedSubstateStoreNodeKey,
    new_root_key: &StagedSubstateStoreNodeKey,
    depth: u32,
) -> u32 {
    if *node_key == *new_root_key {
        return depth;
    }

    let mut dead_weight = 0;
    let children = nodes.get(*node_key).unwrap().next_keys.clone();
    for next_key in children {
        dead_weight += delete_recursive(
            nodes,
            accumulator_hash_to_node,
            &next_key,
            new_root_key,
            depth + 1,
        );
    }

    let node = nodes.get(*node_key).unwrap();

    accumulator_hash_to_node.remove(&node.accumulator_hash);
    nodes.remove(*node_key);

    dead_weight
}

impl<S> StagedSubstateStoreManager<S>
where
    S: ReadableSubstateStore,
    S: QueryableProofStore + TransactionIndex<u64>,
{
    pub fn new(root: S) -> Self {
        StagedSubstateStoreManager {
            root,
            nodes: SlotMap::with_capacity_and_key(1000),
            accumulator_hash_to_node: HashMap::new(),
            first_layer: Vec::new(),
            dead_weight: 0,
        }
    }

    fn get_key(&self, accumulator_hash: &AccumulatorHash) -> Option<StagedSubstateStoreKey> {
        match self.accumulator_hash_to_node.get(accumulator_hash) {
            Some(node_key) => Some(StagedSubstateStoreKey::InternalNodeStoreKey(*node_key)),
            None => {
                if *accumulator_hash
                    == self
                        .root
                        .get_top_of_ledger_transaction_identifiers_unwrap()
                        .accumulator_hash
                {
                    Some(StagedSubstateStoreKey::RootStoreKey)
                } else {
                    None
                }
            }
        }
    }

    pub fn execute_with_cache<W: WasmEngine>(
        &mut self,
        accumulator_hash: &AccumulatorHash,
        ledger_payload_hash: &LedgerPayloadHash,
        scrypto_interpreter: &ScryptoInterpreter<W>,
        fee_reserve_config: &FeeReserveConfig,
        execution_config: &ExecutionConfig,
        executable: &Executable,
    ) -> &StagedSubstateStoreNode {
        let new_accumulator_hash = accumulator_hash.accumulate(ledger_payload_hash);
        if let Some(store_key) = self.get_key(&new_accumulator_hash) {
            match store_key {
                StagedSubstateStoreKey::RootStoreKey => {
                    panic!("Trying to execute previously committed transaction. This should not happen!")
                }
                StagedSubstateStoreKey::InternalNodeStoreKey(key) => {
                    return self.nodes.get(key).unwrap();
                }
            }
        }
        let parent_key = self.get_key(accumulator_hash).unwrap();

        let store = StagedSubstateStore {
            manager: self,
            key: parent_key,
        };

        let receipt = execute_transaction(
            &store,
            scrypto_interpreter,
            fee_reserve_config,
            execution_config,
            executable,
        );

        let mut new_data = match parent_key {
            StagedSubstateStoreKey::RootStoreKey => PersistentHashMap::new(),
            StagedSubstateStoreKey::InternalNodeStoreKey(key) => {
                self.nodes.get(key).unwrap().data.clone()
            }
        };

        if let TransactionResult::Commit(commit) = &receipt.result {
            for (substate_id, output_value) in &commit.state_updates.up_substates {
                new_data.insert(substate_id.clone(), output_value.clone());
            }
        }

        let new_node_key = self.nodes.insert(StagedSubstateStoreNode::new(
            parent_key,
            new_accumulator_hash,
            receipt,
            new_data,
        ));
        self.accumulator_hash_to_node
            .insert(new_accumulator_hash, new_node_key);

        match parent_key {
            StagedSubstateStoreKey::InternalNodeStoreKey(parent_node_key) => {
                let parent_node = self.nodes.get_mut(parent_node_key).unwrap();
                parent_node.next_keys.push(new_node_key);
            }
            StagedSubstateStoreKey::RootStoreKey => {
                self.first_layer.push(new_node_key);
            }
        }

        self.nodes.get(new_node_key).unwrap()
    }

    fn recompute_data(&mut self) {
        self.dead_weight = 0;

        for node_key in self.first_layer.iter() {
            let node = self.nodes.get_mut(*node_key).unwrap();
            node.data = PersistentHashMap::new();
            recompute_data_recursive(&mut self.nodes, *node_key);
        }
    }

    pub fn commit(&mut self, accumulator_hash: &AccumulatorHash) {
        let new_root_key = self.get_key(accumulator_hash).unwrap();

        match new_root_key {
            StagedSubstateStoreKey::RootStoreKey => {}
            StagedSubstateStoreKey::InternalNodeStoreKey(new_root_key) => {
                for node_key in self.first_layer.iter() {
                    self.dead_weight += delete_recursive(
                        &mut self.nodes,
                        &mut self.accumulator_hash_to_node,
                        node_key,
                        &new_root_key,
                        1,
                    );
                }

                let new_root = self.nodes.get(new_root_key).unwrap();

                self.first_layer = new_root.next_keys.clone();
                for key in self.first_layer.iter() {
                    let node = self.nodes.get_mut(*key).unwrap();
                    node.parent_key = StagedSubstateStoreKey::RootStoreKey;
                }

                let new_root = self.nodes.get(new_root_key).unwrap();

                self.accumulator_hash_to_node
                    .remove(&new_root.accumulator_hash);
                self.nodes.remove(new_root_key);

                if self.dead_weight as usize > self.nodes.len() {
                    self.recompute_data();
                }
            }
        }
    }
}

struct StagedSubstateStore<'t, S: ReadableSubstateStore> {
    manager: &'t StagedSubstateStoreManager<S>,
    key: StagedSubstateStoreKey,
}

impl<'t, S: ReadableSubstateStore> ReadableSubstateStore for StagedSubstateStore<'t, S> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        match self.key {
            StagedSubstateStoreKey::RootStoreKey => self.manager.root.get_substate(substate_id),
            StagedSubstateStoreKey::InternalNodeStoreKey(key) => {
                match self
                    .manager
                    .nodes
                    .get(key)
                    .unwrap()
                    .get_substate(substate_id)
                {
                    Some(output_value) => Some(output_value),
                    None => self.manager.root.get_substate(substate_id),
                }
            }
        }
    }
}
