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

use crate::prelude::*;

use crate::store::column_families::*;
use crate::store::historical_state::StateTreeBasedSubstateDatabase;
use crate::store::traits::*;
use rocksdb::*;

/// A listing of all column family names used by the Node.
///
/// This is directly needed to initialize the column families within the DB, but is also a nice
/// place to link to all of them (please see the documentation of each CF to learn about its
/// business purpose and DB schema) and to put the important general notes regarding all of them
/// (see below).
///
/// **Note on the key encoding used throughout all column families:**
/// We often rely on the RocksDB's unsurprising ability to efficiently list entries sorted
/// lexicographically by key. For this reason, our byte-level encoding of certain keys (e.g.
/// [`StateVersion`]) needs to reflect the business-level ordering of the represented concept (i.e.
/// since state versions grow, the "last" state version must have a lexicographically greatest key,
/// which means that we need to use a constant-length big-endian integer encoding).
///
/// **Note on the name strings:**
/// The `NAME` constants defined by `*Cf` structs (and referenced below) are used as database column
/// family names. Any change would effectively mean a ledger wipe. For this reason, we choose to
/// define them manually (rather than using the `Into<String>`, which is refactor-sensitive).
const ALL_STATE_MANAGER_COLUMN_FAMILIES: [&str; 25] = [
    RawLedgerTransactionsCf::DEFAULT_NAME,
    CommittedTransactionIdentifiersCf::VERSIONED_NAME,
    TransactionReceiptsCf::VERSIONED_NAME,
    LocalTransactionExecutionsCf::VERSIONED_NAME,
    IntentHashesCf::DEFAULT_NAME,
    NotarizedTransactionHashesCf::DEFAULT_NAME,
    LedgerTransactionHashesCf::DEFAULT_NAME,
    LedgerProofsCf::VERSIONED_NAME,
    EpochLedgerProofsCf::VERSIONED_NAME,
    ProtocolUpdateInitLedgerProofsCf::VERSIONED_NAME,
    ProtocolUpdateExecutionLedgerProofsCf::VERSIONED_NAME,
    SubstatesCf::DEFAULT_NAME,
    SubstateNodeAncestryRecordsCf::VERSIONED_NAME,
    VertexStoreCf::VERSIONED_NAME,
    StateTreeNodesCf::VERSIONED_NAME,
    StaleStateTreePartsCf::VERSIONED_NAME,
    TransactionAccuTreeSlicesCf::VERSIONED_NAME,
    ReceiptAccuTreeSlicesCf::VERSIONED_NAME,
    ExtensionsDataCf::NAME,
    AccountChangeStateVersionsCf::NAME,
    ExecutedScenariosCf::VERSIONED_NAME,
    LedgerProofsGcProgressCf::VERSIONED_NAME,
    AssociatedStateTreeValuesCf::DEFAULT_NAME,
    TypeAndCreationIndexedEntitiesCf::VERSIONED_NAME,
    BlueprintAndCreationIndexedObjectsCf::VERSIONED_NAME,
];

pub type ActualStateManagerDatabase = StateManagerDatabase<DirectRocks>;

impl<'db> Snapshottable<'db> for StateManagerDatabase<DirectRocks> {
    type Snapshot = StateManagerDatabase<SnapshotRocks<'db>>;

    // TODO(potential performance gain): This is the place where we could use a cached snapshot
    // instead of creating a new one. There are a few options: e.g. cache on-demand (after
    // detecting that DB version has grown) or actively hot-swap a snapshot after each batch-write.
    // However, maybe it's not worth optimizing for at all: according to the measurements from
    // RocksDB authors (https://github.com/facebook/rocksdb/issues/5083), rapid snapshotting *can*
    // become a performance problem, but only at rates way above our use-cases (i.e. >10K snapshots
    // per second).
    fn snapshot(&'db self) -> Self::Snapshot {
        let StateManagerDatabase { config, rocks } = self;
        StateManagerDatabase {
            config: config.clone(),
            rocks: rocks.snapshot(),
        }
    }
}

/// A RocksDB-backed persistence layer for state manager.
pub struct StateManagerDatabase<R> {
    /// Database config.
    ///
    /// The config is passed during construction, validated, persisted, and effectively immutable
    /// during the state manager's lifetime. This field only acts as a cache.
    config: DatabaseConfig,

    /// Underlying RocksDB instance.
    rocks: R,
}

impl ActualStateManagerDatabase {
    pub fn new(
        root_path: PathBuf,
        config: DatabaseConfig,
        network: &NetworkDefinition,
    ) -> Result<Self, DatabaseConfigValidationError> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_STATE_MANAGER_COLUMN_FAMILIES
            .iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&db_opts, root_path.as_path(), column_families).unwrap();

        let state_manager_database = StateManagerDatabase {
            config,
            rocks: DirectRocks { db },
        };

        state_manager_database.validate_and_persist_new_config()?;

        state_manager_database.catchup_account_change_index();
        state_manager_database.restore_december_2023_lost_substates(network);
        state_manager_database.ensure_historical_substate_values();
        state_manager_database.ensure_entity_listing_indices();

        Ok(state_manager_database)
    }

    /// Creates a readonly [`StateManagerDatabase`] that allows only reading from the store, while
    /// some other process is writing to it.
    ///
    /// This is required for the [`ledger-tools`] CLI tool which only reads data from the database
    /// and does not write anything to it. Without this constructor, if [`StateManagerDatabase::new`] is
    /// used by the [`ledger-tools`] CLI then it leads to a lock contention as two threads would
    /// want to have a write lock over the database. This provides the [`ledger-tools`] CLI with a
    /// way of making it clear that it only wants read lock and not a write lock.
    ///
    /// [`ledger-tools`]: https://github.com/radixdlt/ledger-tools
    pub fn new_read_only(root_path: PathBuf) -> Self {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(false);
        db_opts.create_missing_column_families(false);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_STATE_MANAGER_COLUMN_FAMILIES
            .iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors_read_only(
            &db_opts,
            root_path.as_path(),
            column_families,
            false,
        )
        .unwrap();

        StateManagerDatabase {
            config: DatabaseConfig {
                enable_local_transaction_execution_index: false,
                enable_account_change_index: false,
                enable_historical_substate_values: false,
                enable_entity_listing_indices: false,
            },
            rocks: DirectRocks { db },
        }
    }

    /// Creates a [`StateManagerDatabase`] as a secondary instance which may catch up with the
    /// primary.
    pub fn new_as_secondary(
        root_path: PathBuf,
        temp_path: PathBuf,
        column_families: Vec<&str>,
    ) -> Self {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(false);
        db_opts.create_missing_column_families(false);

        let column_families: Vec<ColumnFamilyDescriptor> = column_families
            .iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors_as_secondary(
            &db_opts,
            root_path.as_path(),
            temp_path.as_path(),
            column_families,
        )
        .unwrap();

        StateManagerDatabase {
            config: DatabaseConfig {
                enable_local_transaction_execution_index: false,
                enable_account_change_index: false,
                enable_historical_substate_values: false,
                enable_entity_listing_indices: false,
            },
            rocks: DirectRocks { db },
        }
    }

    pub fn try_catchup_with_primary(&self) {
        self.rocks.try_catchup_with_primary();
    }
}

impl<R: ReadableRocks> StateManagerDatabase<R> {
    /// Starts a read-only interaction with the DB through per-CF type-safe APIs.
    fn open_read_context(&self) -> TypedDbContext<R, NoWriteSupport> {
        TypedDbContext::new(&self.rocks, NoWriteSupport)
    }
}

impl<R: WriteableRocks> StateManagerDatabase<R> {
    /// Starts a read/buffered-write interaction with the DB through per-CF type-safe APIs.
    fn open_rw_context(&self) -> TypedDbContext<R, BufferedWriteSupport<R>> {
        TypedDbContext::new(&self.rocks, BufferedWriteSupport::new(&self.rocks))
    }
}

impl<R: WriteableRocks> StateManagerDatabase<R> {
    fn validate_and_persist_new_config(&self) -> Result<(), DatabaseConfigValidationError> {
        let stored_config_state = self.read_config_state();
        self.config.validate(&stored_config_state)?;
        self.write_config();
        Ok(())
    }

    fn read_config_state(&self) -> DatabaseConfigState {
        let db_context = self.open_read_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        let account_change_index_enabled = extension_data_cf
            .get(&ExtensionsDataKey::AccountChangeIndexEnabled)
            .map(|bytes| scrypto_decode::<bool>(&bytes).unwrap());
        let local_transaction_execution_index_enabled = extension_data_cf
            .get(&ExtensionsDataKey::LocalTransactionExecutionIndexEnabled)
            .map(|bytes| scrypto_decode::<bool>(&bytes).unwrap());
        DatabaseConfigState {
            account_change_index_enabled,
            local_transaction_execution_index_enabled,
        }
    }

    fn write_config(&self) {
        let db_context = self.open_rw_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        extension_data_cf.put(
            &ExtensionsDataKey::AccountChangeIndexEnabled,
            &scrypto_encode(&self.config.enable_account_change_index).unwrap(),
        );
        extension_data_cf.put(
            &ExtensionsDataKey::LocalTransactionExecutionIndexEnabled,
            &scrypto_encode(&self.config.enable_local_transaction_execution_index).unwrap(),
        );
        // Note: the remaining `DatabaseConfig::enable_historical_substate_values` is recorded under
        // `ExtensionsDataKey::StateTreeAssociatedValuesStatus` by the "initialize values"
        // logic, after populating the actual values - so that it correctly handles unexpected
        // Node's restarts.
    }
}

impl<R: WriteableRocks> StateManagerDatabase<R> {
    /// Ensures that the database structures related to historical Substate values are initialized
    /// properly, according to the database configuration.
    ///
    /// Most notably: if the historical state feature becomes enabled, this method runs the
    /// [`Self::populate_state_tree_associated_substate_values()`] initialization and records its
    /// success afterwards. With this approach, the lengthy backfill is tolerant to the Node's
    /// restarts (i.e. it will simply be re-run).
    fn ensure_historical_substate_values(&self) {
        let db_context = self.open_rw_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        let status = extension_data_cf
            .get(&ExtensionsDataKey::StateTreeAssociatedValuesStatus)
            .map(|bytes| {
                scrypto_decode_with_nice_error::<VersionedStateTreeAssociatedValuesStatus>(&bytes)
                    .unwrap()
                    .fully_update_and_into_latest_version()
            });

        if self.config.enable_historical_substate_values {
            if let Some(status) = status {
                info!("Historical Substate values enabled since {:?}", status);
            } else {
                let current_version = self.max_state_version();
                info!(
                    "Enabling historical Substate values at {:?}",
                    current_version
                );
                self.populate_state_tree_associated_substate_values(current_version);
                let status = StateTreeAssociatedValuesStatusV1 {
                    historical_substate_values_available_from: current_version,
                };
                extension_data_cf.put(
                    &ExtensionsDataKey::StateTreeAssociatedValuesStatus,
                    &scrypto_encode(&VersionedStateTreeAssociatedValuesStatus::from(status))
                        .unwrap(),
                );
            }
        } else {
            if let Some(status) = status {
                info!(
                    "Disabling historical Substate values (were available from {:?})",
                    status.historical_substate_values_available_from
                );
                extension_data_cf.delete(&ExtensionsDataKey::StateTreeAssociatedValuesStatus);
            } else {
                info!("Historical Substate values remain disabled");
            }
            // The line below wipes the entire historical values table, which may rise questions:
            //
            // - Why do we even need to wipe it?
            //   In theory, the associated values could be automatically, gradually deleted by
            //   the GC process (by simply catching up to the current state version). However, the
            //   GC is not "free" (i.e. it performs no-op delete operations), so we prefer to
            //   actually skip it if the history feature is disabled. Thus, we also have to clear
            //   the leftovers when we disable the history here.
            //
            // - So could we only wipe when we actually switch from "enabled" to "disabled"?
            //   If we only considered happy-paths - yes. But we also want to handle the situation
            //   where the backfill (i.e. `populate_state_tree_associated_substate_values()`) is
            //   interrupted, and then the Node is restarted with the history disabled. In such
            //   case, the history was never really enabled (since the backfill did not finish!),
            //   so it remains disabled, and yet we have that backfill's partial results persisted
            //   in the DB (unreachable, yet never GCed). It is cheap enough to simply ensure that
            //   this table is empty on every history-disabled boot-up.
            db_context.cf(AssociatedStateTreeValuesCf).delete_all();
        }
    }

    /// Traverses the entire state hash tree at the given version (which necessarily must be the
    /// current version) and populates [`AssociatedStateTreeValuesCf`] for all the Substate
    /// leaf keys, using values from the [`SubstateDatabase`].
    ///
    /// The writing is implemented in byte-size-driven batches (since Substates' sizes vary a lot).
    fn populate_state_tree_associated_substate_values(&self, current_version: StateVersion) {
        const SUBSTATE_BATCH_BYTE_SIZE: usize = 50 * 1024 * 1024; // arbitrary 50 MB work chunks

        let db_context = self.open_rw_context();
        let associated_values_cf = db_context.cf(AssociatedStateTreeValuesCf);
        let substate_database = StateTreeBasedSubstateDatabase::new(self, current_version);
        let substate_leaf_keys = substate_database.iter_substate_leaf_keys();
        for (tree_node_key, (partition_key, sort_key)) in substate_leaf_keys {
            let value = self
                .get_raw_substate_by_db_key(&partition_key, &sort_key)
                .expect("substate value referenced by hash tree does not exist");
            associated_values_cf.put(&tree_node_key, &value);
            if db_context.buffered_data_size() >= SUBSTATE_BATCH_BYTE_SIZE {
                db_context.flush();
                info!(
                    "Populated historical values up to tree node key {} (Substate key {:?}:{:?})",
                    tree_node_key.nibble_path(),
                    SpreadPrefixKeyMapper::from_db_partition_key(&partition_key),
                    hex::encode(&sort_key.0),
                );
            }
        }
        info!("Finished capturing all current Substate values as historical");
    }
}

impl<R: ReadableRocks> ConfigurableDatabase for StateManagerDatabase<R> {
    fn is_account_change_index_enabled(&self) -> bool {
        self.config.enable_account_change_index
    }

    fn is_local_transaction_execution_index_enabled(&self) -> bool {
        self.config.enable_local_transaction_execution_index
    }

    fn are_entity_listing_indices_enabled(&self) -> bool {
        self.config.enable_entity_listing_indices
    }

    fn is_state_history_enabled(&self) -> bool {
        self.config.enable_historical_substate_values
    }

    fn get_first_stored_historical_state_version(&self) -> StateVersion {
        self.open_read_context()
            .cf(ExtensionsDataCf)
            .get(&ExtensionsDataKey::StateTreeAssociatedValuesStatus)
            .map(|bytes| {
                scrypto_decode_with_nice_error::<VersionedStateTreeAssociatedValuesStatus>(&bytes)
                    .unwrap()
                    .fully_update_and_into_latest_version()
            })
            .expect("state history feature metadata not found")
            .historical_substate_values_available_from
    }
}

impl MeasurableDatabase for ActualStateManagerDatabase {
    fn get_data_volume_statistics(&self) -> Vec<CategoryDbVolumeStatistic> {
        let mut statistics = ALL_STATE_MANAGER_COLUMN_FAMILIES
            .iter()
            .map(|cf_name| {
                (
                    cf_name.to_string(),
                    CategoryDbVolumeStatistic::zero(cf_name.to_string()),
                )
            })
            .collect::<IndexMap<_, _>>();
        let live_files = self.rocks.db.live_files().unwrap_or_else(|err| {
            warn!("could not get DB live files; returning 0: {:?}", err);
            Vec::new()
        });

        for live_file in live_files {
            let Some(statistic) = statistics.get_mut(&live_file.column_family_name) else {
                warn!("LiveFile of unknown column family: {:?}", live_file);
                continue;
            };
            statistic.add_sst_summary(
                live_file.num_entries,
                live_file.num_deletions,
                live_file.size,
                live_file.level,
            );
        }
        statistics.into_values().collect()
    }

    fn count_entries(&self, category_name: &str) -> usize {
        self.rocks
            .iterator_cf(self.rocks.cf_handle(category_name), IteratorMode::Start)
            .count()
    }
}

impl<R: WriteableRocks> CommitStore for StateManagerDatabase<R> {
    fn commit(&self, commit_bundle: CommitBundle) {
        let db_context = self.open_rw_context();

        // Check for duplicate intent/payload hashes in the commit request
        let mut user_transactions_count = 0;
        let mut processed_intent_hashes = HashSet::new();
        let transactions_count = commit_bundle.transactions.len();
        let mut processed_ledger_transaction_hashes = HashSet::new();

        let commit_ledger_header = &commit_bundle.proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;

        for transaction_bundle in commit_bundle.transactions {
            let hashes = &transaction_bundle.identifiers.transaction_hashes;
            if let Some(user_hashes) = hashes.as_user() {
                processed_intent_hashes.insert(user_hashes.transaction_intent_hash);
                user_transactions_count += 1;
            }
            processed_ledger_transaction_hashes.insert(hashes.ledger_transaction_hash);
            self.add_transaction_to_write_batch(&db_context, transaction_bundle);
        }

        if processed_intent_hashes.len() != user_transactions_count {
            panic!("Commit request contains duplicate intent hashes");
        }

        if processed_ledger_transaction_hashes.len() != transactions_count {
            panic!("Commit request contains duplicate ledger transaction hashes");
        }

        db_context
            .cf(LedgerProofsCf)
            .put(&commit_state_version, &commit_bundle.proof);

        if let Some(next_epoch) = &commit_ledger_header.next_epoch {
            db_context
                .cf(EpochLedgerProofsCf)
                .put(&next_epoch.epoch, &commit_bundle.proof);
        }

        if commit_ledger_header.next_protocol_version.is_some() {
            db_context
                .cf(ProtocolUpdateInitLedgerProofsCf)
                .put(&commit_state_version, &commit_bundle.proof);
        }

        if let LedgerProofOrigin::ProtocolUpdate { .. } = &commit_bundle.proof.origin {
            db_context
                .cf(ProtocolUpdateExecutionLedgerProofsCf)
                .put(&commit_state_version, &commit_bundle.proof);
        }

        let substates_cf = db_context.cf(SubstatesCf);
        for (node_key, node_updates) in &commit_bundle.substate_store_update.updates.node_updates {
            for (partition_num, partition_updates) in &node_updates.partition_updates {
                let partition_key = DbPartitionKey {
                    node_key: node_key.clone(),
                    partition_num: *partition_num,
                };
                match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        for (sort_key, update) in substate_updates {
                            let substate_key = (partition_key.clone(), sort_key.clone());
                            match update {
                                DatabaseUpdate::Set(substate_value) => {
                                    substates_cf.put(&substate_key, substate_value);
                                }
                                DatabaseUpdate::Delete => {
                                    substates_cf.delete(&substate_key);
                                }
                            }
                        }
                    }
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    } => {
                        substates_cf.delete_group(&partition_key);
                        for (sort_key, value) in new_substate_values {
                            substates_cf.put(&(partition_key.clone(), sort_key.clone()), value);
                        }
                    }
                }
            }
        }

        if let Some(vertex_store) = commit_bundle.vertex_store {
            db_context.cf(VertexStoreCf).put(&(), &vertex_store);
        }

        let state_tree_update = commit_bundle.state_tree_update;
        for (key, node) in state_tree_update.new_nodes {
            db_context.cf(StateTreeNodesCf).put(&key, &node);
        }
        for (version, stale_parts) in state_tree_update.stale_tree_parts_at_state_version {
            db_context
                .cf(StaleStateTreePartsCf)
                .put(&version, &stale_parts);
        }

        for (node_ids, record) in commit_bundle.new_substate_node_ancestry_records {
            for node_id in node_ids {
                db_context
                    .cf(SubstateNodeAncestryRecordsCf)
                    .put(&node_id, &record);
            }
        }

        if self.config.enable_historical_substate_values {
            let associated_values_cf = db_context.cf(AssociatedStateTreeValuesCf);
            for association in commit_bundle.new_leaf_substate_keys {
                associated_values_cf.put(&association.tree_node_key, &association.substate_value);
            }
        }

        db_context
            .cf(TransactionAccuTreeSlicesCf)
            .put(&commit_state_version, &commit_bundle.transaction_tree_slice);
        db_context
            .cf(ReceiptAccuTreeSlicesCf)
            .put(&commit_state_version, &commit_bundle.receipt_tree_slice);
    }
}

impl<R: WriteableRocks> StateManagerDatabase<R> {
    fn add_transaction_to_write_batch(
        &self,
        db_context: &TypedDbContext<R, BufferedWriteSupport<R>>,
        transaction_bundle: CommittedTransactionBundle,
    ) {
        if self.is_account_change_index_enabled() {
            self.batch_update_account_change_index_from_committed_transaction(
                db_context,
                &transaction_bundle,
            );
        }

        if self.config.enable_entity_listing_indices {
            self.batch_update_entity_listing_indices(
                db_context,
                transaction_bundle.state_version,
                &transaction_bundle
                    .receipt
                    .on_ledger
                    .state_changes
                    .substate_level_changes,
            );
        }

        let CommittedTransactionBundle {
            state_version,
            raw,
            receipt,
            identifiers,
        } = transaction_bundle;
        let ledger_transaction_hash = identifiers.transaction_hashes.ledger_transaction_hash;

        // TEMPORARY until this is handled in the engine: we store both an intent lookup and the transaction itself
        if let Some(UserTransactionHashes {
            transaction_intent_hash,
            notarized_transaction_hash,
            ..
        }) = identifiers.transaction_hashes.as_user().as_ref()
        {
            /* For user transactions we only need to check for duplicate intent hashes to know
            that user payload hash and ledger payload hash are also unique. */

            let maybe_existing_state_version =
                db_context.cf(IntentHashesCf).get(transaction_intent_hash);
            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save intent hash {:?} which already exists at state version {:?}",
                    transaction_intent_hash, existing_state_version
                );
            }

            db_context
                .cf(IntentHashesCf)
                .put(transaction_intent_hash, &state_version);
            db_context
                .cf(NotarizedTransactionHashesCf)
                .put(notarized_transaction_hash, &state_version);
        } else {
            let maybe_existing_state_version = db_context
                .cf(LedgerTransactionHashesCf)
                .get(&ledger_transaction_hash);
            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save ledger transaction hash {:?} which already exists at state version {:?}",
                    ledger_transaction_hash,
                    existing_state_version
                );
            }
        }

        db_context
            .cf(LedgerTransactionHashesCf)
            .put(&ledger_transaction_hash, &state_version);
        db_context
            .cf(RawLedgerTransactionsCf)
            .put(&state_version, &raw);
        db_context
            .cf(CommittedTransactionIdentifiersCf)
            .put(&state_version, &identifiers);
        db_context
            .cf(TransactionReceiptsCf)
            .put(&state_version, &receipt.on_ledger);

        if self.is_local_transaction_execution_index_enabled() {
            db_context
                .cf(LocalTransactionExecutionsCf)
                .put(&state_version, &receipt.local_execution);
        }
    }
}

impl<R: WriteableRocks> ExecutedScenarioStore for StateManagerDatabase<R> {
    fn put_next_scenario(&self, scenario: ExecutedScenario) {
        let db_context = self.open_rw_context();
        let scenarios_cf = db_context.cf(ExecutedScenariosCf);
        let next_sequence_number = scenarios_cf
            .get_last_key()
            .map(|last_number| last_number.checked_add(1).expect("cannot auto-increment"))
            .unwrap_or_default();
        scenarios_cf.put(&next_sequence_number, &scenario);
    }

    fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedScenario)> {
        self.open_read_context()
            .cf(ExecutedScenariosCf)
            .iterate(Direction::Forward)
            .collect()
    }
}

pub struct RocksDBCommittedTransactionBundleIterator<'r> {
    state_version: StateVersion,
    txns_iter: Box<dyn Iterator<Item = (StateVersion, RawLedgerTransaction)> + 'r>,
    ledger_receipts_iter: Box<dyn Iterator<Item = (StateVersion, LedgerTransactionReceipt)> + 'r>,
    local_executions_iter: Box<dyn Iterator<Item = (StateVersion, LocalTransactionExecution)> + 'r>,
    identifiers_iter:
        Box<dyn Iterator<Item = (StateVersion, CommittedTransactionIdentifiers)> + 'r>,
}

impl<'r> RocksDBCommittedTransactionBundleIterator<'r> {
    fn new<R: ReadableRocks, W: WriteSupport>(
        from_state_version: StateVersion,
        db_context: TypedDbContext<'r, R, W>,
    ) -> Self {
        Self {
            state_version: from_state_version,
            txns_iter: db_context
                .cf(RawLedgerTransactionsCf)
                .iterate_from(&from_state_version, Direction::Forward),
            ledger_receipts_iter: db_context
                .cf(TransactionReceiptsCf)
                .iterate_from(&from_state_version, Direction::Forward),
            local_executions_iter: db_context
                .cf(LocalTransactionExecutionsCf)
                .iterate_from(&from_state_version, Direction::Forward),
            identifiers_iter: db_context
                .cf(CommittedTransactionIdentifiersCf)
                .iterate_from(&from_state_version, Direction::Forward),
        }
    }
}

impl<'r> Iterator for RocksDBCommittedTransactionBundleIterator<'r> {
    type Item = CommittedTransactionBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let (txn_version, txn) = self.txns_iter.next()?;

        let (ledger_receipt_version, ledger_receipt) = self
            .ledger_receipts_iter
            .next()
            .expect("missing ledger receipt");
        let (local_execution_version, local_execution) = self
            .local_executions_iter
            .next()
            .expect("missing local transaction execution");
        let (identifiers_version, identifiers) = self
            .identifiers_iter
            .next()
            .expect("missing transaction identifiers");

        let current_state_version = self.state_version;
        for (other_row_description, other_row_version) in [
            ("transaction version", txn_version),
            ("ledger receipt version", ledger_receipt_version),
            ("local execution version", local_execution_version),
            ("identifiers version", identifiers_version),
        ] {
            if other_row_version != current_state_version {
                panic!("DB inconsistency! {other_row_description} ({other_row_version}) doesn't match expected state version ({current_state_version})");
            }
        }

        self.state_version = self
            .state_version
            .next()
            .expect("Invalid next state version!");

        Some(CommittedTransactionBundle {
            state_version: current_state_version,
            raw: txn,
            receipt: LocalTransactionReceipt {
                on_ledger: ledger_receipt,
                local_execution,
            },
            identifiers,
        })
    }
}

impl<R: ReadableRocks> IterableTransactionStore for StateManagerDatabase<R> {
    fn get_committed_transaction_bundle_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = CommittedTransactionBundle> + '_> {
        // This should not happen. This interface should be used after checking (e.g. `core-api-server/src/core-api/handlers/`).
        // However, with or without this debug_assert there would still be a panic if LocalTransactionExecution is missing.
        debug_assert!(self.is_local_transaction_execution_index_enabled());

        Box::new(RocksDBCommittedTransactionBundleIterator::new(
            from_state_version,
            self.open_read_context(),
        ))
    }
}

impl<R: ReadableRocks> QueryableTransactionStore for StateManagerDatabase<R> {
    fn get_committed_transaction(
        &self,
        state_version: StateVersion,
    ) -> Option<RawLedgerTransaction> {
        self.open_read_context()
            .cf(RawLedgerTransactionsCf)
            .get(&state_version)
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: StateVersion,
    ) -> Option<CommittedTransactionIdentifiers> {
        self.open_read_context()
            .cf(CommittedTransactionIdentifiersCf)
            .get(&state_version)
    }

    fn get_committed_ledger_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LedgerTransactionReceipt> {
        self.open_read_context()
            .cf(TransactionReceiptsCf)
            .get(&state_version)
    }

    fn get_committed_local_transaction_execution(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionExecution> {
        self.open_read_context()
            .cf(LocalTransactionExecutionsCf)
            .get(&state_version)
    }

    fn get_committed_local_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionReceipt> {
        let ledger_transaction_receipt =
            self.get_committed_ledger_transaction_receipt(state_version);
        let local_transaction_execution =
            self.get_committed_local_transaction_execution(state_version);
        match (ledger_transaction_receipt, local_transaction_execution) {
            (Some(on_ledger), Some(local_execution)) => Some(LocalTransactionReceipt {
                on_ledger,
                local_execution,
            }),
            (None, Some(_)) => panic!("missing ledger receipt at state version {state_version}"),
            (Some(_), None) => {
                if self.is_local_transaction_execution_index_enabled() {
                    panic!("missing local execution at state version {state_version}")
                }
                None
            }
            (None, None) => None,
        }
    }
}

impl<R: ReadableRocks> TransactionIndex<&TransactionIntentHash> for StateManagerDatabase<R> {
    fn get_txn_state_version_by_identifier(
        &self,
        intent_hash: &TransactionIntentHash,
    ) -> Option<StateVersion> {
        self.open_read_context().cf(IntentHashesCf).get(intent_hash)
    }
}

impl<R: ReadableRocks> TransactionIndex<&NotarizedTransactionHash> for StateManagerDatabase<R> {
    fn get_txn_state_version_by_identifier(
        &self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<StateVersion> {
        self.open_read_context()
            .cf(NotarizedTransactionHashesCf)
            .get(notarized_transaction_hash)
    }
}

impl<R: ReadableRocks> TransactionIndex<&LedgerTransactionHash> for StateManagerDatabase<R> {
    fn get_txn_state_version_by_identifier(
        &self,
        ledger_transaction_hash: &LedgerTransactionHash,
    ) -> Option<StateVersion> {
        self.open_read_context()
            .cf(LedgerTransactionHashesCf)
            .get(ledger_transaction_hash)
    }
}

impl<R: ReadableRocks> TransactionIdentifierLoader for StateManagerDatabase<R> {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)> {
        self.open_read_context()
            .cf(CommittedTransactionIdentifiersCf)
            .get_last()
    }
}

impl<R: ReadableRocks> IterableProofStore for StateManagerDatabase<R> {
    fn get_proof_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = LedgerProof> + '_> {
        Box::new(
            self.open_read_context()
                .cf(LedgerProofsCf)
                .iterate_from(&from_state_version, Direction::Forward)
                .map(|(_, proof)| proof),
        )
    }

    fn get_next_epoch_proof_iter(
        &self,
        from_epoch: Epoch,
    ) -> Box<dyn Iterator<Item = LedgerProof> + '_> {
        Box::new(
            self.open_read_context()
                .cf(EpochLedgerProofsCf)
                .iterate_from(&from_epoch, Direction::Forward)
                .map(|(_, proof)| proof),
        )
    }

    fn get_protocol_update_init_proof_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = LedgerProof> + '_> {
        Box::new(
            self.open_read_context()
                .cf(ProtocolUpdateInitLedgerProofsCf)
                .iterate_from(&from_state_version, Direction::Forward)
                .map(|(_, proof)| proof),
        )
    }

    fn get_protocol_update_execution_proof_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = LedgerProof> + '_> {
        Box::new(
            self.open_read_context()
                .cf(ProtocolUpdateExecutionLedgerProofsCf)
                .iterate_from(&from_state_version, Direction::Forward)
                .map(|(_, proof)| proof),
        )
    }
}

impl<R: ReadableRocks> QueryableProofStore for StateManagerDatabase<R> {
    fn max_state_version(&self) -> StateVersion {
        self.open_read_context()
            .cf(RawLedgerTransactionsCf)
            .get_last_key()
            .unwrap_or(StateVersion::pre_genesis())
    }

    fn get_syncable_txns_and_proof(
        &self,
        start_state_version_inclusive: StateVersion,
        max_number_of_txns_if_more_than_one_proof: u32,
        max_payload_size_in_bytes: u32,
    ) -> Result<TxnsAndProof, GetSyncableTxnsAndProofError> {
        let mut payload_size_so_far = 0;
        let mut latest_usable_proof: Option<LedgerProof> = None;
        let mut txns = Vec::new();

        let mut proofs_iter = self
            .open_read_context()
            .cf(LedgerProofsCf)
            .iterate_from(&start_state_version_inclusive, Direction::Forward);
        let mut txns_iter = self
            .open_read_context()
            .cf(RawLedgerTransactionsCf)
            .iterate_from(&start_state_version_inclusive, Direction::Forward);

        // A few flags used to be able to provide an accurate error response
        let mut encountered_protocol_update_proof = None;
        let mut any_consensus_proof_iterated = false;

        'proof_loop: while payload_size_so_far <= max_payload_size_in_bytes
            && txns.len() <= (max_number_of_txns_if_more_than_one_proof as usize)
        {
            // Fetch next proof and see if all txns it includes can fit
            // If they do - add them to the output and update the latest usable proof then continue the iteration
            // If they don't - (sadly) ignore this proof's txns read so far and break the loop
            // If we're out of proofs (or some txns are missing): also break the loop
            match proofs_iter.next() {
                Some((next_proof_state_version, next_proof)) => {
                    // We're not serving any genesis or protocol update transactions.
                    // All nodes should have them hardcoded/configured/generated locally.
                    // Stop iterating the proofs and return whatever txns/proof we have
                    // collected so far (or an empty response).
                    match next_proof.origin {
                        LedgerProofOrigin::ProtocolUpdate { .. } => {
                            encountered_protocol_update_proof = Some(next_proof);
                            break 'proof_loop;
                        }
                        LedgerProofOrigin::Consensus { .. } => {
                            any_consensus_proof_iterated = true;
                        }
                    }

                    let mut payload_size_including_next_proof_txns = payload_size_so_far;
                    let mut next_proof_txns = Vec::new();

                    // It looks convoluted, but really isn't :D
                    // * max_payload_size_in_bytes limit is always enforced
                    // * max_number_of_txns_if_more_than_one_proof limit is skipped
                    //   if there isn't yet any usable proof (so the response may
                    //   contain more than max_number_of_txns_if_more_than_one_proof txns
                    //   if that's what it takes to be able to produce a response at all)
                    'proof_txns_loop: while payload_size_including_next_proof_txns
                        <= max_payload_size_in_bytes
                        && (latest_usable_proof.is_none()
                            || txns.len() + next_proof_txns.len()
                                <= (max_number_of_txns_if_more_than_one_proof as usize))
                    {
                        match txns_iter.next() {
                            Some((next_txn_state_version, next_txn)) => {
                                payload_size_including_next_proof_txns += next_txn.len() as u32;
                                next_proof_txns.push(next_txn);

                                if next_txn_state_version == next_proof_state_version {
                                    // We've reached the last txn under next_proof
                                    break 'proof_txns_loop;
                                }
                            }
                            None => {
                                // A txn must be missing! Log an error as this indicates DB corruption
                                error!("The DB is missing transactions! There is a proof at state version {} but only got {} txns (starting from state version {} inclusive)",
                                    next_proof_state_version, (txns.len() + next_proof_txns.len()), start_state_version_inclusive);
                                // We can still serve a response (return whatever txns/proof we've collected so far)
                                break 'proof_loop;
                            }
                        }
                    }

                    // All txns under next_proof have been processed, once again confirm
                    // that they can all fit in the response (the last txn could have crossed the limit)
                    if payload_size_including_next_proof_txns <= max_payload_size_in_bytes
                        && (latest_usable_proof.is_none()
                            || txns.len() + next_proof_txns.len()
                                <= (max_number_of_txns_if_more_than_one_proof as usize))
                    {
                        // Yup, all good, use next_proof as the result and add its txns
                        let next_proof_is_a_protocol_update =
                            next_proof.ledger_header.next_protocol_version.is_some();
                        let next_proof_is_an_epoch_change =
                            next_proof.ledger_header.next_epoch.is_some();
                        latest_usable_proof = Some(next_proof);
                        txns.append(&mut next_proof_txns);
                        payload_size_so_far = payload_size_including_next_proof_txns;

                        if next_proof_is_a_protocol_update || next_proof_is_an_epoch_change {
                            // Stop if we've reached a protocol update or end of epoch
                            break 'proof_loop;
                        }
                    } else {
                        // We couldn't fit next proof's txns so there's no point in further iteration
                        break 'proof_loop;
                    }
                }
                None => {
                    // No more proofs
                    break 'proof_loop;
                }
            }
        }

        latest_usable_proof
            .map(|proof| TxnsAndProof { txns, proof })
            .ok_or(if any_consensus_proof_iterated {
                // We have iterated at least one valid consensus proof
                // but still were unable to produce a response,
                // so this must have been a limit issue.
                GetSyncableTxnsAndProofError::FailedToPrepareAResponseWithinLimits
            } else {
                // We have not iterated any valid consensus proof.
                // Check if we've broken due to encountering
                // one of the non-Consensus originated proofs.
                if let Some(protocol_update_proof) = encountered_protocol_update_proof {
                    GetSyncableTxnsAndProofError::RefusedToServeProtocolUpdate {
                        refused_proof: Box::new(protocol_update_proof),
                    }
                } else {
                    // We have not iterated any Consensus proof
                    // or any other proof.
                    // So the request must have been ahead of our current ledger.
                    GetSyncableTxnsAndProofError::NothingToServeAtTheGivenStateVersion
                }
            })
    }

    fn get_first_proof(&self) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(LedgerProofsCf)
            .get_first_value()
    }

    fn get_proof(&self, state_version: StateVersion) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(LedgerProofsCf)
            .get(&state_version)
    }

    fn get_post_genesis_epoch_proof(&self) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(EpochLedgerProofsCf)
            .get_first_value()
    }

    fn get_epoch_proof(&self, epoch: Epoch) -> Option<LedgerProof> {
        self.open_read_context().cf(EpochLedgerProofsCf).get(&epoch)
    }

    fn get_latest_proof(&self) -> Option<LedgerProof> {
        self.open_read_context().cf(LedgerProofsCf).get_last_value()
    }

    fn get_latest_epoch_proof(&self) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(EpochLedgerProofsCf)
            .get_last_value()
    }

    fn get_closest_epoch_proof_on_or_before(
        &self,
        state_version: StateVersion,
    ) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(LedgerProofsCf)
            .iterate_from(&state_version, Direction::Reverse)
            .map(|(_, proof)| proof)
            .find(|proof| proof.ledger_header.next_epoch.is_some())
    }

    fn get_latest_protocol_update_init_proof(&self) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(ProtocolUpdateInitLedgerProofsCf)
            .get_last_value()
    }

    fn get_latest_protocol_update_execution_proof(&self) -> Option<LedgerProof> {
        self.open_read_context()
            .cf(ProtocolUpdateExecutionLedgerProofsCf)
            .get_last_value()
    }
}

impl<R: CheckpointableRocks> StateManagerDatabase<R> {
    /// Creates a checkpoint in `path`
    pub fn create_checkpoint(&self, path: String) -> Result<(), String> {
        self.rocks
            .create_checkpoint(PathBuf::from(path))
            .map_err(|err| err.to_string())
    }
}

impl<R: ReadableRocks> SubstateDatabase for StateManagerDatabase<R> {
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.open_read_context()
            .cf(SubstatesCf)
            .get(&(partition_key.clone(), sort_key.clone()))
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        let from_sort_key = from_sort_key.cloned().unwrap_or(DbSortKey(vec![]));
        Box::new(
            self.open_read_context()
                .cf(SubstatesCf)
                .iterate_group_from(&(partition_key.clone(), from_sort_key), Direction::Forward)
                .map(|((_, sort_key), value)| (sort_key, value)),
        )
    }
}

impl<R: ReadableRocks> ListableSubstateDatabase for StateManagerDatabase<R> {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        self.open_read_context()
            .cf(SubstatesCf)
            .iterate_key_groups()
    }
}

impl<R: ReadableRocks> SubstateNodeAncestryStore for StateManagerDatabase<R> {
    fn batch_get_ancestry<'a>(
        &self,
        node_ids: impl IntoIterator<Item = &'a NodeId>,
    ) -> Vec<Option<SubstateNodeAncestryRecord>> {
        self.open_read_context()
            .cf(SubstateNodeAncestryRecordsCf)
            .get_many(Vec::from_iter(node_ids))
    }
}

impl<R: ReadableRocks> ReadableTreeStore for StateManagerDatabase<R> {
    fn get_node(&self, key: &StoredTreeNodeKey) -> Option<TreeNode> {
        self.open_read_context().cf(StateTreeNodesCf).get(key)
    }
}

impl<R: WriteableRocks> StateTreeGcStore for StateManagerDatabase<R> {
    fn get_stale_tree_parts_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (StateVersion, StaleTreeParts)> + '_> {
        self.open_read_context()
            .cf(StaleStateTreePartsCf)
            .iterate(Direction::Forward)
    }

    fn progress_historical_substate_values_availability(&self, available_from: StateVersion) {
        let db_context = self.open_rw_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        let status = extension_data_cf
            .get(&ExtensionsDataKey::StateTreeAssociatedValuesStatus)
            .map(|bytes| {
                scrypto_decode_with_nice_error::<VersionedStateTreeAssociatedValuesStatus>(&bytes)
                    .unwrap()
                    .fully_update_and_into_latest_version()
            });
        let Some(mut status) = status else {
            // The state history feature is simply not enabled.
            return;
        };
        if available_from <= status.historical_substate_values_available_from {
            // The state history feature was enabled after this state version.
            return;
        }
        status.historical_substate_values_available_from = available_from;
        extension_data_cf.put(
            &ExtensionsDataKey::StateTreeAssociatedValuesStatus,
            &scrypto_encode(&VersionedStateTreeAssociatedValuesStatus::from(status)).unwrap(),
        );
    }

    fn batch_delete_node<'a>(&self, keys: impl IntoIterator<Item = &'a StoredTreeNodeKey>) {
        let db_context = self.open_rw_context();
        let tree_nodes_cf = db_context.cf(StateTreeNodesCf);
        let associated_values_cf = db_context.cf(AssociatedStateTreeValuesCf);
        for key in keys {
            tree_nodes_cf.delete(key);
            if self.config.enable_historical_substate_values {
                // Note: not every key represents a Substate. But majority does, so we simply accept
                // some fraction of no-op deletes here, in the name of simplicity.
                associated_values_cf.delete(key);
            }
        }
    }

    fn delete_stale_tree_parts_up_to_version(&self, to_state_version: StateVersion) {
        self.open_rw_context()
            .cf(StaleStateTreePartsCf)
            .delete_range(&StateVersion::pre_genesis(), &to_state_version);
    }
}

impl<R: WriteableRocks> LedgerProofsGcStore for StateManagerDatabase<R> {
    fn get_progress(&self) -> Option<LedgerProofsGcProgress> {
        self.open_read_context()
            .cf(LedgerProofsGcProgressCf)
            .get(&())
    }

    fn set_progress(&self, progress: LedgerProofsGcProgress) {
        self.open_rw_context()
            .cf(LedgerProofsGcProgressCf)
            .put(&(), &progress);
    }

    fn delete_ledger_proofs_range(&self, from: StateVersion, to: StateVersion) {
        self.open_rw_context()
            .cf(LedgerProofsCf)
            .delete_range(&from, &to);
    }
}

impl<R: ReadableRocks> ReadableAccuTreeStore<StateVersion, TransactionTreeHash>
    for StateManagerDatabase<R>
{
    fn get_tree_slice(
        &self,
        state_version: &StateVersion,
    ) -> Option<TreeSlice<TransactionTreeHash>> {
        self.open_read_context()
            .cf(TransactionAccuTreeSlicesCf)
            .get(state_version)
            .map(|slice| slice.0)
    }
}

impl<R: ReadableRocks> ReadableAccuTreeStore<StateVersion, ReceiptTreeHash>
    for StateManagerDatabase<R>
{
    fn get_tree_slice(&self, state_version: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.open_read_context()
            .cf(ReceiptAccuTreeSlicesCf)
            .get(state_version)
            .map(|slice| slice.0)
    }
}

impl<R: WriteableRocks> WriteableVertexStore for StateManagerDatabase<R> {
    fn save_vertex_store(&self, blob: VertexStoreBlob) {
        self.open_rw_context().cf(VertexStoreCf).put(&(), &blob)
    }
}

impl<R: ReadableRocks> RecoverableVertexStore for StateManagerDatabase<R> {
    fn get_vertex_store(&self) -> Option<VertexStoreBlob> {
        self.open_read_context().cf(VertexStoreCf).get(&())
    }
}

impl<R: WriteableRocks> StateManagerDatabase<R> {
    fn batch_update_account_change_index_from_receipt(
        &self,
        db_context: &TypedDbContext<R, BufferedWriteSupport<R>>,
        state_version: StateVersion,
        execution: &LocalTransactionExecution,
    ) {
        for address in execution
            .global_balance_summary
            .global_balance_changes
            .keys()
            .filter(|address| address.is_account())
        {
            db_context
                .cf(AccountChangeStateVersionsCf)
                .put(&(*address, state_version), &());
        }
    }

    fn batch_update_account_change_index_from_committed_transaction(
        &self,
        db_context: &TypedDbContext<R, BufferedWriteSupport<R>>,
        transaction_bundle: &CommittedTransactionBundle,
    ) {
        let state_version = transaction_bundle.state_version;
        self.batch_update_account_change_index_from_receipt(
            db_context,
            state_version,
            &transaction_bundle.receipt.local_execution,
        );

        db_context.cf(ExtensionsDataCf).put(
            &ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            &state_version.to_be_bytes().to_vec(),
        );
    }

    fn update_account_change_index_from_store(
        &self,
        start_state_version_inclusive: StateVersion,
        limit: u64,
    ) -> StateVersion {
        let db_context = self.open_rw_context();
        let mut executions_iter = db_context
            .cf(LocalTransactionExecutionsCf)
            .iterate_from(&start_state_version_inclusive, Direction::Forward);

        let mut last_state_version = start_state_version_inclusive;
        let mut index = 0;
        while index < limit {
            match executions_iter.next() {
                Some((next_execution_state_version, next_execution)) => {
                    let expected_state_version = start_state_version_inclusive
                        .relative(index)
                        .expect("Invalid relative state version!");
                    if expected_state_version != next_execution_state_version {
                        panic!("DB inconsistency! Missing local transaction execution at state version {expected_state_version}");
                    }
                    last_state_version = expected_state_version;
                    self.batch_update_account_change_index_from_receipt(
                        &db_context,
                        last_state_version,
                        &next_execution,
                    );
                    index += 1;
                }
                None => {
                    break;
                }
            }
        }

        db_context.cf(ExtensionsDataCf).put(
            &ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            &last_state_version.to_be_bytes().to_vec(),
        );

        last_state_version
    }

    fn batch_update_entity_listing_indices(
        &self,
        db_context: &TypedDbContext<R, BufferedWriteSupport<R>>,
        state_version: StateVersion,
        substate_changes: &BySubstate<SubstateChangeAction>,
    ) {
        for (index_within_txn, node_id) in substate_changes.iter_node_ids().enumerate() {
            let type_info_change = substate_changes.get(
                node_id,
                &TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            );
            let Some(type_info_change) = type_info_change else {
                continue;
            };
            let created_type_info_value = match type_info_change {
                SubstateChangeAction::Create { new } => new,
                SubstateChangeAction::Update { .. } => {
                    // Even if TypeInfo is updated (e.g. its blueprint version bumped), the fields
                    // that we care about (package address and blueprint name) are effectively
                    // immutable - we can thus safely ignore all updates to this substate.
                    continue;
                }
                SubstateChangeAction::Delete { .. } => {
                    panic!(
                        "type info substate should not be deleted: {:?}",
                        type_info_change
                    )
                }
            };
            let type_info =
                scrypto_decode_with_nice_error::<TypeInfoSubstate>(created_type_info_value)
                    .expect("decode type info");

            let entity_type = node_id.entity_type().expect("type of upserted Entity");
            let creation_id = CreationId::new(state_version, index_within_txn);

            match type_info {
                TypeInfoSubstate::Object(object_info) => {
                    let blueprint_id = object_info.blueprint_info.blueprint_id;
                    let BlueprintId {
                        package_address,
                        blueprint_name,
                    } = blueprint_id.clone();
                    db_context.cf(TypeAndCreationIndexedEntitiesCf).put(
                        &(entity_type, creation_id.clone()),
                        &EntityBlueprintId::of_object(*node_id, blueprint_id),
                    );
                    db_context.cf(BlueprintAndCreationIndexedObjectsCf).put(
                        &(package_address, hash(&blueprint_name), creation_id),
                        &ObjectBlueprintNameV1 {
                            node_id: *node_id,
                            blueprint_name,
                        },
                    );
                }
                TypeInfoSubstate::KeyValueStore(_kv_store_info) => {
                    db_context.cf(TypeAndCreationIndexedEntitiesCf).put(
                        &(entity_type, creation_id),
                        &EntityBlueprintId::of_kv_store(*node_id),
                    );
                }
                TypeInfoSubstate::GlobalAddressReservation(_)
                | TypeInfoSubstate::GlobalAddressPhantom(_) => {
                    panic!("should not be persisted: {:?}", type_info)
                }
            }
        }
    }

    fn ensure_entity_listing_indices(&self) {
        const TXN_FLUSH_INTERVAL: u64 = 10_000;
        const PROGRESS_LOG_INTERVAL: u64 = 1_000_000;

        let db_context = self.open_rw_context();

        if !self.config.enable_entity_listing_indices {
            info!("Entity listing indices are disabled.");
            // We remove the indices' data and metadata in a single, cheap write batch:
            db_context.cf(TypeAndCreationIndexedEntitiesCf).delete_all();
            db_context
                .cf(BlueprintAndCreationIndexedObjectsCf)
                .delete_all();
            db_context
                .cf(ExtensionsDataCf)
                .delete(&ExtensionsDataKey::EntityListingIndicesLastProcessedStateVersion);
            info!("Deleted entity listing indices.");
            return;
        }

        info!("Entity listing indices are enabled.");
        let last_processed_state_version = db_context
            .cf(ExtensionsDataCf)
            .get(&ExtensionsDataKey::EntityListingIndicesLastProcessedStateVersion)
            .map(StateVersion::from_be_bytes)
            .unwrap_or(StateVersion::pre_genesis());
        let catchup_from_version = last_processed_state_version.next().expect("next version");

        let mut receipts_iter = db_context
            .cf(TransactionReceiptsCf)
            .iterate_from(&catchup_from_version, Direction::Forward)
            .peekable();

        while let Some((state_version, receipt)) = receipts_iter.next() {
            self.batch_update_entity_listing_indices(
                &db_context,
                state_version,
                &receipt.state_changes.substate_level_changes,
            );
            if state_version.number() % TXN_FLUSH_INTERVAL == 0 || receipts_iter.peek().is_none() {
                if state_version.number() % PROGRESS_LOG_INTERVAL == 0 {
                    info!("Entity listing indices updated to {}", state_version);
                }
                db_context.cf(ExtensionsDataCf).put(
                    &ExtensionsDataKey::EntityListingIndicesLastProcessedStateVersion,
                    &state_version.to_be_bytes().to_vec(),
                );
                db_context.flush();
            }
        }
        info!("Caught up Entity listing indices.");
    }
}

impl<R: WriteableRocks> AccountChangeIndexExtension for StateManagerDatabase<R> {
    fn account_change_index_last_processed_state_version(&self) -> StateVersion {
        self.open_read_context()
            .cf(ExtensionsDataCf)
            .get(&ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion)
            .map(StateVersion::from_be_bytes)
            .unwrap_or(StateVersion::pre_genesis())
    }

    fn catchup_account_change_index(&self) {
        if !self.config.enable_account_change_index {
            return; // Nothing to do
        }

        const MAX_TRANSACTION_BATCH: u64 = 16 * 1024;

        info!("Account Change Index is enabled!");

        let last_state_version = self.max_state_version();
        let mut last_processed_state_version =
            self.account_change_index_last_processed_state_version();

        if last_processed_state_version == last_state_version {
            return;
        }

        info!("Account Change Index is behind at state version {last_processed_state_version} out of {last_state_version}. Catching up ...");

        while last_processed_state_version < last_state_version {
            last_processed_state_version = self.update_account_change_index_from_store(
                last_processed_state_version
                    .next()
                    .expect("Invalid next state version!"),
                MAX_TRANSACTION_BATCH,
            );
            info!("Account Change Index updated to {last_processed_state_version}/{last_state_version}");
        }

        info!("Account Change Index catchup done!");
    }
}

impl<R: WriteableRocks> RestoreDecember2023LostSubstates for StateManagerDatabase<R> {
    fn restore_december_2023_lost_substates(&self, network: &NetworkDefinition) {
        let db_context = self.open_rw_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        let december_2023_lost_substates_restored =
            extension_data_cf.get(&ExtensionsDataKey::December2023LostSubstatesRestored);

        let should_restore_substates = if network.id == NetworkDefinition::mainnet().id {
            // For mainnet, we have a tested, working fix at an epoch learnt during investigation:

            // Skip restoration if substates already restored
            if december_2023_lost_substates_restored.is_some() {
                return;
            }

            // Substates were deleted on the transition to epoch 51817 so no need to restore
            // substates if the current epoch has not reached this epoch yet.
            self.get_latest_epoch_proof().map_or(false, |p| {
                p.ledger_header.next_epoch.unwrap().epoch.number() >= 51817
            })
        } else {
            // For other networks, we can calculate the "problem" epoch from theoretical principles:
            let (Some(first_proof), Some(latest_epoch_proof)) =
                (self.get_first_proof(), self.get_latest_epoch_proof())
            else {
                return; // empty ledger; no fix needed
            };
            let first_epoch = first_proof.ledger_header.epoch.number();
            let last_epoch = latest_epoch_proof.ledger_header.epoch.number();
            // magic number below is: (256 * 3 / 4 - 1) * 100 - 1
            let problem_at_end_of_epoch = first_epoch + 19099;
            // Due to another bug, stokenet nodes may mistakenly believe that they already applied
            // the fix. Thus, we have to ignore the `december_2023_lost_substates_restored` flag and
            // make a decision based on "being stuck in the problematic epoch range". The fix is
            // effectively idempotent, so we are fine with re-running it in an edge case.
            last_epoch >= problem_at_end_of_epoch && last_epoch <= (problem_at_end_of_epoch + 2)
        };

        if should_restore_substates {
            info!("Restoring lost substates...");
            let last_state_version = self
                .get_latest_proof()
                .map_or(StateVersion::of(1u64), |s| s.ledger_header.state_version);

            let txn_tracker_db_node_key =
                SpreadPrefixKeyMapper::to_db_node_key(TRANSACTION_TRACKER.as_node_id());

            let substates_cf = db_context.cf(SubstatesCf);

            let receipts_iter: Box<dyn Iterator<Item = (StateVersion, LedgerTransactionReceipt)>> =
                db_context
                    .cf(TransactionReceiptsCf)
                    .iterate_from(&StateVersion::of(1u64), Direction::Forward);

            for (version, receipt) in receipts_iter {
                for (substate_ref, change) in receipt.state_changes.substate_level_changes.iter() {
                    let db_partition_key =
                        SpreadPrefixKeyMapper::to_db_partition_key(&substate_ref.0, substate_ref.1);

                    // The substate was deleted if it's DbNodeKey is lexicographically greater than the DbNodeKey
                    // of the transaction tracker. So here we re-flash the substates directly into the state store.
                    if db_partition_key.node_key.gt(&txn_tracker_db_node_key) {
                        let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&substate_ref.2);
                        let substate_key = (db_partition_key.clone(), sort_key);

                        match change {
                            SubstateChangeAction::Create { new }
                            | SubstateChangeAction::Update { new, .. } => {
                                substates_cf.put(&substate_key, new);
                            }
                            SubstateChangeAction::Delete { .. } => {
                                substates_cf.delete(&substate_key);
                            }
                        }
                    }
                }

                if version.number() % 10000 == 0 {
                    db_context.flush();
                    info!(
                        "Scanned {} of {} transactions...",
                        version.number(),
                        last_state_version.number()
                    );
                }
            }

            info!("Finished restoring lost substates!");
        }

        db_context.cf(ExtensionsDataCf).put(
            &ExtensionsDataKey::December2023LostSubstatesRestored,
            &vec![],
        );
        db_context.flush();
    }
}

impl<R: ReadableRocks> IterableAccountChangeIndex for StateManagerDatabase<R> {
    fn get_state_versions_for_account_iter(
        &self,
        account: GlobalAddress,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = StateVersion> + '_> {
        Box::new(
            self.open_read_context()
                .cf(AccountChangeStateVersionsCf)
                .iterate_from(&(account, from_state_version), Direction::Forward)
                .take_while(move |((next_account, _), _)| next_account == &account)
                .map(|((_, state_version), _)| state_version),
        )
    }
}

impl<R: ReadableRocks> EntityListingIndex for StateManagerDatabase<R> {
    fn get_created_entity_iter(
        &self,
        entity_type: EntityType,
        from_creation_id: Option<&CreationId>,
    ) -> Box<dyn Iterator<Item = (CreationId, EntityBlueprintId)> + '_> {
        let from_creation_id = from_creation_id.cloned().unwrap_or_else(CreationId::zero);
        Box::new(
            self.open_read_context()
                .cf(TypeAndCreationIndexedEntitiesCf)
                .iterate_group_from(&(entity_type, from_creation_id), Direction::Forward)
                .map(|((_, creation_id), entity_blueprint_id)| (creation_id, entity_blueprint_id)),
        )
    }

    fn get_blueprint_entity_iter(
        &self,
        blueprint_id: &BlueprintId,
        from_creation_id: Option<&CreationId>,
    ) -> Box<dyn Iterator<Item = (CreationId, EntityBlueprintId)> + '_> {
        let BlueprintId {
            package_address,
            blueprint_name,
        } = blueprint_id;
        let blueprint_name_hash = hash(blueprint_name);
        let from_creation_id = from_creation_id.cloned().unwrap_or_else(CreationId::zero);
        Box::new(
            self.open_read_context()
                .cf(BlueprintAndCreationIndexedObjectsCf)
                .iterate_group_from(
                    &(*package_address, blueprint_name_hash, from_creation_id),
                    Direction::Forward,
                )
                .map(
                    |((package_address, _, creation_id), object_blueprint_name)| {
                        (
                            creation_id,
                            EntityBlueprintId::of_object(
                                object_blueprint_name.node_id,
                                BlueprintId::new(
                                    &package_address,
                                    object_blueprint_name.blueprint_name,
                                ),
                            ),
                        )
                    },
                ),
        )
    }
}

impl<R: ReadableRocks> LeafSubstateValueStore for StateManagerDatabase<R> {
    fn get_associated_value(&self, tree_node_key: &StoredTreeNodeKey) -> Option<DbSubstateValue> {
        self.open_read_context()
            .cf(AssociatedStateTreeValuesCf)
            .get(tree_node_key)
    }
}
