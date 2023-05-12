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

use crate::staging::StateHashTreeDiff;
use crate::store::StateManagerDatabase;
use crate::transaction::LedgerTransaction;
use crate::{CommittedTransactionIdentifiers, LedgerProof, LocalTransactionReceipt};
pub use commit::*;
use enum_dispatch::enum_dispatch;
pub use proofs::*;
use radix_engine_common::{Categorize, Decode, Encode};
pub use substate::*;
pub use transactions::*;
pub use vertex::*;

pub enum DatabaseConfigValidationError {
    AccountChangeIndexRequiresLocalTransactionExecutionIndex,
    LocalTransactionExecutionIndexChanged,
}

/// Database flags required for initialization built from
/// config file and environment variables.
#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct DatabaseFlags {
    pub enable_local_transaction_execution_index: bool,
    pub enable_account_change_index: bool,
}

impl Default for DatabaseFlags {
    fn default() -> Self {
        DatabaseFlags {
            enable_local_transaction_execution_index: true,
            enable_account_change_index: true,
        }
    }
}

/// Current state of database configuration. We need Option<T> for
/// fields that are missing. Missing fields usually mean the database is
/// just being initialized (when all of the fields are None) but also
/// when new configurations are added - this is a cheap work around to
/// limit future needed ledger wipes until we have a better solution.
pub struct DatabaseFlagsState {
    pub local_transaction_execution_index_enabled: Option<bool>,
    pub account_change_index_enabled: Option<bool>,
}

impl DatabaseFlags {
    pub fn validate(
        &self,
        current_database_config: &DatabaseFlagsState,
    ) -> Result<(), DatabaseConfigValidationError> {
        if !self.enable_local_transaction_execution_index && self.enable_account_change_index {
            return Err(DatabaseConfigValidationError::AccountChangeIndexRequiresLocalTransactionExecutionIndex);
        }
        if let Some(local_transaction_execution_index_enabled) =
            current_database_config.local_transaction_execution_index_enabled
        {
            if self.enable_local_transaction_execution_index
                != local_transaction_execution_index_enabled
            {
                return Err(DatabaseConfigValidationError::LocalTransactionExecutionIndexChanged);
            }
        }
        Ok(())
    }
}

#[enum_dispatch]
pub trait ConfigurableDatabase {
    fn read_flags_state(&self) -> DatabaseFlagsState;

    fn write_flags(&mut self, flags: &DatabaseFlags);

    fn is_account_change_index_enabled(&self) -> bool;

    fn is_local_transaction_execution_index_enabled(&self) -> bool;
}

pub type CommittedTransactionBundle = (
    LedgerTransaction,
    LocalTransactionReceipt,
    CommittedTransactionIdentifiers,
);

pub mod vertex {
    use super::*;

    #[enum_dispatch]
    pub trait RecoverableVertexStore {
        fn get_vertex_store(&self) -> Option<Vec<u8>>;
    }

    #[enum_dispatch]
    pub trait WriteableVertexStore {
        fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>);
    }
}

pub mod substate {
    pub use radix_engine_store_interface::interface::{
        CommittableSubstateDatabase, SubstateDatabase,
    };
}

pub mod transactions {
    use super::*;

    use crate::store::traits::CommittedTransactionBundle;
    use crate::transaction::LedgerTransaction;
    use crate::{
        CommittedTransactionIdentifiers, LedgerTransactionReceipt, LocalTransactionExecution,
        LocalTransactionReceipt,
    };

    #[enum_dispatch]
    pub trait QueryableTransactionStore {
        fn get_committed_transaction_bundles(
            &self,
            start_state_version_inclusive: u64,
            limit: usize,
        ) -> Vec<CommittedTransactionBundle>;

        fn get_committed_transaction(&self, state_version: u64) -> Option<LedgerTransaction>;

        fn get_committed_transaction_identifiers(
            &self,
            state_version: u64,
        ) -> Option<CommittedTransactionIdentifiers>;

        fn get_committed_ledger_transaction_receipt(
            &self,
            state_version: u64,
        ) -> Option<LedgerTransactionReceipt>;

        fn get_committed_local_transaction_execution(
            &self,
            state_version: u64,
        ) -> Option<LocalTransactionExecution>;

        fn get_committed_local_transaction_receipt(
            &self,
            state_version: u64,
        ) -> Option<LocalTransactionReceipt>;
    }

    #[enum_dispatch]
    pub trait TransactionIndex<T>: QueryableTransactionStore {
        fn get_txn_state_version_by_identifier(&self, identifier: T) -> Option<u64>;
    }
}

pub mod proofs {
    use super::*;

    #[enum_dispatch]
    pub trait QueryableProofStore {
        fn max_state_version(&self) -> u64;
        fn get_txns_and_proof(
            &self,
            start_state_version_inclusive: u64,
            max_number_of_txns_if_more_than_one_proof: u32,
            max_payload_size_in_bytes: u32,
        ) -> Option<(Vec<Vec<u8>>, LedgerProof)>;
        fn get_first_epoch_proof(&self) -> Option<LedgerProof>;
        fn get_epoch_proof(&self, epoch: u64) -> Option<LedgerProof>;
        fn get_last_proof(&self) -> Option<LedgerProof>;
        fn get_last_epoch_proof(&self) -> Option<LedgerProof>;
    }
}

pub mod commit {
    use super::*;
    use crate::accumulator_tree::storage::TreeSlice;
    use crate::{ReceiptTreeHash, TransactionTreeHash};

    use radix_engine_store_interface::interface::{
        DatabaseUpdate, DatabaseUpdates, DbPartitionKey, DbSortKey,
    };
    use radix_engine_stores::hash_tree::tree_store::{NodeKey, PartitionPayload, TreeNode};
    use utils::rust::collections::IndexMap;

    pub struct CommitBundle {
        pub transactions: Vec<CommittedTransactionBundle>,
        pub proof: LedgerProof,
        pub substate_store_update: SubstateStoreUpdate,
        pub vertex_store: Option<Vec<u8>>,
        pub state_tree_update: HashTreeUpdate,
        pub transaction_tree_slice: TreeSlice<TransactionTreeHash>,
        pub receipt_tree_slice: TreeSlice<ReceiptTreeHash>,
    }

    pub struct SubstateStoreUpdate {
        pub updates: IndexMap<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>>,
    }

    impl SubstateStoreUpdate {
        pub fn new() -> Self {
            Self {
                updates: IndexMap::new(),
            }
        }

        pub fn apply(&mut self, database_updates: DatabaseUpdates) {
            for (partition_key, partition_updates) in database_updates {
                let curr_partition_updates = self
                    .updates
                    .entry(partition_key)
                    .or_insert_with(IndexMap::new);
                for (sort_key, database_update) in partition_updates {
                    curr_partition_updates.insert(sort_key, database_update);
                }
            }
        }
    }

    impl Default for SubstateStoreUpdate {
        fn default() -> Self {
            Self::new()
        }
    }

    pub struct HashTreeUpdate {
        pub new_re_node_layer_nodes: Vec<(NodeKey, TreeNode<PartitionPayload>)>,
        pub new_substate_layer_nodes: Vec<(NodeKey, TreeNode<()>)>,
        pub stale_node_keys_at_state_version: Vec<(u64, Vec<NodeKey>)>,
    }

    impl HashTreeUpdate {
        pub fn new() -> Self {
            Self {
                new_re_node_layer_nodes: Vec::new(),
                new_substate_layer_nodes: Vec::new(),
                stale_node_keys_at_state_version: Vec::new(),
            }
        }

        pub fn add(&mut self, at_state_version: u64, diff: StateHashTreeDiff) {
            self.new_re_node_layer_nodes
                .extend(diff.new_re_node_layer_nodes);
            self.new_substate_layer_nodes
                .extend(diff.new_substate_layer_nodes);
            self.stale_node_keys_at_state_version
                .push((at_state_version, diff.stale_hash_tree_node_keys));
        }
    }

    impl Default for HashTreeUpdate {
        fn default() -> Self {
            Self::new()
        }
    }

    #[enum_dispatch]
    pub trait CommitStore {
        fn commit(&mut self, commit_bundle: CommitBundle);
    }
}

pub mod extensions {
    use super::*;
    use radix_engine::types::GlobalAddress;

    #[enum_dispatch]
    pub trait AccountChangeIndexExtension {
        fn account_change_index_last_processed_state_version(&self) -> u64;

        fn catchup_account_change_index(&mut self);

        fn get_state_versions_for_account(
            &self,
            account: GlobalAddress,
            start_state_version_inclusive: u64,
            limit: usize,
        ) -> Vec<u64>;
    }
}
