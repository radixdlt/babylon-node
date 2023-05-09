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
use crate::store::{InMemoryStore, RocksDBStore};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::UserPayloadHash;

use enum_dispatch::enum_dispatch;
use radix_engine::ledger::{OutputValue, QueryableSubstateStore, ReadableSubstateStore};
use radix_engine::system::node_substates::PersistedSubstate;

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::CommittedTransactionIdentifiers;
use crate::{IntentHash, LedgerPayloadHash, ReceiptTreeHash, TransactionTreeHash};
use radix_engine::types::*;
use radix_engine::types::{KeyValueStoreId, SubstateId};
use radix_engine_stores::hash_tree::tree_store::{NodeKey, Payload, ReadableTreeStore, TreeNode};

use super::in_memory::InMemoryCommittedTransactionBundleIterator;
use super::rocks_db::RocksDBCommittedTransactionBundleIterator;

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub enum DatabaseBackendConfig {
    InMemory,
    RocksDB(String),
}

// As of May 2023, enum_dispatch does not work with generic traits (or other libraries that do the same).
// We can also extend code generation for remaining local (declared in this crate) traits once
// trait aliases/specialization makes it into stable Rust.
// Unfortunately this doesn't work across crates since it's a proc_macro (i.e. for ReadableSubstateStore).
#[enum_dispatch(
    ConfigurableDatabase,
    QueryableProofStore,
    TransactionIdentifierLoader,
    WriteableVertexStore,
    RecoverableVertexStore,
    AccountChangeIndexExtension,
    QueryableTransactionStore,
    CommitStore
)]
pub enum StateManagerDatabase {
    InMemory(InMemoryStore),
    RocksDB(RocksDBStore),
}

impl StateManagerDatabase {
    pub fn from_config(backend_config: DatabaseBackendConfig, flags: DatabaseFlags) -> Self {
        match backend_config {
            DatabaseBackendConfig::InMemory => {
                let store = InMemoryStore::new(flags);
                StateManagerDatabase::InMemory(store)
            }
            DatabaseBackendConfig::RocksDB(path) => {
                let db = {
                    match RocksDBStore::new(PathBuf::from(path), flags) {
                        Ok(db) => db,
                        Err(error) => {
                            match error {
                                DatabaseConfigValidationError::AccountChangeIndexRequiresLocalTransactionExecutionIndex => {
                                    panic!("Local transaction execution index needs to be enabled in order for account change index to work.")
                                },
                                DatabaseConfigValidationError::LocalTransactionExecutionIndexChanged => {
                                    panic!("Local transaction execution index can not be changed once configured.\n\
                                            If you need to change it, please wipe ledger data and resync.\n")
                                }
                            }
                        }
                    }
                };
                StateManagerDatabase::RocksDB(db)
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum StateManagerDatabaseCommittedTransactionBundleIterator<'a> {
    InMemory(InMemoryCommittedTransactionBundleIterator<'a>),
    RocksDB(RocksDBCommittedTransactionBundleIterator<'a>),
}

impl Iterator for StateManagerDatabaseCommittedTransactionBundleIterator<'_> {
    type Item = CommittedTransactionBundle;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StateManagerDatabaseCommittedTransactionBundleIterator::InMemory(iterator) => {
                iterator.next()
            }
            StateManagerDatabaseCommittedTransactionBundleIterator::RocksDB(iterator) => {
                iterator.next()
            }
        }
    }
}

impl IterableTransactionStore for StateManagerDatabase {
    type CommittedTransactionBundleIterator<'a> =
        StateManagerDatabaseCommittedTransactionBundleIterator<'a>;

    fn get_committed_transaction_bundle_iter(
        &self,
        from_state_version: u64,
    ) -> Self::CommittedTransactionBundleIterator<'_> {
        match self {
            StateManagerDatabase::InMemory(store) => {
                StateManagerDatabaseCommittedTransactionBundleIterator::InMemory(
                    store.get_committed_transaction_bundle_iter(from_state_version),
                )
            }
            StateManagerDatabase::RocksDB(store) => {
                StateManagerDatabaseCommittedTransactionBundleIterator::RocksDB(
                    store.get_committed_transaction_bundle_iter(from_state_version),
                )
            }
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

impl<P: Payload> ReadableTreeStore<P> for StateManagerDatabase {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_node(key),
            StateManagerDatabase::RocksDB(store) => store.get_node(key),
        }
    }
}

impl ReadableSubstateStore for StateManagerDatabase {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_substate(substate_id),
            StateManagerDatabase::RocksDB(store) => store.get_substate(substate_id),
        }
    }
}

impl QueryableSubstateStore for StateManagerDatabase {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        match self {
            StateManagerDatabase::InMemory(store) => store.get_kv_store_entries(kv_store_id),
            StateManagerDatabase::RocksDB(store) => store.get_kv_store_entries(kv_store_id),
        }
    }
}
