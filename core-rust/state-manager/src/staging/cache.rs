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

use super::stage_tree::{DerivedStageKey, StageKey};
use crate::staging::stage_tree::{Accumulator, Delta, StageTree};
use crate::AccumulatorHash;
use im::hashmap::HashMap as ImmutableHashMap;
use radix_engine::ledger::{OutputValue, ReadableSubstateStore, WriteableSubstateStore};
use radix_engine::transaction::{TransactionReceipt, TransactionResult};
use radix_engine_interface::api::types::SubstateId;
use sbor::rust::collections::HashMap;
use slotmap::SecondaryMap;

pub struct ExecutionCache {
    stage_tree: StageTree<TransactionReceipt, ImmutableStore>,
    root_accumulator_hash: AccumulatorHash,
    accumulator_hash_to_key: HashMap<AccumulatorHash, DerivedStageKey>,
    key_to_accumulator_hash: SecondaryMap<DerivedStageKey, AccumulatorHash>,
}

impl ExecutionCache {
    pub fn new(root_accumulator_hash: AccumulatorHash) -> Self {
        ExecutionCache {
            stage_tree: StageTree::new(),
            root_accumulator_hash,
            accumulator_hash_to_key: HashMap::new(),
            key_to_accumulator_hash: SecondaryMap::new(),
        }
    }

    pub fn execute<S, T>(
        &mut self,
        root_store: &S,
        parent_hash: &AccumulatorHash,
        new_hash: &AccumulatorHash,
        transaction: T,
    ) -> &TransactionReceipt
    where
        S: ReadableSubstateStore,
        T: FnOnce(&StagedSubstateStore<S>) -> TransactionReceipt,
    {
        match self.accumulator_hash_to_key.get(new_hash) {
            Some(new_key) => self.stage_tree.get_delta(new_key),
            None => {
                let parent_key = self.get_existing_substore_key(parent_hash);
                let staged_store = StagedSubstateStore::new(
                    root_store,
                    self.stage_tree.get_accumulator(&parent_key),
                );
                let receipt = transaction(&staged_store);
                let new_key = self.stage_tree.new_child_node(parent_key, receipt);
                self.key_to_accumulator_hash.insert(new_key, *new_hash);
                self.accumulator_hash_to_key.insert(*new_hash, new_key);
                self.stage_tree.get_delta(&new_key)
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn progress_root(&mut self, new_root_hash: &AccumulatorHash) {
        let new_root_key = self.get_existing_substore_key(new_root_hash);
        let mut removed_keys = Vec::new();
        self.stage_tree
            .reparent(new_root_key, &mut |key| removed_keys.push(*key));
        for removed_key in removed_keys {
            self.remove_node(&removed_key);
        }
        self.root_accumulator_hash = *new_root_hash;
    }

    fn get_existing_substore_key(&self, accumulator_hash: &AccumulatorHash) -> StageKey {
        if *accumulator_hash == self.root_accumulator_hash {
            StageKey::Root
        } else {
            StageKey::Derived(*self.accumulator_hash_to_key.get(accumulator_hash).unwrap())
        }
    }

    fn remove_node(&mut self, key: &DerivedStageKey) {
        // Note: we don't have to remove anything from key_to_accumulator_hash.
        // Since it's a SecondaryMap, it's guaranteed to be removed once the key
        // is removed from the "primary" SlotMap.
        match self.key_to_accumulator_hash.get(*key) {
            None => {}
            Some(accumulator_hash) => {
                self.accumulator_hash_to_key.remove(accumulator_hash);
            }
        };
    }
}

pub struct StagedSubstateStore<'s, S: ReadableSubstateStore> {
    root: &'s S,
    overlay: &'s ImmutableStore,
}

impl<'s, S: ReadableSubstateStore> StagedSubstateStore<'s, S> {
    pub fn new(root: &'s S, overlay: &'s ImmutableStore) -> Self {
        Self { root, overlay }
    }
}

impl<'s, S: ReadableSubstateStore> ReadableSubstateStore for StagedSubstateStore<'s, S> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.overlay
            .outputs
            .get(substate_id)
            .cloned()
            .or_else(|| self.root.get_substate(substate_id))
    }
}

impl Delta for TransactionReceipt {
    fn weight(&self) -> usize {
        match &self.result {
            TransactionResult::Commit(commit) => commit.state_updates.up_substates.len(),
            TransactionResult::Reject(_) => 0,
            TransactionResult::Abort(_) => 0,
        }
    }
}

#[derive(Clone)]
pub struct ImmutableStore {
    outputs: ImmutableHashMap<SubstateId, OutputValue>,
}

impl Accumulator<TransactionReceipt> for ImmutableStore {
    fn create_empty() -> Self {
        Self {
            outputs: ImmutableHashMap::new(),
        }
    }

    fn accumulate(&mut self, delta: &TransactionReceipt) {
        if let TransactionResult::Commit(commit) = &delta.result {
            commit.state_updates.commit(self);
        }
    }

    fn constant_clone(&self) -> Self {
        self.clone()
    }
}

impl WriteableSubstateStore for ImmutableStore {
    fn put_substate(&mut self, substate_id: SubstateId, output: OutputValue) {
        self.outputs.insert(substate_id, output);
    }
}
