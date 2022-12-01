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

        let mut stack = Vec::new();
        for node_key in self.first_layer.iter() {
            stack.push(*node_key);
        }

        while !stack.is_empty() {
            let node_key = stack.pop().unwrap();

            let node = self.nodes.get_mut(node_key).unwrap();
            for next_key in node.next_keys.iter() {
                stack.push(*next_key);
            }

            self.accumulator_hash_to_node.remove(&node.accumulator_hash);
            self.nodes.remove(node_key);
        }
    }

    pub fn commit(&mut self, accumulator_hash: &AccumulatorHash) {
        let new_root_key = self.get_key(accumulator_hash).unwrap();

        match new_root_key {
            StagedSubstateStoreKey::RootStoreKey => {}
            StagedSubstateStoreKey::InternalNodeStoreKey(new_root_key) => {
                let mut stack = Vec::new();
                for node_key in self.first_layer.iter() {
                    stack.push((*node_key, 1));
                }

                while !stack.is_empty() {
                    let (node_key, depth) = stack.pop().unwrap();

                    if node_key == new_root_key {
                        self.dead_weight += depth;
                        continue;
                    }

                    let node = self.nodes.get(node_key).unwrap();
                    for next_key in node.next_keys.iter() {
                        stack.push((*next_key, depth + 1));
                    }

                    self.accumulator_hash_to_node.remove(&node.accumulator_hash);
                    self.nodes.remove(node_key);
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
