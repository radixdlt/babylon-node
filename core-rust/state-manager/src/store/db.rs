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

use crate::jni::java_structure::*;

use crate::store::traits::*;
use crate::store::{InMemoryStore, RocksDBStore};

use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::UserPayloadHash;

use radix_engine::ledger::{
    bootstrap, OutputValue, QueryableSubstateStore, ReadableSubstateStore, WriteableSubstateStore,
};
use radix_engine::model::PersistedSubstate;

use radix_engine_stores::memory_db::SerializedInMemorySubstateStore;

use crate::store::in_memory::InMemoryVertexStore;
use crate::store::rocks_db::RocksDBCommitTransaction;
use crate::store::traits::RecoverableVertexStore;
use crate::transaction::{LedgerTransaction, ValidatorTransaction};
use crate::{
    AccumulatorHash, CommittedTransactionIdentifiers, IntentHash, LedgerPayloadHash,
    LedgerTransactionReceipt,
};
use scrypto::engine::types::{KeyValueStoreId, SubstateId};

use tracing::debug;

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub enum DatabaseConfig {
    InMemory,
    RocksDB(String),
    None,
}

pub enum StateManagerDatabase {
    InMemory {
        transactions_and_proofs: InMemoryStore,
        substates: SerializedInMemorySubstateStore,
        vertices: InMemoryVertexStore,
    },
    RocksDB(RocksDBStore),
    None,
}

impl StateManagerDatabase {
    pub fn from_config(config: DatabaseConfig) -> Self {
        let mut state_manager_db = match config {
            DatabaseConfig::InMemory => StateManagerDatabase::InMemory {
                transactions_and_proofs: InMemoryStore::new(),
                substates: SerializedInMemorySubstateStore::new(),
                vertices: InMemoryVertexStore::new(),
            },
            DatabaseConfig::RocksDB(path) => {
                let db = RocksDBStore::new(PathBuf::from(path));
                StateManagerDatabase::RocksDB(db)
            }
            DatabaseConfig::None => StateManagerDatabase::None,
        };

        // Bootstrap genesis
        if !matches!(state_manager_db, StateManagerDatabase::None)
            && state_manager_db.max_state_version() == 0
        {
            debug!("Running genesis on the engine...");
            let mut db_txn = state_manager_db.create_db_transaction();

            let genesis_receipt = bootstrap(&mut db_txn).expect("Genesis wasn't run");

            // TODO: Remove this when serialized genesis intent is implemented
            {
                let ledger_receipt: LedgerTransactionReceipt = genesis_receipt
                    .try_into()
                    .expect("Failed to convert genesis receipt to LedgerTransactionReceipt");

                let mock_genesis =
                    LedgerTransaction::Validator(ValidatorTransaction::EpochUpdate(0)); // Mocked
                let payload_hash = mock_genesis.get_hash();
                let identifiers = CommittedTransactionIdentifiers {
                    state_version: 1,
                    accumulator_hash: AccumulatorHash::pre_genesis().accumulate(&payload_hash),
                };
                db_txn.insert_committed_transactions(vec![(
                    mock_genesis,
                    ledger_receipt,
                    identifiers,
                )]);
                db_txn.insert_tids_without_proof(1, vec![payload_hash]);
            }

            db_txn.commit();
        }

        state_manager_db
    }
}

impl ReadableSubstateStore for StateManagerDatabase {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        match self {
            StateManagerDatabase::InMemory { substates, .. } => substates.get_substate(substate_id),
            StateManagerDatabase::RocksDB(store) => store.get_substate(substate_id),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

pub enum StateManagerCommitTransaction<'db> {
    InMemory {
        transactions_and_proofs: &'db mut InMemoryStore,
        substates: &'db mut SerializedInMemorySubstateStore,
        vertices: &'db mut InMemoryVertexStore,
    },
    RocksDB(RocksDBCommitTransaction<'db>),
}

impl<'db> ReadableSubstateStore for StateManagerCommitTransaction<'db> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        match self {
            StateManagerCommitTransaction::InMemory { substates, .. } => {
                substates.get_substate(substate_id)
            }
            StateManagerCommitTransaction::RocksDB(db_txn) => db_txn.get_substate(substate_id),
        }
    }
}

impl<'db> WriteableTransactionStore for StateManagerCommitTransaction<'db> {
    fn insert_committed_transactions(
        &mut self,
        transactions: Vec<(
            LedgerTransaction,
            LedgerTransactionReceipt,
            CommittedTransactionIdentifiers,
        )>,
    ) {
        match self {
            StateManagerCommitTransaction::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.insert_committed_transactions(transactions),
            StateManagerCommitTransaction::RocksDB(db_txn) => {
                db_txn.insert_committed_transactions(transactions)
            }
        }
    }
}

impl<'db> WriteableProofStore for StateManagerCommitTransaction<'db> {
    fn insert_tids_and_proof(
        &mut self,
        state_version: u64,
        ids: Vec<LedgerPayloadHash>,
        proof_bytes: Vec<u8>,
    ) {
        match self {
            StateManagerCommitTransaction::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.insert_tids_and_proof(state_version, ids, proof_bytes),
            StateManagerCommitTransaction::RocksDB(db_txn) => {
                db_txn.insert_tids_and_proof(state_version, ids, proof_bytes)
            }
        }
    }

    fn insert_tids_without_proof(&mut self, state_version: u64, ids: Vec<LedgerPayloadHash>) {
        match self {
            StateManagerCommitTransaction::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.insert_tids_without_proof(state_version, ids),
            StateManagerCommitTransaction::RocksDB(db_txn) => {
                db_txn.insert_tids_without_proof(state_version, ids)
            }
        }
    }
}

impl<'db> WriteableSubstateStore for StateManagerCommitTransaction<'db> {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        match self {
            StateManagerCommitTransaction::InMemory { substates, .. } => {
                substates.put_substate(substate_id, substate)
            }
            StateManagerCommitTransaction::RocksDB(db_txn) => {
                db_txn.put_substate(substate_id, substate)
            }
        }
    }
}

impl<'db> WriteableVertexStore for StateManagerCommitTransaction<'db> {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        match self {
            StateManagerCommitTransaction::InMemory { vertices, .. } => {
                vertices.save_vertex_store(vertex_store_bytes)
            }
            StateManagerCommitTransaction::RocksDB(db_txn) => {
                db_txn.save_vertex_store(vertex_store_bytes)
            }
        }
    }
}

impl<'db> CommitStoreTransaction<'db> for StateManagerCommitTransaction<'db> {
    fn commit(self) {
        match self {
            StateManagerCommitTransaction::InMemory { .. } => {}
            StateManagerCommitTransaction::RocksDB(db_txn) => db_txn.commit(),
        }
    }
}

impl<'db> CommitStore<'db> for StateManagerDatabase {
    type DBTransaction = StateManagerCommitTransaction<'db>;

    fn create_db_transaction(&'db mut self) -> StateManagerCommitTransaction<'db> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                substates,
                vertices,
            } => StateManagerCommitTransaction::InMemory {
                transactions_and_proofs,
                substates,
                vertices,
            },
            StateManagerDatabase::RocksDB(store) => {
                let db_txn = store.create_db_transaction();
                StateManagerCommitTransaction::RocksDB(db_txn)
            }
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl QueryableTransactionStore for StateManagerDatabase {
    #[tracing::instrument(skip_all)]
    fn get_committed_transaction(
        &self,
        payload_hash: &LedgerPayloadHash,
    ) -> Option<(
        LedgerTransaction,
        LedgerTransactionReceipt,
        CommittedTransactionIdentifiers,
    )> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.get_committed_transaction(payload_hash),
            StateManagerDatabase::RocksDB(store) => store.get_committed_transaction(payload_hash),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl TransactionIndex<&UserPayloadHash> for StateManagerDatabase {
    fn get_payload_hash(&self, identifier: &UserPayloadHash) -> Option<LedgerPayloadHash> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.get_payload_hash(identifier),
            StateManagerDatabase::RocksDB(store) => store.get_payload_hash(identifier),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl TransactionIndex<&IntentHash> for StateManagerDatabase {
    fn get_payload_hash(&self, identifier: &IntentHash) -> Option<LedgerPayloadHash> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.get_payload_hash(identifier),
            StateManagerDatabase::RocksDB(store) => store.get_payload_hash(identifier),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl TransactionIndex<u64> for StateManagerDatabase {
    fn get_payload_hash(&self, state_version: u64) -> Option<LedgerPayloadHash> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.get_payload_hash(state_version),
            StateManagerDatabase::RocksDB(store) => store.get_payload_hash(state_version),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl QueryableProofStore for StateManagerDatabase {
    fn max_state_version(&self) -> u64 {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.max_state_version(),
            StateManagerDatabase::RocksDB(store) => store.max_state_version(),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }

    fn get_next_proof(&self, state_version: u64) -> Option<(Vec<LedgerPayloadHash>, Vec<u8>)> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.get_next_proof(state_version),
            StateManagerDatabase::RocksDB(store) => store.get_next_proof(state_version),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }

    fn get_last_proof(&self) -> Option<Vec<u8>> {
        match self {
            StateManagerDatabase::InMemory {
                transactions_and_proofs,
                ..
            } => transactions_and_proofs.get_last_proof(),
            StateManagerDatabase::RocksDB(store) => store.get_last_proof(),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl QueryableSubstateStore for StateManagerDatabase {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        match self {
            StateManagerDatabase::InMemory { substates, .. } => {
                substates.get_kv_store_entries(kv_store_id)
            }
            StateManagerDatabase::RocksDB(store) => store.get_kv_store_entries(kv_store_id),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}

impl RecoverableVertexStore for StateManagerDatabase {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        match self {
            StateManagerDatabase::InMemory { vertices, .. } => vertices.get_vertex_store(),
            StateManagerDatabase::RocksDB(store) => store.get_vertex_store(),
            StateManagerDatabase::None => panic!("Unexpected call to no state manager store"),
        }
    }
}
