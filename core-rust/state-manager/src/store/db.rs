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
use crate::store::RocksDBStore;
use crate::transaction::LedgerTransactionHash;
use std::path::PathBuf;

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::{
    CommittedTransactionIdentifiers, LedgerHashes, ReceiptTreeHash, StateVersion,
    TransactionTreeHash,
};
use enum_dispatch::enum_dispatch;
use radix_engine_store_interface::interface::{
    DbPartitionKey, DbSortKey, DbSubstateValue, PartitionEntry, SubstateDatabase,
};
use transaction::model::*;

use radix_engine_stores::hash_tree::tree_store::{NodeKey, ReadableTreeStore, TreeNode};
use sbor::{Categorize, Decode, Encode};

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct DatabaseBackendConfig {
    pub rocks_db_path: String,
}

// As of May 2023, enum_dispatch does not work with generic traits (or other libraries that do the same).
// We can also extend code generation for remaining local (declared in this crate) traits once
// trait aliases/specialization makes it into stable Rust.
// Unfortunately this doesn't work across crates since it's a proc_macro (i.e. for ReadableSubstateStore).
#[enum_dispatch(
    ConfigurableDatabase,
    MeasurableDatabase,
    QueryableProofStore,
    TransactionIdentifierLoader,
    WriteableVertexStore,
    RecoverableVertexStore,
    AccountChangeIndexExtension,
    QueryableTransactionStore,
    CommitStore,
    SubstateNodeAncestryStore,
    IterableAccountChangeIndex,
    IterableTransactionStore,
    IterableProofStore,
    ExecutedGenesisScenarioStore,
    StateHashTreeGcStore,
    LedgerProofsGcStore
)]
pub enum StateManagerDatabase {
    // TODO(clean-up): After InMemoryDb was deleted, we can get rid of this middle-man as well.
    RocksDB(RocksDBStore),
}

impl StateManagerDatabase {
    pub fn from_config(backend_config: DatabaseBackendConfig, flags: DatabaseFlags) -> Self {
        let db = {
            match RocksDBStore::new(PathBuf::from(backend_config.rocks_db_path), flags) {
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

impl SubstateDatabase for StateManagerDatabase {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        match self {
            StateManagerDatabase::RocksDB(store) => store.get_substate(partition_key, sort_key),
        }
    }

    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        match self {
            StateManagerDatabase::RocksDB(store) => {
                store.list_entries_from(partition_key, from_sort_key)
            }
        }
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        match self {
            StateManagerDatabase::RocksDB(store) => store.list_entries(partition_key),
        }
    }
}

impl ReadableTreeStore for StateManagerDatabase {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        match self {
            StateManagerDatabase::RocksDB(store) => store.get_node(key),
        }
    }
}

impl ReadableAccuTreeStore<StateVersion, TransactionTreeHash> for StateManagerDatabase {
    fn get_tree_slice(
        &self,
        state_version: &StateVersion,
    ) -> Option<TreeSlice<TransactionTreeHash>> {
        match self {
            StateManagerDatabase::RocksDB(store) => store.get_tree_slice(state_version),
        }
    }
}

impl ReadableAccuTreeStore<StateVersion, ReceiptTreeHash> for StateManagerDatabase {
    fn get_tree_slice(&self, state_version: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        match self {
            StateManagerDatabase::RocksDB(store) => store.get_tree_slice(state_version),
        }
    }
}

impl TransactionIndex<&IntentHash> for StateManagerDatabase {
    fn get_txn_state_version_by_identifier(&self, identifier: &IntentHash) -> Option<StateVersion> {
        match self {
            StateManagerDatabase::RocksDB(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
        }
    }
}

impl TransactionIndex<&NotarizedTransactionHash> for StateManagerDatabase {
    fn get_txn_state_version_by_identifier(
        &self,
        identifier: &NotarizedTransactionHash,
    ) -> Option<StateVersion> {
        match self {
            StateManagerDatabase::RocksDB(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
        }
    }
}

impl TransactionIndex<&LedgerTransactionHash> for StateManagerDatabase {
    fn get_txn_state_version_by_identifier(
        &self,
        identifier: &LedgerTransactionHash,
    ) -> Option<StateVersion> {
        match self {
            StateManagerDatabase::RocksDB(store) => {
                store.get_txn_state_version_by_identifier(identifier)
            }
        }
    }
}
