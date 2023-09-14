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
use crate::transaction::*;
use crate::{CommittedTransactionIdentifiers, LedgerProof, LocalTransactionReceipt, StateVersion};
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

pub struct CommittedTransactionBundle {
    pub state_version: StateVersion,
    pub raw: RawLedgerTransaction,
    pub receipt: LocalTransactionReceipt,
    pub identifiers: CommittedTransactionIdentifiers,
}

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
    use super::*;
    use radix_engine::types::{ScryptoCategorize, ScryptoDecode, ScryptoEncode};
    use radix_engine_common::types::NodeId;
    use std::slice;

    use crate::SubstateReference;
    pub use radix_engine_store_interface::interface::{
        CommittableSubstateDatabase, SubstateDatabase,
    };

    /// A low-level storage of [`SubstateNodeAncestryRecord`].
    /// API note: this trait defines a simple "get by ID" method, and also a performance-driven
    /// batch method. Both provide default implementations (which mutually reduce one problem to the
    /// other). The implementer must choose to implement at least one of the methods, based on its
    /// nature (though implementing both rarely makes sense).
    #[enum_dispatch]
    pub trait SubstateNodeAncestryStore {
        /// Returns the [`SubstateNodeAncestryRecord`] for the given [`NodeId`], or [`None`] if:
        /// - the `node_id` happens to be a root Node (since they do not have "ancestry");
        /// - or the `node_id` does not exist yet.
        fn get_ancestry(&self, node_id: &NodeId) -> Option<SubstateNodeAncestryRecord> {
            let records = self.batch_get_ancestry(slice::from_ref(node_id));
            if records.len() != 1 {
                panic!(
                    "trait contract violated: expected a single result for {:?}, got {:?}",
                    node_id, records
                )
            }
            records.into_iter().next().unwrap()
        }

        /// A batch counterpart of the [`get_ancestry()`].
        /// The results are returned in the same order as the input `node_ids`.
        fn batch_get_ancestry<'a>(
            &self,
            node_ids: impl IntoIterator<Item = &'a NodeId>,
        ) -> Vec<Option<SubstateNodeAncestryRecord>> {
            node_ids
                .into_iter()
                .map(|node_id| self.get_ancestry(node_id))
                .collect()
        }
    }

    /// Ancestry information of a RE Node.
    #[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
    pub struct SubstateNodeAncestryRecord {
        /// A substate owning the Node (i.e. its immediate parent).
        /// Note: this will always be present, since we do not need ancestry of root RE Nodes.
        pub parent: SubstateReference,
        /// A root ancestor of the Node's tree (i.e. the top of its parent chain).
        /// Note: the returned reference is guaranteed to resolve to a [`GlobalAddress`].
        pub root: SubstateReference,
    }

    /// A [`SubstateNodeAncestryRecord`] accompanied by a set of sibling [`NodeId`]s (which of
    /// course share the same parent).
    pub type KeyedSubstateNodeAncestryRecord = (Vec<NodeId>, SubstateNodeAncestryRecord);
}

pub mod transactions {
    use super::*;

    use crate::store::traits::CommittedTransactionBundle;
    use crate::{
        CommittedTransactionIdentifiers, LedgerHashes, LedgerTransactionReceipt,
        LocalTransactionExecution, LocalTransactionReceipt,
    };

    #[enum_dispatch]
    pub trait IterableTransactionStore {
        fn get_committed_transaction_bundle_iter(
            &self,
            from_state_version: StateVersion,
        ) -> Box<dyn Iterator<Item = CommittedTransactionBundle> + '_>;
    }

    #[enum_dispatch]
    pub trait QueryableTransactionStore {
        fn get_committed_transaction(
            &self,
            state_version: StateVersion,
        ) -> Option<RawLedgerTransaction>;

        fn get_committed_transaction_identifiers(
            &self,
            state_version: StateVersion,
        ) -> Option<CommittedTransactionIdentifiers>;

        fn get_committed_ledger_transaction_receipt(
            &self,
            state_version: StateVersion,
        ) -> Option<LedgerTransactionReceipt>;

        fn get_committed_local_transaction_execution(
            &self,
            state_version: StateVersion,
        ) -> Option<LocalTransactionExecution>;

        fn get_committed_local_transaction_receipt(
            &self,
            state_version: StateVersion,
        ) -> Option<LocalTransactionReceipt>;

        fn get_committed_ledger_hashes(&self, state_version: StateVersion) -> Option<LedgerHashes> {
            self.get_committed_transaction_identifiers(state_version)
                .map(|ids| ids.resultant_ledger_hashes)
        }
    }

    pub trait TransactionIndex<T>: QueryableTransactionStore {
        fn get_txn_state_version_by_identifier(&self, identifier: T) -> Option<StateVersion>;
    }
}

pub mod proofs {
    use radix_engine_common::types::Epoch;

    use super::*;

    #[enum_dispatch]
    pub trait QueryableProofStore {
        fn max_state_version(&self) -> StateVersion;
        fn get_txns_and_proof(
            &self,
            start_state_version_inclusive: StateVersion,
            max_number_of_txns_if_more_than_one_proof: u32,
            max_payload_size_in_bytes: u32,
        ) -> Option<(Vec<RawLedgerTransaction>, LedgerProof)>;
        fn get_first_proof(&self) -> Option<LedgerProof>;
        fn get_post_genesis_epoch_proof(&self) -> Option<LedgerProof>;
        fn get_epoch_proof(&self, epoch: Epoch) -> Option<LedgerProof>;
        fn get_last_proof(&self) -> Option<LedgerProof>;
        fn get_last_epoch_proof(&self) -> Option<LedgerProof>;
    }
}

pub mod commit {
    use super::*;
    use crate::accumulator_tree::storage::TreeSlice;
    use crate::{ReceiptTreeHash, StateVersion, TransactionTreeHash};

    use radix_engine_store_interface::interface::{
        DatabaseUpdate, DatabaseUpdates, NodeDatabaseUpdates, PartitionDatabaseUpdates,
    };
    use radix_engine_stores::hash_tree::tree_store::{NodeKey, StaleTreePart, TreeNode};

    pub struct CommitBundle {
        pub transactions: Vec<CommittedTransactionBundle>,
        pub proof: LedgerProof,
        pub substate_store_update: SubstateStoreUpdate,
        pub vertex_store: Option<Vec<u8>>,
        pub state_tree_update: HashTreeUpdate,
        pub transaction_tree_slice: TreeSlice<TransactionTreeHash>,
        pub receipt_tree_slice: TreeSlice<ReceiptTreeHash>,
        pub new_substate_node_ancestry_records: Vec<KeyedSubstateNodeAncestryRecord>,
    }

    pub struct SubstateStoreUpdate {
        pub updates: DatabaseUpdates,
    }

    impl SubstateStoreUpdate {
        pub fn new() -> Self {
            Self {
                updates: DatabaseUpdates::default(),
            }
        }

        pub fn from_single(database_updates: DatabaseUpdates) -> Self {
            Self {
                updates: database_updates,
            }
        }

        pub fn apply(&mut self, database_updates: DatabaseUpdates) {
            if self.updates.node_updates.is_empty() {
                self.updates = database_updates;
                return;
            }
            for (node_key, node_updates) in database_updates.node_updates {
                Self::merge_in_node_updates(
                    self.updates.node_updates.entry(node_key).or_default(),
                    node_updates,
                );
            }
        }

        fn merge_in_node_updates(target: &mut NodeDatabaseUpdates, source: NodeDatabaseUpdates) {
            for (partition_num, partition_updates) in source.partition_updates {
                Self::merge_in_partition_updates(
                    target.partition_updates.entry(partition_num).or_default(),
                    partition_updates,
                );
            }
        }

        fn merge_in_partition_updates(
            target: &mut PartitionDatabaseUpdates,
            source: PartitionDatabaseUpdates,
        ) {
            match source {
                PartitionDatabaseUpdates::Delta {
                    substate_updates: source_updates,
                } => match target {
                    PartitionDatabaseUpdates::Delta {
                        substate_updates: target_updates,
                    } => {
                        target_updates.extend(source_updates);
                    }
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values: target_values,
                    } => {
                        for (sort_key, update) in source_updates {
                            match update {
                                DatabaseUpdate::Set(value) => {
                                    target_values.insert(sort_key, value);
                                }
                                DatabaseUpdate::Delete => {
                                    let existed = target_values.remove(&sort_key).is_some();
                                    if !existed {
                                        panic!("broken invariant: deleting non-existent substate");
                                    }
                                }
                            }
                        }
                    }
                },
                PartitionDatabaseUpdates::Reset { .. } => {
                    *target = source;
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
        pub new_nodes: Vec<(NodeKey, TreeNode)>,
        pub stale_tree_parts_at_state_version: Vec<(StateVersion, Vec<StaleTreePart>)>,
    }

    impl HashTreeUpdate {
        pub fn new() -> Self {
            Self {
                new_nodes: Vec::new(),
                stale_tree_parts_at_state_version: Vec::new(),
            }
        }

        pub fn from_single(at_state_version: StateVersion, diff: StateHashTreeDiff) -> Self {
            Self {
                new_nodes: diff.new_nodes,
                stale_tree_parts_at_state_version: vec![(at_state_version, diff.stale_tree_parts)],
            }
        }

        pub fn add(&mut self, at_state_version: StateVersion, diff: StateHashTreeDiff) {
            self.new_nodes.extend(diff.new_nodes);
            self.stale_tree_parts_at_state_version
                .push((at_state_version, diff.stale_tree_parts));
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

pub mod scenario {
    use super::*;

    use transaction::model::IntentHash;

    pub type ScenarioSequenceNumber = u32;

    #[derive(Debug, Clone, Categorize, Encode, Decode)]
    pub struct ExecutedGenesisScenario {
        pub logical_name: String,
        pub committed_transactions: Vec<ExecutedScenarioTransaction>,
        pub addresses: Vec<DescribedAddress>,
    }

    #[derive(Debug, Clone, Categorize, Encode, Decode)]
    pub struct DescribedAddress {
        pub logical_name: String,
        pub rendered_address: String, // we store it pre-rendered, since `GlobalAddress` has no SBOR coding
    }

    #[derive(Debug, Clone, Categorize, Encode, Decode)]
    pub struct ExecutedScenarioTransaction {
        pub logical_name: String,
        pub state_version: StateVersion,
        pub intent_hash: IntentHash,
    }

    /// A store of testing-specific [`ExecutedGenesisScenario`], meant to be as separated as
    /// possible from the production stores (e.g. the writes happening outside of the regular commit
    /// batch write).
    #[enum_dispatch]
    pub trait ExecutedGenesisScenarioStore {
        /// Writes the given Scenario under a caller-managed sequence number (which means: it allows
        /// overwriting, writing out-of-order, leaving gaps, etc.).
        fn put_scenario(
            &mut self,
            number: ScenarioSequenceNumber,
            scenario: ExecutedGenesisScenario,
        );

        /// Returns all Scenarios written so far, ordered by their sequence numbers (but with no
        /// guarantees regarding gaps; see [`put_scenario()`]'s contract).
        /// Performance note: this method assumes a small number of Scenarios.
        fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedGenesisScenario)>;
    }
}

pub mod extensions {
    use super::*;
    use radix_engine::types::GlobalAddress;

    #[enum_dispatch]
    pub trait AccountChangeIndexExtension {
        fn account_change_index_last_processed_state_version(&self) -> StateVersion;

        fn catchup_account_change_index(&mut self);
    }

    #[enum_dispatch]
    pub trait IterableAccountChangeIndex {
        fn get_state_versions_for_account_iter(
            &self,
            account: GlobalAddress,
            from_state_version: StateVersion,
        ) -> Box<dyn Iterator<Item = StateVersion> + '_>;
    }
}
