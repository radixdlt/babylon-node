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

use crate::types::UserPayloadHash;
use std::collections::HashMap;

use crate::store::traits::*;
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, IntentHash, LedgerPayloadHash,
    LedgerTransactionReceipt,
};
use radix_engine::ledger::{
    OutputValue, QueryableSubstateStore, ReadableSubstateStore, WriteableSubstateStore,
};
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{
    scrypto_decode, scrypto_encode, KeyValueStoreId, KeyValueStoreOffset, RENodeId, SubstateId,
    SubstateOffset,
};
use rocksdb::{Direction, IteratorMode, SingleThreaded, TransactionDB};
use std::path::PathBuf;

#[macro_export]
macro_rules! db_key {
    ($table:expr, $slice:expr) => {{
        let mut vec = vec![$table.prefix()];
        vec.extend_from_slice($slice);
        vec
    }};
}

enum RocksDBTable {
    Transactions,
    StateVersions,
    Proofs,
    Substates,
    VertexStore,
    TransactionIntentLookup,
    UserPayloadHashLookup,
}
use crate::transaction::LedgerTransaction;
use RocksDBTable::*;

impl RocksDBTable {
    fn prefix(&self) -> u8 {
        match self {
            Transactions => 0u8,
            StateVersions => 1u8,
            Proofs => 2u8,
            Substates => 3u8,
            VertexStore => 4u8,
            TransactionIntentLookup => 5u8,
            UserPayloadHashLookup => 6u8,
        }
    }
}

pub struct RocksDBStore {
    db: TransactionDB<SingleThreaded>,
}

fn get_transaction_key(payload_hash: &LedgerPayloadHash) -> Vec<u8> {
    db_key!(Transactions, payload_hash.as_ref())
}

fn get_user_transaction_payload_key(payload_hash: &UserPayloadHash) -> Vec<u8> {
    db_key!(UserPayloadHashLookup, payload_hash.as_ref())
}

fn get_transaction_intent_key(intent_hash: &IntentHash) -> Vec<u8> {
    db_key!(TransactionIntentLookup, intent_hash.as_ref())
}

impl RocksDBStore {
    pub fn new(root: PathBuf) -> RocksDBStore {
        let db = TransactionDB::open_default(root.as_path()).unwrap();
        RocksDBStore { db }
    }
}

impl<'db> CommitStore<'db> for RocksDBStore {
    type DBTransaction = RocksDBCommitTransaction<'db>;

    fn create_db_transaction(&'db mut self) -> RocksDBCommitTransaction<'db> {
        let db_txn = self.db.transaction();
        RocksDBCommitTransaction { db_txn }
    }
}

impl WriteableSubstateStore for RocksDBStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.db
            .put(
                db_key!(Substates, &scrypto_encode(&substate_id).unwrap()),
                scrypto_encode(&substate).unwrap(),
            )
            .expect("RockDB: put_substate unexpected error");
    }
}

pub struct RocksDBCommitTransaction<'db> {
    db_txn: rocksdb::Transaction<'db, TransactionDB>,
}

impl<'db> RocksDBCommitTransaction<'db> {
    fn insert_transaction(
        &mut self,
        transaction: LedgerTransaction,
        receipt: LedgerTransactionReceipt,
        identifiers: CommittedTransactionIdentifiers,
    ) {
        // TEMPORARY until this is handled in the engine: we store both an intent lookup and the transaction itself
        if let LedgerTransaction::User(notarized_transaction) = &transaction {
            let key = get_transaction_intent_key(&notarized_transaction.intent_hash());
            let existing_intent_option = self
                .db_txn
                .get_for_update(key, true)
                .expect("RocksDB: failure to read intent hash");
            if let Some(existing_intent_hash) = existing_intent_option {
                panic!(
                    "Attempted to save intent hash which already exists: {:?}",
                    IntentHash::from_raw_bytes(existing_intent_hash.try_into().unwrap())
                );
            }
            self.db_txn
                .put(
                    get_transaction_intent_key(&notarized_transaction.intent_hash()),
                    transaction.get_hash().as_ref(),
                )
                .expect("RocksDB: failure to put intent hash");
        }
        self.db_txn
            .put(
                get_transaction_key(&transaction.get_hash()),
                scrypto_encode(&(transaction, receipt, identifiers)).unwrap(),
            )
            .expect("RocksDB: failure to put transaction");
    }
}

impl<'db> ReadableSubstateStore for RocksDBCommitTransaction<'db> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        // TODO: Use get_pinned
        self.db_txn
            .get(db_key!(Substates, &scrypto_encode(substate_id).unwrap()))
            .unwrap()
            .map(|b| scrypto_decode(&b).unwrap())
    }
}

impl<'db> WriteableTransactionStore for RocksDBCommitTransaction<'db> {
    fn insert_committed_transactions(
        &mut self,
        transactions: Vec<(
            LedgerTransaction,
            LedgerTransactionReceipt,
            CommittedTransactionIdentifiers,
        )>,
    ) {
        for (txn, receipt, identifiers) in transactions {
            self.insert_transaction(txn, receipt, identifiers);
        }
    }
}

impl<'db> WriteableProofStore for RocksDBCommitTransaction<'db> {
    fn insert_tids_and_proof(
        &mut self,
        state_version: u64,
        ids: Vec<LedgerPayloadHash>,
        proof_bytes: Vec<u8>,
    ) {
        self.insert_tids_without_proof(state_version, ids);

        let proof_version_key = db_key!(Proofs, &state_version.to_be_bytes());
        self.db_txn.put(proof_version_key, proof_bytes).unwrap();
    }

    fn insert_tids_without_proof(&mut self, state_version: u64, ids: Vec<LedgerPayloadHash>) {
        if !ids.is_empty() {
            let first_state_version = state_version - u64::try_from(ids.len() - 1).unwrap();
            for (index, payload_hash) in ids.into_iter().enumerate() {
                let txn_state_version = first_state_version + index as u64;
                let version_key = db_key!(StateVersions, &txn_state_version.to_be_bytes());
                self.db_txn.put(version_key, payload_hash.as_ref()).unwrap();
            }
        }
    }
}

impl<'db> WriteableSubstateStore for RocksDBCommitTransaction<'db> {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.db_txn
            .put(
                db_key!(Substates, &scrypto_encode(&substate_id).unwrap()),
                scrypto_encode(&substate).unwrap(),
            )
            .expect("RocksDB: put_substate unexpected error");
    }
}

impl<'db> WriteableVertexStore for RocksDBCommitTransaction<'db> {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        self.db_txn
            .put(db_key!(VertexStore, &[]), &vertex_store_bytes)
            .unwrap();
    }
}

impl<'db> CommitStoreTransaction<'db> for RocksDBCommitTransaction<'db> {
    fn commit(self) {
        self.db_txn
            .commit()
            .expect("Unable to commit rocksdb transaction");
    }
}

impl ReadableSubstateStore for RocksDBStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        // TODO: Use get_pinned
        self.db
            .get(db_key!(Substates, &scrypto_encode(substate_id).unwrap()))
            .unwrap()
            .map(|b| scrypto_decode(&b).unwrap())
    }
}

impl QueryableTransactionStore for RocksDBStore {
    fn get_committed_transaction(
        &self,
        payload_hash: &LedgerPayloadHash,
    ) -> Option<(
        LedgerTransaction,
        LedgerTransactionReceipt,
        CommittedTransactionIdentifiers,
    )> {
        let entry = self
            .db
            .get(get_transaction_key(payload_hash))
            .expect("DB error loading transaction")?;
        let decoded = scrypto_decode(&entry).expect("Transaction wasn't encoded as expected");
        Some(decoded)
    }
}

impl TransactionIndex<&UserPayloadHash> for RocksDBStore {
    fn get_payload_hash(&self, identifier: &UserPayloadHash) -> Option<LedgerPayloadHash> {
        let payload_hash_entry = self
            .db
            .get(get_user_transaction_payload_key(identifier))
            .expect("DB error loading payload hash")?;
        let hash = LedgerPayloadHash::from_raw_bytes(
            payload_hash_entry
                .try_into()
                .expect("Saved payload hash is wrong length"),
        );
        Some(hash)
    }
}

impl TransactionIndex<&IntentHash> for RocksDBStore {
    fn get_payload_hash(&self, identifier: &IntentHash) -> Option<LedgerPayloadHash> {
        let payload_hash_entry = self
            .db
            .get(get_transaction_intent_key(identifier))
            .expect("DB error loading payload hash")?;
        let hash = LedgerPayloadHash::from_raw_bytes(
            payload_hash_entry
                .try_into()
                .expect("Saved payload hash is wrong length"),
        );
        Some(hash)
    }
}

impl TransactionIndex<u64> for RocksDBStore {
    fn get_payload_hash(&self, state_version: u64) -> Option<LedgerPayloadHash> {
        let state_version_entry = self
            .db
            .get(&db_key!(StateVersions, &state_version.to_be_bytes()))
            .expect("DB error loading state version")?;
        let hash = LedgerPayloadHash::from_raw_bytes(
            state_version_entry
                .try_into()
                .expect("Saved payload hash is wrong length"),
        );
        Some(hash)
    }
}

impl QueryableProofStore for RocksDBStore {
    fn max_state_version(&self) -> u64 {
        self.db
            .iterator(IteratorMode::From(
                &[StateVersions.prefix() + 1],
                Direction::Reverse,
            ))
            .next()
            .map(|res| res.unwrap())
            .filter(|(key, _)| key[0] == StateVersions.prefix())
            .map(|(key, _)| {
                let (_, state_version_bytes) = key.split_first().unwrap();
                u64::from_be_bytes(state_version_bytes.try_into().unwrap())
            })
            .unwrap_or(0)
    }

    fn get_next_proof(&self, state_version: u64) -> Option<(Vec<LedgerPayloadHash>, Vec<u8>)> {
        let first_state_version = state_version + 1;
        let proof_version_key = db_key!(Proofs, &first_state_version.to_be_bytes());
        let (next_state_version, proof) = self
            .db
            .iterator(IteratorMode::From(&proof_version_key, Direction::Forward))
            .next()
            .map(|res| res.unwrap())
            .filter(|(key, _)| key[0] == Proofs.prefix())
            .map(|(key, proof)| {
                let (_, state_version_bytes) = key.split_first().unwrap();
                let next_state_version =
                    u64::from_be_bytes(state_version_bytes.try_into().unwrap());
                (next_state_version, proof.to_vec())
            })?;

        let mut tids = Vec::new();
        for v in first_state_version..=next_state_version {
            let txn_version_key = db_key!(StateVersions, &v.to_be_bytes());
            let bytes = self.db.get(txn_version_key).unwrap().unwrap();
            tids.push(LedgerPayloadHash::from_raw_bytes(
                bytes.try_into().expect("Payload hash is the wrong length"),
            ));
        }
        Some((tids, proof))
    }

    fn get_last_proof(&self) -> Option<Vec<u8>> {
        self.db
            .iterator(IteratorMode::From(
                &[Proofs.prefix() + 1],
                Direction::Reverse,
            ))
            .map(|res| res.unwrap())
            .next()
            .filter(|(key, _)| key[0] == 2u8)
            .map(|(_, proof)| proof.to_vec())
    }
}

impl QueryableSubstateStore for RocksDBStore {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        let id = scrypto_encode(&SubstateId(
            RENodeId::KeyValueStore(*kv_store_id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(vec![])),
        ))
        .unwrap();

        let iter = self.db.iterator(IteratorMode::From(
            &db_key!(Substates, &id),
            Direction::Forward,
        ));
        let mut items = HashMap::new();
        for res in iter {
            let (key, value) = res.unwrap();
            let (prefix, key) = key.split_first().unwrap();
            if *prefix != Substates.prefix() {
                break;
            }

            let substate: OutputValue = scrypto_decode(&value).unwrap();
            let substate_id: SubstateId = scrypto_decode(key).unwrap();
            if let SubstateId(
                RENodeId::KeyValueStore(id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) = substate_id
            {
                if id == *kv_store_id {
                    items.insert(key, substate.substate)
                } else {
                    break;
                }
            } else {
                break;
            };
        }
        items
    }
}

impl RecoverableVertexStore for RocksDBStore {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        self.db.get(db_key!(VertexStore, &[])).unwrap()
    }
}
