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

use crate::store::traits::CommitBundle;
use crate::store::traits::*;
use crate::store::{InMemoryStore, RocksDBStore};

use std::path::PathBuf;

use crate::types::UserPayloadHash;

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::store::traits::RecoverableVertexStore;
use crate::transaction::LedgerTransaction;
use crate::{
    CommittedTransactionIdentifiers, IntentHash, LedgerPayloadHash, LedgerProof,
    LocalTransactionReceipt, ReceiptTreeHash, TransactionTreeHash,
};
use radix_engine_stores::hash_tree::tree_store::{NodeKey, Payload, ReadableTreeStore, TreeNode};
use radix_engine_stores::interface::SubstateDatabase;

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub enum DatabaseConfig {
    InMemory,
    RocksDB(String),
}

pub enum StateManagerDatabase {
    InMemory(InMemoryStore),
    RocksDB(RocksDBStore),
}

impl StateManagerDatabase {
    pub fn from_config(config: DatabaseConfig) -> Self {
        match config {
            DatabaseConfig::InMemory => StateManagerDatabase::InMemory(InMemoryStore::new()),
            DatabaseConfig::RocksDB(path) => {
                let db = RocksDBStore::new(PathBuf::from(path));
                StateManagerDatabase::RocksDB(db)
            }
        }
    }
}

impl SubstateDatabase for StateManagerDatabase {
    fn get_substate(&self, index_id: &Vec<u8>, key: &Vec<u8>) -> Option<Vec<u8>> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_substate(index_id, key),
            StateManagerDatabase::RocksDB(store) => store.get_substate(index_id, key),
        }
    }

    fn list_substates(
        &self,
        index_id: &Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        match self {
            StateManagerDatabase::InMemory(store) => store.list_substates(index_id),
            StateManagerDatabase::RocksDB(store) => store.list_substates(index_id),
        }
    }
}

impl<P: Payload> ReadableTreeStore<P> for StateManagerDatabase {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_node(key),
            StateManagerDatabase::RocksDB(store) => store.get_node(key),
        }
    }
}

impl ReadableAccuTreeStore<u64, TransactionTreeHash> for StateManagerDatabase {
    fn get_tree_slice(&self, state_version: &u64) -> Option<TreeSlice<TransactionTreeHash>> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_tree_slice(state_version),
            StateManagerDatabase::RocksDB(store) => store.get_tree_slice(state_version),
        }
    }
}

impl ReadableAccuTreeStore<u64, ReceiptTreeHash> for StateManagerDatabase {
    fn get_tree_slice(&self, state_version: &u64) -> Option<TreeSlice<ReceiptTreeHash>> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_tree_slice(state_version),
            StateManagerDatabase::RocksDB(store) => store.get_tree_slice(state_version),
        }
    }
}

impl CommitStore for StateManagerDatabase {
    fn commit(&mut self, commit_bundle: CommitBundle) {
        match self {
            StateManagerDatabase::InMemory(store) => store.commit(commit_bundle),
            StateManagerDatabase::RocksDB(store) => store.commit(commit_bundle),
        }
    }
}

impl QueryableTransactionStore for StateManagerDatabase {
    #[tracing::instrument(skip_all)]
    fn get_committed_transaction_bundles(
        &self,
        start_state_version_inclusive: u64,
        limit: usize,
    ) -> Vec<CommittedTransactionBundle> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                store.get_committed_transaction_bundles(start_state_version_inclusive, limit)
            }
            StateManagerDatabase::RocksDB(store) => {
                store.get_committed_transaction_bundles(start_state_version_inclusive, limit)
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn get_committed_transaction(&self, state_version: u64) -> Option<LedgerTransaction> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_committed_transaction(state_version),
            StateManagerDatabase::RocksDB(store) => store.get_committed_transaction(state_version),
        }
    }

    #[tracing::instrument(skip_all)]
    fn get_committed_transaction_receipt(
        &self,
        state_version: u64,
    ) -> Option<LocalTransactionReceipt> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                store.get_committed_transaction_receipt(state_version)
            }
            StateManagerDatabase::RocksDB(store) => {
                store.get_committed_transaction_receipt(state_version)
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn get_committed_transaction_identifiers(
        &self,
        state_version: u64,
    ) -> Option<CommittedTransactionIdentifiers> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                store.get_committed_transaction_identifiers(state_version)
            }
            StateManagerDatabase::RocksDB(store) => {
                store.get_committed_transaction_identifiers(state_version)
            }
        }
    }
}

impl TransactionIndex<&IntentHash> for StateManagerDatabase {
    fn get_txn_state_version_by_identifier(&self, identifier: &IntentHash) -> Option<u64> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
            StateManagerDatabase::RocksDB(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
        }
    }
}

impl TransactionIndex<&UserPayloadHash> for StateManagerDatabase {
    fn get_txn_state_version_by_identifier(&self, identifier: &UserPayloadHash) -> Option<u64> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
            StateManagerDatabase::RocksDB(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
        }
    }
}

impl TransactionIndex<&LedgerPayloadHash> for StateManagerDatabase {
    fn get_txn_state_version_by_identifier(&self, identifier: &LedgerPayloadHash) -> Option<u64> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
            StateManagerDatabase::RocksDB(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
        }
    }
}

impl TransactionIdentifierLoader for StateManagerDatabase {
    fn get_top_transaction_identifiers(&self) -> CommittedTransactionIdentifiers {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_top_transaction_identifiers(),
            StateManagerDatabase::RocksDB(store) => store.get_top_transaction_identifiers(),
        }
    }
}

impl QueryableProofStore for StateManagerDatabase {
    fn max_state_version(&self) -> u64 {
        match self {
            StateManagerDatabase::InMemory(store) => store.max_state_version(),
            StateManagerDatabase::RocksDB(store) => store.max_state_version(),
        }
    }

    fn get_txns_and_proof(
        &self,
        start_state_version_inclusive: u64,
        max_number_of_txns_if_more_than_one_proof: u32,
        max_payload_size_in_bytes: u32,
    ) -> Option<(Vec<Vec<u8>>, LedgerProof)> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_txns_and_proof(
                start_state_version_inclusive,
                max_number_of_txns_if_more_than_one_proof,
                max_payload_size_in_bytes,
            ),
            StateManagerDatabase::RocksDB(store) => store.get_txns_and_proof(
                start_state_version_inclusive,
                max_number_of_txns_if_more_than_one_proof,
                max_payload_size_in_bytes,
            ),
        }
    }

    fn get_epoch_proof(&self, epoch: u64) -> Option<LedgerProof> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_epoch_proof(epoch),
            StateManagerDatabase::RocksDB(store) => store.get_epoch_proof(epoch),
        }
    }

    fn get_last_proof(&self) -> Option<LedgerProof> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_last_proof(),
            StateManagerDatabase::RocksDB(store) => store.get_last_proof(),
        }
    }

    fn get_last_epoch_proof(&self) -> Option<LedgerProof> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_last_epoch_proof(),
            StateManagerDatabase::RocksDB(store) => store.get_last_epoch_proof(),
        }
    }
}

impl WriteableVertexStore for StateManagerDatabase {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        match self {
            StateManagerDatabase::InMemory(store) => store.save_vertex_store(vertex_store_bytes),
            StateManagerDatabase::RocksDB(store) => store.save_vertex_store(vertex_store_bytes),
        }
    }
}

impl RecoverableVertexStore for StateManagerDatabase {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_vertex_store(),
            StateManagerDatabase::RocksDB(store) => store.get_vertex_store(),
        }
    }
}
