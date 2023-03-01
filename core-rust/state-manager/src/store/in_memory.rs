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

use crate::store::traits::*;
use crate::transaction::LedgerTransaction;
use crate::types::UserPayloadHash;
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, HasLedgerPayloadHash, HasUserPayloadHash,
    IntentHash, LedgerPayloadHash, LedgerTransactionReceipt,
};

use radix_engine::ledger::OutputValue;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine_interface::api::types::{KeyValueStoreId, SubstateId};
use radix_engine_stores::hash_tree::tree_store::{NodeKey, Payload, ReadableTreeStore, SerializedInMemoryTreeStore, TreeNode, WriteableTreeStore};
use radix_engine_stores::memory_db::SerializedInMemorySubstateStore;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct InMemoryStore {
    transactions: BTreeMap<u64, LedgerTransaction>,
    transaction_receipts: BTreeMap<u64, LedgerTransactionReceipt>,
    transaction_identifiers: BTreeMap<u64, CommittedTransactionIdentifiers>,
    transaction_intent_lookup: HashMap<IntentHash, u64>,
    user_payload_hash_lookup: HashMap<UserPayloadHash, u64>,
    ledger_payload_hash_lookup: HashMap<LedgerPayloadHash, u64>,
    proofs: BTreeMap<u64, Vec<u8>>,
    epoch_proofs: BTreeMap<u64, Vec<u8>>,
    vertex_store: Option<Vec<u8>>,
    substate_store: SerializedInMemorySubstateStore,
    tree_node_store: SerializedInMemoryTreeStore,
}

impl InMemoryStore {
    pub fn new() -> InMemoryStore {
        InMemoryStore {
            transactions: BTreeMap::new(),
            transaction_receipts: BTreeMap::new(),
            transaction_identifiers: BTreeMap::new(),
            transaction_intent_lookup: HashMap::new(),
            user_payload_hash_lookup: HashMap::new(),
            ledger_payload_hash_lookup: HashMap::new(),
            proofs: BTreeMap::new(),
            epoch_proofs: BTreeMap::new(),
            vertex_store: None,
            substate_store: SerializedInMemorySubstateStore::new(),
            tree_node_store: SerializedInMemoryTreeStore::new(),
        }
    }

    fn insert_transaction(
        &mut self,
        transaction: LedgerTransaction,
        receipt: LedgerTransactionReceipt,
        identifiers: CommittedTransactionIdentifiers,
    ) {
        if let LedgerTransaction::User(notarized_transaction) = &transaction {
            let intent_hash = notarized_transaction.intent_hash();
            let key_already_exists = self.transaction_intent_lookup.get(&intent_hash);
            if let Some(existing_payload_hash) = key_already_exists {
                panic!(
                    "Attempted to save intent hash which already exists: {:?}",
                    existing_payload_hash
                );
            }
            self.transaction_intent_lookup
                .insert(intent_hash, identifiers.state_version);

            self.user_payload_hash_lookup.insert(
                notarized_transaction.user_payload_hash(),
                identifiers.state_version,
            );
        }

        self.ledger_payload_hash_lookup
            .insert(transaction.ledger_payload_hash(), identifiers.state_version);

        self.transactions
            .insert(identifiers.state_version, transaction);
        self.transaction_receipts
            .insert(identifiers.state_version, receipt);
        self.transaction_identifiers
            .insert(identifiers.state_version, identifiers);
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteableVertexStore for InMemoryStore {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        self.vertex_store = Some(vertex_store_bytes);
    }
}

impl RecoverableVertexStore for InMemoryStore {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        self.vertex_store.clone()
    }
}

impl TransactionIndex<&IntentHash> for InMemoryStore {
    fn get_txn_state_version_by_identifier(&self, identifier: &IntentHash) -> Option<u64> {
        self.transaction_intent_lookup.get(identifier).cloned()
    }
}

impl TransactionIndex<&UserPayloadHash> for InMemoryStore {
    fn get_txn_state_version_by_identifier(&self, identifier: &UserPayloadHash) -> Option<u64> {
        self.user_payload_hash_lookup.get(identifier).cloned()
    }
}

impl TransactionIndex<&LedgerPayloadHash> for InMemoryStore {
    fn get_txn_state_version_by_identifier(&self, identifier: &LedgerPayloadHash) -> Option<u64> {
        self.ledger_payload_hash_lookup.get(identifier).cloned()
    }
}

impl ReadableSubstateStore for InMemoryStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.substate_store.get_substate(substate_id)
    }
}

impl<P: Payload> ReadableTreeStore<P> for InMemoryStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        self.tree_node_store.get_node(key)
    }
}

impl QueryableSubstateStore for InMemoryStore {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        self.substate_store.get_kv_store_entries(kv_store_id)
    }
}

impl CommitStore for InMemoryStore {
    fn commit(&mut self, commit_bundle: CommitBundle) {
        for (txn, receipt, identifiers) in commit_bundle.transactions {
            self.insert_transaction(txn, receipt, identifiers);
        }

        if let Some(epoch_boundary) = commit_bundle.epoch_boundary {
            self.epoch_proofs
                .insert(epoch_boundary, commit_bundle.proof_bytes.clone());
        }
        self.proofs
            .insert(commit_bundle.proof_state_version, commit_bundle.proof_bytes);

        for (substate_id, substate) in commit_bundle.substates {
            self.substate_store.put_substate(substate_id, substate);
        }

        if let Some(vertex_store) = commit_bundle.vertex_store {
            self.save_vertex_store(vertex_store)
        }

        for (key, node) in commit_bundle.state_hash_tree_update.new_nodes {
            self.tree_node_store.insert_node(key, node);
        }
    }
}

impl QueryableTransactionStore for InMemoryStore {
    fn get_committed_transaction_bundles(
        &self,
        start_state_version_inclusive: u64,
        limit: usize,
    ) -> Vec<CommittedTransactionBundle> {
        let mut res = Vec::new();

        while res.len() < limit {
            let next_state_version = start_state_version_inclusive + res.len() as u64;
            res.push((
                self.transactions.get(&next_state_version).unwrap().clone(),
                self.transaction_receipts
                    .get(&next_state_version)
                    .unwrap()
                    .clone(),
                self.transaction_identifiers
                    .get(&next_state_version)
                    .unwrap()
                    .clone(),
            ));
        }
        res
    }

    fn get_committed_transaction(&self, state_version: u64) -> Option<LedgerTransaction> {
        Some(self.transactions.get(&state_version)?.clone())
    }

    fn get_committed_transaction_receipt(
        &self,
        state_version: u64,
    ) -> Option<LedgerTransactionReceipt> {
        Some(self.transaction_receipts.get(&state_version)?.clone())
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: u64,
    ) -> Option<CommittedTransactionIdentifiers> {
        Some(self.transaction_identifiers.get(&state_version)?.clone())
    }
}

impl QueryableProofStore for InMemoryStore {
    fn max_state_version(&self) -> u64 {
        self.transactions
            .iter()
            .next_back()
            .map(|(k, _v)| *k)
            .unwrap_or_default()
    }

    /// In memory implementation doesn't need to respect the limits
    fn get_txns_and_proof(
        &self,
        start_state_version_inclusive: u64,
        _max_number_of_txns_if_more_than_one_proof: u32,
        _max_payload_size_in_bytes: u32,
    ) -> Option<(Vec<Vec<u8>>, Vec<u8>)> {
        self.proofs
            .range(start_state_version_inclusive..)
            .next()
            .map(|(v, proof)| {
                let mut txns = Vec::new();
                for (_, txn) in self.transactions.range(start_state_version_inclusive..=*v) {
                    txns.push(txn.create_payload().unwrap());
                }
                (txns, proof.clone())
            })
    }

    fn get_epoch_proof(&self, epoch: u64) -> Option<Vec<u8>> {
        self.epoch_proofs.get(&epoch).cloned()
    }

    fn get_last_proof(&self) -> Option<Vec<u8>> {
        self.proofs
            .iter()
            .next_back()
            .map(|(_, bytes)| bytes.clone())
    }
}
