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
use crate::transaction::Transaction;
use crate::types::UserPayloadHash;
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, HasUserPayloadHash, IntentHash,
    LedgerTransactionReceipt, TransactionPayloadHash,
};
use scrypto::prelude::{scrypto_decode, scrypto_encode};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct InMemoryVertexStore {
    vertex_store: Option<Vec<u8>>,
}

impl InMemoryVertexStore {
    pub fn new() -> Self {
        Self { vertex_store: None }
    }
}

impl WriteableVertexStore for InMemoryVertexStore {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        self.vertex_store = Some(vertex_store_bytes);
    }
}

impl RecoverableVertexStore for InMemoryVertexStore {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        self.vertex_store.clone()
    }
}

impl Default for InMemoryVertexStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct InMemoryStore {
    transactions: HashMap<TransactionPayloadHash, Vec<u8>>,
    transaction_intent_lookup: HashMap<IntentHash, TransactionPayloadHash>,
    user_payload_hash_lookup: HashMap<UserPayloadHash, TransactionPayloadHash>,
    proofs: BTreeMap<u64, Vec<u8>>,
    txids: BTreeMap<u64, TransactionPayloadHash>,
}

impl InMemoryStore {
    pub fn new() -> InMemoryStore {
        InMemoryStore {
            transactions: HashMap::new(),
            transaction_intent_lookup: HashMap::new(),
            user_payload_hash_lookup: HashMap::new(),
            proofs: BTreeMap::new(),
            txids: BTreeMap::new(),
        }
    }

    fn insert_transaction(
        &mut self,
        transaction: Transaction,
        receipt: LedgerTransactionReceipt,
        identifiers: CommittedTransactionIdentifiers,
    ) {
        let payload_hash = transaction.get_hash();
        if let Transaction::User(notarized_transaction) = &transaction {
            let intent_hash = notarized_transaction.intent_hash();
            let key_already_exists = self.transaction_intent_lookup.get(&intent_hash);
            if let Some(existing_payload_hash) = key_already_exists {
                panic!(
                    "Attempted to save intent hash which already exists: {:?}",
                    existing_payload_hash
                );
            }
            self.user_payload_hash_lookup
                .insert(notarized_transaction.user_payload_hash(), payload_hash);
            self.transaction_intent_lookup
                .insert(intent_hash, payload_hash);
        }
        self.transactions.insert(
            payload_hash,
            scrypto_encode(&(transaction, receipt, identifiers)),
        );
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteableTransactionStore for InMemoryStore {
    fn insert_committed_transactions(
        &mut self,
        transactions: Vec<(
            Transaction,
            LedgerTransactionReceipt,
            CommittedTransactionIdentifiers,
        )>,
    ) {
        for (txn, receipt, identifiers) in transactions {
            self.insert_transaction(txn, receipt, identifiers);
        }
    }
}

impl UserTransactionIndex<UserPayloadHash> for InMemoryStore {
    fn get_hash(&self, identifier: &UserPayloadHash) -> Option<TransactionPayloadHash> {
        self.user_payload_hash_lookup.get(identifier).cloned()
    }
}

impl UserTransactionIndex<IntentHash> for InMemoryStore {
    fn get_hash(&self, identifier: &IntentHash) -> Option<TransactionPayloadHash> {
        self.transaction_intent_lookup.get(identifier).cloned()
    }
}

impl QueryableTransactionStore for InMemoryStore {
    fn get_committed_transaction(
        &self,
        payload_hash: &TransactionPayloadHash,
    ) -> Option<(
        Transaction,
        LedgerTransactionReceipt,
        CommittedTransactionIdentifiers,
    )> {
        let saved = self.transactions.get(payload_hash)?;
        let decoded = scrypto_decode(saved)
            .unwrap_or_else(|_| panic!("Failed to decode a stored transaction {}", payload_hash));

        Some(decoded)
    }
}

impl WriteableProofStore for InMemoryStore {
    fn insert_tids_and_proof(
        &mut self,
        state_version: u64,
        ids: Vec<TransactionPayloadHash>,
        proof_bytes: Vec<u8>,
    ) {
        self.insert_tids_without_proof(state_version, ids);

        self.proofs.insert(state_version, proof_bytes);
    }

    fn insert_tids_without_proof(&mut self, state_version: u64, ids: Vec<TransactionPayloadHash>) {
        if !ids.is_empty() {
            let first_state_version = state_version - u64::try_from(ids.len() - 1).unwrap();
            for (index, id) in ids.into_iter().enumerate() {
                let txn_state_version = first_state_version + index as u64;
                self.txids.insert(txn_state_version, id);
            }
        }
    }
}

impl QueryableProofStore for InMemoryStore {
    fn max_state_version(&self) -> u64 {
        self.txids
            .iter()
            .next_back()
            .map(|(k, _v)| *k)
            .unwrap_or_default()
    }

    fn get_payload_hash(&self, state_version: u64) -> Option<TransactionPayloadHash> {
        self.txids.get(&state_version).cloned()
    }

    /// Returns the next proof from a state version (excluded)
    fn get_next_proof(&self, state_version: u64) -> Option<(Vec<TransactionPayloadHash>, Vec<u8>)> {
        let next_state_version = state_version + 1;
        self.proofs
            .range(next_state_version..)
            .next()
            .map(|(v, proof)| {
                let mut ids = Vec::new();
                for (_, id) in self.txids.range(next_state_version..=*v) {
                    ids.push(*id);
                }
                (ids, proof.clone())
            })
    }

    fn get_last_proof(&self) -> Option<Vec<u8>> {
        self.proofs
            .iter()
            .next_back()
            .map(|(_, bytes)| bytes.clone())
    }
}
