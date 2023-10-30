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

use std::collections::HashSet;
use std::fmt;

use crate::store::traits::*;
use crate::{
    CommittedTransactionIdentifiers, LedgerProof, LedgerTransactionReceipt,
    LocalTransactionExecution, LocalTransactionReceipt, ReceiptTreeHash, StateVersion,
    TransactionTreeHash, VersionedCommittedTransactionIdentifiers, VersionedLedgerProof,
    VersionedLedgerTransactionReceipt, VersionedLocalTransactionExecution,
};
use node_common::utils::IsAccountExt;
use radix_engine::types::*;
use radix_engine_stores::hash_tree::tree_store::{
    encode_key, NodeKey, ReadableTreeStore, TreeNode, VersionedTreeNode,
};
use rocksdb::{ColumnFamilyDescriptor, Direction, Options, DB};
use transaction::model::*;

use radix_engine_store_interface::interface::*;

use itertools::Itertools;
use std::path::PathBuf;

use tracing::{error, info, warn};

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::store::traits::gc::StateHashTreeGcStore;
use crate::store::traits::measurement::{CategoryDbVolumeStatistic, MeasurableDatabase};
use crate::store::traits::scenario::{
    ExecutedGenesisScenario, ExecutedGenesisScenarioStore, ScenarioSequenceNumber,
    VersionedExecutedGenesisScenario,
};
use crate::store::typed_cf_api::*;
use crate::transaction::{
    LedgerTransactionHash, RawLedgerTransaction, TypedTransactionIdentifiers,
};

use super::traits::extensions::*;

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
const ALL_COLUMN_FAMILIES: [&str; 19] = [
    RawLedgerTransactionsCf::DEFAULT_NAME,
    CommittedTransactionIdentifiersCf::VERSIONED_NAME,
    TransactionReceiptsCf::VERSIONED_NAME,
    LocalTransactionExecutionsCf::VERSIONED_NAME,
    IntentHashesCf::DEFAULT_NAME,
    NotarizedTransactionHashesCf::DEFAULT_NAME,
    LedgerTransactionHashesCf::DEFAULT_NAME,
    LedgerProofsCf::VERSIONED_NAME,
    EpochLedgerProofsCf::VERSIONED_NAME,
    SubstatesCf::DEFAULT_NAME,
    SubstateNodeAncestryRecordsCf::VERSIONED_NAME,
    VertexStoreCf::VERSIONED_NAME,
    StateHashTreeNodesCf::VERSIONED_NAME,
    StaleStateHashTreePartsCf::VERSIONED_NAME,
    TransactionAccuTreeSlicesCf::VERSIONED_NAME,
    ReceiptAccuTreeSlicesCf::VERSIONED_NAME,
    ExtensionsDataCf::NAME,
    AccountChangeStateVersionsCf::NAME,
    ExecutedGenesisScenariosCf::VERSIONED_NAME,
];

/// Committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `RawLedgerTransaction.as_ref::<[u8]>()`
/// Note: This table does not use explicit versioning wrapper, since the serialized content of
/// [`RawLedgerTransaction`] is itself versioned.
struct RawLedgerTransactionsCf;
impl DefaultCf<StateVersion, RawLedgerTransaction> for RawLedgerTransactionsCf {
    const DEFAULT_NAME: &'static str = "raw_ledger_transactions";
    type KeyCodec = StateVersionDbCodec;
    type ValueCodec = RawLedgerTransactionDbCodec;
}

/// Identifiers of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedCommittedTransactionIdentifiers)`
struct CommittedTransactionIdentifiersCf;
impl VersionedCf<StateVersion, CommittedTransactionIdentifiers>
    for CommittedTransactionIdentifiersCf
{
    const VERSIONED_NAME: &'static str = "committed_transaction_identifiers";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedCommittedTransactionIdentifiers;
}

/// Ledger receipts of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerTransactionReceipt)`
struct TransactionReceiptsCf;
impl VersionedCf<StateVersion, LedgerTransactionReceipt> for TransactionReceiptsCf {
    const VERSIONED_NAME: &'static str = "transaction_receipts";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLedgerTransactionReceipt;
}

/// Off-ledger details of committed transaction results (stored only when configured via
/// `enable_local_transaction_execution_index`).
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLocalTransactionExecution)`
struct LocalTransactionExecutionsCf;
impl VersionedCf<StateVersion, LocalTransactionExecution> for LocalTransactionExecutionsCf {
    const VERSIONED_NAME: &'static str = "local_transaction_executions";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLocalTransactionExecution;
}

/// Ledger proofs of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
struct LedgerProofsCf;
impl VersionedCf<StateVersion, LedgerProof> for LedgerProofsCf {
    const VERSIONED_NAME: &'static str = "ledger_proofs";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLedgerProof;
}

/// Ledger proofs of epochs.
/// Schema: `Epoch.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
/// Note: This duplicates a small subset of [`StateVersionToLedgerProof`]'s values.
struct EpochLedgerProofsCf;
impl VersionedCf<Epoch, LedgerProof> for EpochLedgerProofsCf {
    const VERSIONED_NAME: &'static str = "epoch_ledger_proofs";
    type KeyCodec = EpochDbCodec;
    type VersionedValue = VersionedLedgerProof;
}

/// DB "index" table for transaction's [`IntentHash`] resolution.
/// Schema: `IntentHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
/// Note: This table does not use explicit versioning wrapper, since the value represents a DB
/// key of another table (and versioning DB keys is not useful).
struct IntentHashesCf;
impl DefaultCf<IntentHash, StateVersion> for IntentHashesCf {
    const DEFAULT_NAME: &'static str = "intent_hashes";
    type KeyCodec = HashDbCodec<IntentHash>;
    type ValueCodec = StateVersionDbCodec;
}

/// DB "index" table for transaction's [`NotarizedTransactionHash`] resolution.
/// Schema: `NotarizedTransactionHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
/// Note: This table does not use explicit versioning wrapper, since the value represents a DB
/// key of another table (and versioning DB keys is not useful).
struct NotarizedTransactionHashesCf;
impl DefaultCf<NotarizedTransactionHash, StateVersion> for NotarizedTransactionHashesCf {
    const DEFAULT_NAME: &'static str = "notarized_transaction_hashes";
    type KeyCodec = HashDbCodec<NotarizedTransactionHash>;
    type ValueCodec = StateVersionDbCodec;
}

/// DB "index" table for transaction's [`LedgerTransactionHash`] resolution.
/// Schema: `LedgerTransactionHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
/// Note: This table does not use explicit versioning wrapper, since the value represents a DB
/// key of another table (and versioning DB keys is not useful).
struct LedgerTransactionHashesCf;
impl DefaultCf<LedgerTransactionHash, StateVersion> for LedgerTransactionHashesCf {
    const DEFAULT_NAME: &'static str = "ledger_transaction_hashes";
    type KeyCodec = HashDbCodec<LedgerTransactionHash>;
    type ValueCodec = StateVersionDbCodec;
}

/// Radix Engine's runtime Substate database.
/// Schema: `encode_to_rocksdb_bytes(DbPartitionKey, DbSortKey)` -> `Vec<u8>`
/// Note: This table does not use explicit versioning wrapper, since each serialized substate
/// value is already versioned.
struct SubstatesCf;
impl DefaultCf<DbSubstateKey, DbSubstateValue> for SubstatesCf {
    const DEFAULT_NAME: &'static str = "substates";
    type KeyCodec = SubstateKeyDbCodec;
    type ValueCodec = DirectDbCodec;
}

/// Ancestor information for the RE Nodes of [`Substates`] (which is useful and can be computed,
/// but is not provided by the Engine itself).
/// Schema: `NodeId.0` -> `scrypto_encode(VersionedSubstateNodeAncestryRecord)`
/// Note: we do not persist records of root Nodes (which do not have any ancestor).
struct SubstateNodeAncestryRecordsCf;
impl VersionedCf<NodeId, SubstateNodeAncestryRecord> for SubstateNodeAncestryRecordsCf {
    const VERSIONED_NAME: &'static str = "substate_node_ancestry_records";
    type KeyCodec = NodeIdDbCodec;
    type VersionedValue = VersionedSubstateNodeAncestryRecord;
}

/// Vertex store.
/// Schema: `[]` -> `scrypto_encode(VersionedVertexStore)`
/// Note: This is a single-entry table (i.e. the empty key is the only allowed key).
struct VertexStoreCf;
impl VersionedCf<(), VertexStoreBlob> for VertexStoreCf {
    const VERSIONED_NAME: &'static str = "vertex_store";
    type KeyCodec = UnitDbCodec;
    type VersionedValue = VersionedVertexStoreBlob;
}

/// Individual nodes of the Substate database's hash tree.
/// Schema: `encode_key(NodeKey)` -> `scrypto_encode(VersionedTreeNode)`.
struct StateHashTreeNodesCf;
impl VersionedCf<NodeKey, TreeNode> for StateHashTreeNodesCf {
    const VERSIONED_NAME: &'static str = "state_hash_tree_nodes";
    type KeyCodec = NodeKeyDbCodec;
    type VersionedValue = VersionedTreeNode;
}

/// Parts of the Substate database's hash tree that became stale at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedStaleTreeParts)`.
struct StaleStateHashTreePartsCf;
impl VersionedCf<StateVersion, StaleTreeParts> for StaleStateHashTreePartsCf {
    const VERSIONED_NAME: &'static str = "stale_state_hash_tree_parts";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedStaleTreeParts;
}

/// Transaction accumulator tree slices added at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedTransactionAccuTreeSlice)`.
struct TransactionAccuTreeSlicesCf;
impl VersionedCf<StateVersion, TransactionAccuTreeSlice> for TransactionAccuTreeSlicesCf {
    const VERSIONED_NAME: &'static str = "transaction_accu_tree_slices";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedTransactionAccuTreeSlice;
}

/// Receipt accumulator tree slices added at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedReceiptAccuTreeSlice)`.
struct ReceiptAccuTreeSlicesCf;
impl VersionedCf<StateVersion, ReceiptAccuTreeSlice> for ReceiptAccuTreeSlicesCf {
    const VERSIONED_NAME: &'static str = "receipt_accu_tree_slices";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedReceiptAccuTreeSlice;
}

/// Various data needed by extensions.
/// Schema: `ExtensionsDataKeys.to_string().as_bytes() -> Vec<u8>`.
/// Note: This table does not use explicit versioning wrapper, since each extension manages the
/// serialization of their data (of its custom type).
struct ExtensionsDataCf;
impl TypedCf<ExtensionsDataKey, Vec<u8>, PredefinedDbCodec<ExtensionsDataKey>, DirectDbCodec>
    for ExtensionsDataCf
{
    const NAME: &'static str = "extensions_data";

    fn key_codec(&self) -> PredefinedDbCodec<ExtensionsDataKey> {
        PredefinedDbCodec::new_from_string_representations(vec![
            ExtensionsDataKey::AccountChangeIndexEnabled,
            ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            ExtensionsDataKey::LocalTransactionExecutionIndexEnabled,
        ])
    }

    fn value_codec(&self) -> DirectDbCodec {
        DirectDbCodec::default()
    }
}

/// Account addresses and state versions at which they were changed.
/// Schema: `[GlobalAddress.0, StateVersion.to_bytes()].concat() -> []`.
/// Note: This is a key-only table (i.e. the empty value is the only allowed value). Given fast
/// prefix iterator from RocksDB this emulates a `Map<Account, Set<StateVersion>>`.
struct AccountChangeStateVersionsCf;
impl
    TypedCf<
        (GlobalAddress, StateVersion),
        (),
        PrefixGlobalAddressDbCodec<StateVersion, StateVersionDbCodec>,
        UnitDbCodec,
    > for AccountChangeStateVersionsCf
{
    const NAME: &'static str = "account_change_state_versions";

    fn key_codec(&self) -> PrefixGlobalAddressDbCodec<StateVersion, StateVersionDbCodec> {
        PrefixGlobalAddressDbCodec::new(StateVersionDbCodec::default())
    }

    fn value_codec(&self) -> UnitDbCodec {
        UnitDbCodec::default()
    }
}

/// Additional details of "Scenarios" (and their transactions) executed as part of Genesis,
/// keyed by their sequence number (i.e. their index in the list of Scenarios to execute).
/// Schema: `ScenarioSequenceNumber.to_be_bytes()` -> `scrypto_encode(VersionedExecutedGenesisScenario)`
struct ExecutedGenesisScenariosCf;
impl VersionedCf<ScenarioSequenceNumber, ExecutedGenesisScenario> for ExecutedGenesisScenariosCf {
    const VERSIONED_NAME: &'static str = "executed_genesis_scenarios";
    type KeyCodec = ScenarioSequenceNumberDbCodec;
    type VersionedValue = VersionedExecutedGenesisScenario;
}

/// An enum key for [`ExtensionsDataCf`].
#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Debug)]
enum ExtensionsDataKey {
    AccountChangeIndexLastProcessedStateVersion,
    AccountChangeIndexEnabled,
    LocalTransactionExecutionIndexEnabled,
}

// IMPORTANT NOTE: the strings defined below are used as database identifiers. Any change would
// effectively clear the associated extension's state in DB. For this reason, we choose to
// define them manually (rather than using the enum's `Into<String>`, which is refactor-sensitive).
impl fmt::Display for ExtensionsDataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::AccountChangeIndexLastProcessedStateVersion => {
                "account_change_index_last_processed_state_version"
            }
            Self::AccountChangeIndexEnabled => "account_change_index_enabled",
            Self::LocalTransactionExecutionIndexEnabled => {
                "local_transaction_execution_index_enabled"
            }
        };
        write!(f, "{str}")
    }
}

pub struct RocksDBStore {
    /// Database feature flags.
    ///
    /// These were passed during construction, validated and persisted. They are made available by
    /// this field as a cache.
    config: DatabaseFlags,

    /// Underlying RocksDB instance.
    ///
    /// **Note on usage:**
    /// A typical use-case should not need to access this field directly, but instead use a
    /// type-safe, write-buffering [`RocksDBStore::open_db_context()`].
    db: DB,
}

impl RocksDBStore {
    pub fn new(
        root: PathBuf,
        config: DatabaseFlags,
    ) -> Result<RocksDBStore, DatabaseConfigValidationError> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_COLUMN_FAMILIES
            .iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&db_opts, root.as_path(), column_families).unwrap();

        let rocks_db_store = RocksDBStore {
            config: config.clone(),
            db,
        };

        let current_database_config = rocks_db_store.read_flags_state();
        config.validate(&current_database_config)?;
        rocks_db_store.write_flags(&config);

        if rocks_db_store.config.enable_account_change_index {
            rocks_db_store.catchup_account_change_index();
        }

        Ok(rocks_db_store)
    }

    /// Creates a readonly [`RocksDBStore`] that allows reading from the store while some other
    /// process is writing to it. Any write operation that happens against a read-only store leads
    /// to a panic.
    pub fn new_read_only(root: PathBuf) -> Result<RocksDBStore, DatabaseConfigValidationError> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_COLUMN_FAMILIES
            .iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db =
            DB::open_cf_descriptors_read_only(&db_opts, root.as_path(), column_families, false)
                .unwrap();

        Ok(RocksDBStore {
            config: DatabaseFlags {
                enable_local_transaction_execution_index: false,
                enable_account_change_index: false,
            },
            db,
        })
    }

    /// Starts a read/batch-write interaction with the DB through per-CF type-safe APIs.
    fn open_db_context(&self) -> TypedDbContext {
        TypedDbContext::new(&self.db)
    }

    fn add_transaction_to_write_batch(
        &self,
        db_context: &TypedDbContext,
        transaction_bundle: CommittedTransactionBundle,
    ) {
        if self.is_account_change_index_enabled() {
            self.batch_update_account_change_index_from_committed_transaction(
                db_context,
                transaction_bundle.state_version,
                &transaction_bundle,
            );
        }

        let CommittedTransactionBundle {
            state_version,
            raw,
            receipt,
            identifiers,
        } = transaction_bundle;
        let ledger_transaction_hash = identifiers.payload.ledger_transaction_hash;

        // TEMPORARY until this is handled in the engine: we store both an intent lookup and the transaction itself
        if let TypedTransactionIdentifiers::User {
            intent_hash,
            notarized_transaction_hash,
            ..
        } = &identifiers.payload.typed
        {
            /* For user transactions we only need to check for duplicate intent hashes to know
            that user payload hash and ledger payload hash are also unique. */

            let maybe_existing_state_version = db_context.cf(IntentHashesCf).get(intent_hash);
            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save intent hash {:?} which already exists at state version {:?}",
                    intent_hash, existing_state_version
                );
            }

            db_context
                .cf(IntentHashesCf)
                .put(intent_hash, &state_version);
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

impl ConfigurableDatabase for RocksDBStore {
    fn read_flags_state(&self) -> DatabaseFlagsState {
        let db_context = self.open_db_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        let account_change_index_enabled = extension_data_cf
            .get(&ExtensionsDataKey::AccountChangeIndexEnabled)
            .map(|bytes| scrypto_decode::<bool>(&bytes).unwrap());
        let local_transaction_execution_index_enabled = extension_data_cf
            .get(&ExtensionsDataKey::LocalTransactionExecutionIndexEnabled)
            .map(|bytes| scrypto_decode::<bool>(&bytes).unwrap());
        DatabaseFlagsState {
            account_change_index_enabled,
            local_transaction_execution_index_enabled,
        }
    }

    fn write_flags(&self, database_config: &DatabaseFlags) {
        let db_context = self.open_db_context();
        let extension_data_cf = db_context.cf(ExtensionsDataCf);
        extension_data_cf.put(
            &ExtensionsDataKey::AccountChangeIndexEnabled,
            &scrypto_encode(&database_config.enable_account_change_index).unwrap(),
        );
        extension_data_cf.put(
            &ExtensionsDataKey::LocalTransactionExecutionIndexEnabled,
            &scrypto_encode(&database_config.enable_local_transaction_execution_index).unwrap(),
        );
    }

    fn is_account_change_index_enabled(&self) -> bool {
        self.config.enable_account_change_index
    }

    fn is_local_transaction_execution_index_enabled(&self) -> bool {
        self.config.enable_local_transaction_execution_index
    }
}

impl MeasurableDatabase for RocksDBStore {
    fn get_data_volume_statistics(&self) -> Vec<CategoryDbVolumeStatistic> {
        let mut statistics = ALL_COLUMN_FAMILIES
            .iter()
            .map(|cf_name| {
                (
                    cf_name.to_string(),
                    CategoryDbVolumeStatistic::zero(cf_name.to_string()),
                )
            })
            .collect::<IndexMap<_, _>>();
        let live_files = match self.db.live_files() {
            Ok(live_files) => live_files,
            Err(err) => {
                warn!("could not get DB live files; returning 0: {:?}", err);
                Vec::new()
            }
        };
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
}

impl CommitStore for RocksDBStore {
    fn commit(&self, commit_bundle: CommitBundle) {
        let db_context = self.open_db_context();

        // Check for duplicate intent/payload hashes in the commit request
        let mut user_transactions_count = 0;
        let mut processed_intent_hashes = HashSet::new();
        let transactions_count = commit_bundle.transactions.len();
        let mut processed_ledger_transaction_hashes = HashSet::new();

        let commit_ledger_header = &commit_bundle.proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;

        for transaction_bundle in commit_bundle.transactions {
            let payload_identifiers = &transaction_bundle.identifiers.payload;
            if let TypedTransactionIdentifiers::User { intent_hash, .. } =
                &payload_identifiers.typed
            {
                processed_intent_hashes.insert(*intent_hash);
                user_transactions_count += 1;
            }
            processed_ledger_transaction_hashes.insert(payload_identifiers.ledger_transaction_hash);
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

        let substates_cf = db_context.cf(SubstatesCf);
        for (node_key, node_updates) in commit_bundle.substate_store_update.updates.node_updates {
            for (partition_num, partition_updates) in node_updates.partition_updates {
                let partition_key = DbPartitionKey {
                    node_key: node_key.clone(),
                    partition_num,
                };
                match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        for (sort_key, update) in substate_updates {
                            let substate_key = (partition_key.clone(), sort_key);
                            match update {
                                DatabaseUpdate::Set(substate_value) => {
                                    substates_cf.put(&substate_key, &substate_value);
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
                        substates_cf.delete_range(
                            &(partition_key.clone(), DbSortKey(vec![])),
                            &(partition_key.next(), DbSortKey(vec![])),
                        );
                        for (sort_key, substate_value) in new_substate_values {
                            substates_cf.put(&(partition_key.clone(), sort_key), &substate_value);
                        }
                    }
                }
            }
        }

        if let Some(vertex_store) = commit_bundle.vertex_store {
            db_context.cf(VertexStoreCf).put(&(), &vertex_store);
        }

        let state_hash_tree_update = commit_bundle.state_tree_update;
        for (key, node) in state_hash_tree_update.new_nodes {
            db_context.cf(StateHashTreeNodesCf).put(&key, &node);
        }
        for (version, stale_parts) in state_hash_tree_update.stale_tree_parts_at_state_version {
            db_context
                .cf(StaleStateHashTreePartsCf)
                .put(&version, &stale_parts);
        }

        for (node_ids, record) in commit_bundle.new_substate_node_ancestry_records {
            for node_id in node_ids {
                db_context
                    .cf(SubstateNodeAncestryRecordsCf)
                    .put(&node_id, &record);
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

impl ExecutedGenesisScenarioStore for RocksDBStore {
    fn put_scenario(&self, number: ScenarioSequenceNumber, scenario: ExecutedGenesisScenario) {
        self.open_db_context()
            .cf(ExecutedGenesisScenariosCf)
            .put(&number, &scenario);
    }

    fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedGenesisScenario)> {
        self.open_db_context()
            .cf(ExecutedGenesisScenariosCf)
            .iterate(Direction::Forward)
            .collect()
    }
}

pub struct RocksDBCommittedTransactionBundleIterator<'db> {
    state_version: StateVersion,
    txns_iter: Box<dyn Iterator<Item = (StateVersion, RawLedgerTransaction)> + 'db>,
    ledger_receipts_iter: Box<dyn Iterator<Item = (StateVersion, LedgerTransactionReceipt)> + 'db>,
    local_executions_iter:
        Box<dyn Iterator<Item = (StateVersion, LocalTransactionExecution)> + 'db>,
    identifiers_iter:
        Box<dyn Iterator<Item = (StateVersion, CommittedTransactionIdentifiers)> + 'db>,
}

impl<'db> RocksDBCommittedTransactionBundleIterator<'db> {
    fn new(from_state_version: StateVersion, db_context: TypedDbContext<'db>) -> Self {
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

impl<'db> Iterator for RocksDBCommittedTransactionBundleIterator<'db> {
    type Item = CommittedTransactionBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let Some((txn_version, txn)) = self.txns_iter.next() else {
            return None;
        };

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

impl IterableTransactionStore for RocksDBStore {
    fn get_committed_transaction_bundle_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = CommittedTransactionBundle> + '_> {
        // This should not happen. This interface should be used after checking (e.g. `core-api-server/src/core-api/handlers/`).
        // However, with or without this debug_assert there would still be a panic if LocalTransactionExecution is missing.
        debug_assert!(self.is_local_transaction_execution_index_enabled());

        Box::new(RocksDBCommittedTransactionBundleIterator::new(
            from_state_version,
            self.open_db_context(),
        ))
    }
}

impl QueryableTransactionStore for RocksDBStore {
    fn get_committed_transaction(
        &self,
        state_version: StateVersion,
    ) -> Option<RawLedgerTransaction> {
        self.open_db_context()
            .cf(RawLedgerTransactionsCf)
            .get(&state_version)
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: StateVersion,
    ) -> Option<CommittedTransactionIdentifiers> {
        self.open_db_context()
            .cf(CommittedTransactionIdentifiersCf)
            .get(&state_version)
    }

    fn get_committed_ledger_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LedgerTransactionReceipt> {
        self.open_db_context()
            .cf(TransactionReceiptsCf)
            .get(&state_version)
    }

    fn get_committed_local_transaction_execution(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionExecution> {
        self.open_db_context()
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

impl TransactionIndex<&IntentHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        intent_hash: &IntentHash,
    ) -> Option<StateVersion> {
        self.open_db_context().cf(IntentHashesCf).get(intent_hash)
    }
}

impl TransactionIndex<&NotarizedTransactionHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<StateVersion> {
        self.open_db_context()
            .cf(NotarizedTransactionHashesCf)
            .get(notarized_transaction_hash)
    }
}

impl TransactionIndex<&LedgerTransactionHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        ledger_transaction_hash: &LedgerTransactionHash,
    ) -> Option<StateVersion> {
        self.open_db_context()
            .cf(LedgerTransactionHashesCf)
            .get(ledger_transaction_hash)
    }
}

impl TransactionIdentifierLoader for RocksDBStore {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)> {
        self.open_db_context()
            .cf(CommittedTransactionIdentifiersCf)
            .get_last()
    }
}

impl IterableProofStore for RocksDBStore {
    fn get_proof_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = LedgerProof> + '_> {
        Box::new(
            self.open_db_context()
                .cf(LedgerProofsCf)
                .iterate_from(&from_state_version, Direction::Forward)
                .map(|(_, proof)| proof),
        )
    }
}

impl QueryableProofStore for RocksDBStore {
    fn max_state_version(&self) -> StateVersion {
        self.open_db_context()
            .cf(RawLedgerTransactionsCf)
            .get_last_key()
            .unwrap_or(StateVersion::pre_genesis())
    }

    fn get_txns_and_proof(
        &self,
        start_state_version_inclusive: StateVersion,
        max_number_of_txns_if_more_than_one_proof: u32,
        max_payload_size_in_bytes: u32,
    ) -> Option<(Vec<RawLedgerTransaction>, LedgerProof)> {
        let mut payload_size_so_far = 0;
        let mut latest_usable_proof: Option<LedgerProof> = None;
        let mut txns = Vec::new();

        let mut proofs_iter = self
            .open_db_context()
            .cf(LedgerProofsCf)
            .iterate_from(&start_state_version_inclusive, Direction::Forward);
        let mut txns_iter = self
            .open_db_context()
            .cf(RawLedgerTransactionsCf)
            .iterate_from(&start_state_version_inclusive, Direction::Forward);

        'proof_loop: while payload_size_so_far <= max_payload_size_in_bytes
            && txns.len() <= (max_number_of_txns_if_more_than_one_proof as usize)
        {
            // Fetch next proof and see if all txns it includes can fit
            // If they do - add them to the output and update the latest usable proof then continue the iteration
            // If they don't - (sadly) ignore this proof's txns read so far and break the loop
            // If we're out of proofs (or some txns are missing): also break the loop
            match proofs_iter.next() {
                Some((next_proof_state_version, next_proof)) => {
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
                                payload_size_including_next_proof_txns += next_txn.0.len() as u32;
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
                        let next_proof_at_epoch = next_proof.ledger_header.next_epoch.is_some();
                        latest_usable_proof = Some(next_proof);
                        txns.append(&mut next_proof_txns);
                        payload_size_so_far = payload_size_including_next_proof_txns;

                        if next_proof_at_epoch {
                            // Stop if we've reached an epoch proof
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

        latest_usable_proof.map(|proof| (txns, proof))
    }

    fn get_first_proof(&self) -> Option<LedgerProof> {
        self.open_db_context().cf(LedgerProofsCf).get_first_value()
    }

    fn get_post_genesis_epoch_proof(&self) -> Option<LedgerProof> {
        self.open_db_context()
            .cf(EpochLedgerProofsCf)
            .get_first_value()
    }

    fn get_epoch_proof(&self, epoch: Epoch) -> Option<LedgerProof> {
        self.open_db_context().cf(EpochLedgerProofsCf).get(&epoch)
    }

    fn get_last_proof(&self) -> Option<LedgerProof> {
        self.open_db_context().cf(LedgerProofsCf).get_last_value()
    }

    fn get_last_epoch_proof(&self) -> Option<LedgerProof> {
        self.open_db_context()
            .cf(EpochLedgerProofsCf)
            .get_last_value()
    }
}

impl SubstateDatabase for RocksDBStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.open_db_context()
            .cf(SubstatesCf)
            .get(&(partition_key.clone(), sort_key.clone()))
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        Box::new(
            self.open_db_context()
                .cf(SubstatesCf)
                .iterate_from(
                    &(partition_key.clone(), DbSortKey(vec![])),
                    Direction::Forward,
                )
                .take_while(move |((next_key, _), _)| next_key == &partition_key)
                .map(|((_, sort_key), value)| (sort_key, value)),
        )
    }
}

impl ListableSubstateDatabase for RocksDBStore {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        Box::new(
            self.open_db_context()
                .cf(SubstatesCf)
                .iterate(Direction::Forward)
                .map(|(iter_key_bytes, _)| iter_key_bytes.0)
                // Rocksdb iterator returns sorted entries, so ok to to eliminate
                // duplicates with dedup()
                .dedup(),
        )
    }
}

impl SubstateNodeAncestryStore for RocksDBStore {
    fn batch_get_ancestry<'a>(
        &self,
        node_ids: impl IntoIterator<Item = &'a NodeId>,
    ) -> Vec<Option<SubstateNodeAncestryRecord>> {
        self.open_db_context()
            .cf(SubstateNodeAncestryRecordsCf)
            .get_many(Vec::from_iter(node_ids))
    }
}

impl ReadableTreeStore for RocksDBStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.open_db_context().cf(StateHashTreeNodesCf).get(key)
    }
}

impl StateHashTreeGcStore for RocksDBStore {
    fn get_stale_tree_parts_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (StateVersion, StaleTreeParts)> + '_> {
        self.open_db_context()
            .cf(StaleStateHashTreePartsCf)
            .iterate(Direction::Forward)
    }

    fn batch_delete_node<'a>(&self, keys: impl IntoIterator<Item = &'a NodeKey>) {
        let db_context = self.open_db_context();
        for key in keys {
            db_context.cf(StateHashTreeNodesCf).delete(key);
        }
    }

    fn batch_delete_stale_tree_part<'a>(
        &self,
        state_versions: impl IntoIterator<Item = &'a StateVersion>,
    ) {
        let db_context = self.open_db_context();
        for state_version in state_versions {
            db_context
                .cf(StaleStateHashTreePartsCf)
                .delete(state_version);
        }
    }
}

impl ReadableAccuTreeStore<StateVersion, TransactionTreeHash> for RocksDBStore {
    fn get_tree_slice(
        &self,
        state_version: &StateVersion,
    ) -> Option<TreeSlice<TransactionTreeHash>> {
        self.open_db_context()
            .cf(TransactionAccuTreeSlicesCf)
            .get(state_version)
            .map(|slice| slice.0)
    }
}

impl ReadableAccuTreeStore<StateVersion, ReceiptTreeHash> for RocksDBStore {
    fn get_tree_slice(&self, state_version: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.open_db_context()
            .cf(ReceiptAccuTreeSlicesCf)
            .get(state_version)
            .map(|slice| slice.0)
    }
}

impl WriteableVertexStore for RocksDBStore {
    fn save_vertex_store(&self, blob: VertexStoreBlob) {
        self.open_db_context().cf(VertexStoreCf).put(&(), &blob)
    }
}

impl RecoverableVertexStore for RocksDBStore {
    fn get_vertex_store(&self) -> Option<VertexStoreBlob> {
        self.open_db_context().cf(VertexStoreCf).get(&())
    }
}

fn encode_to_rocksdb_bytes(partition_key: &DbPartitionKey, sort_key: &DbSortKey) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + partition_key.node_key.len() + 1 + sort_key.0.len());
    buffer.push(
        u8::try_from(partition_key.node_key.len())
            .expect("Node key length is effectively constant 32 so should fit in a u8"),
    );
    buffer.extend(partition_key.node_key.clone());
    buffer.push(partition_key.partition_num);
    buffer.extend(sort_key.0.clone());
    buffer
}

fn decode_from_rocksdb_bytes(buffer: &[u8]) -> DbSubstateKey {
    let node_key_start: usize = 1usize;
    let partition_key_start = 1 + usize::from(buffer[0]);
    let sort_key_start = 1 + partition_key_start;

    let node_key = buffer[node_key_start..partition_key_start].to_vec();
    let partition_num = buffer[partition_key_start];
    let sort_key = buffer[sort_key_start..].to_vec();
    (
        DbPartitionKey {
            node_key,
            partition_num,
        },
        DbSortKey(sort_key),
    )
}

impl RocksDBStore {
    fn batch_update_account_change_index_from_receipt(
        &self,
        db_context: &TypedDbContext,
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
        db_context: &TypedDbContext,
        state_version: StateVersion,
        transaction_bundle: &CommittedTransactionBundle,
    ) {
        self.batch_update_account_change_index_from_receipt(
            db_context,
            state_version,
            &transaction_bundle.receipt.local_execution,
        );

        db_context.cf(ExtensionsDataCf).put(
            &ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            &state_version.to_bytes().to_vec(),
        );
    }

    fn update_account_change_index_from_store(
        &self,
        start_state_version_inclusive: StateVersion,
        limit: u64,
    ) -> StateVersion {
        let db_context = self.open_db_context();
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
            &last_state_version.to_bytes().to_vec(),
        );

        last_state_version
    }
}

impl AccountChangeIndexExtension for RocksDBStore {
    fn account_change_index_last_processed_state_version(&self) -> StateVersion {
        self.open_db_context()
            .cf(ExtensionsDataCf)
            .get(&ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion)
            .map(StateVersion::from_bytes)
            .unwrap_or(StateVersion::pre_genesis())
    }

    fn catchup_account_change_index(&self) {
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

impl IterableAccountChangeIndex for RocksDBStore {
    fn get_state_versions_for_account_iter(
        &self,
        account: GlobalAddress,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = StateVersion> + '_> {
        Box::new(
            self.open_db_context()
                .cf(AccountChangeStateVersionsCf)
                .iterate_from(&(account, from_state_version), Direction::Forward)
                .take_while(move |((next_account, _), _)| next_account == &account)
                .map(|((_, state_version), _)| state_version),
        )
    }
}

// Concrete DB-level codecs of keys/values:

#[derive(Default)]
struct StateVersionDbCodec {}

impl DbCodec<StateVersion> for StateVersionDbCodec {
    fn encode(&self, value: &StateVersion) -> Vec<u8> {
        value.to_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> StateVersion {
        StateVersion::from_bytes(bytes)
    }
}

#[derive(Default)]
struct EpochDbCodec {}

impl DbCodec<Epoch> for EpochDbCodec {
    fn encode(&self, value: &Epoch) -> Vec<u8> {
        value.number().to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> Epoch {
        Epoch::of(u64::from_be_bytes(copy_u8_array(bytes)))
    }
}

#[derive(Default)]
struct ScenarioSequenceNumberDbCodec {}

impl DbCodec<ScenarioSequenceNumber> for ScenarioSequenceNumberDbCodec {
    fn encode(&self, value: &ScenarioSequenceNumber) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> ScenarioSequenceNumber {
        ScenarioSequenceNumber::from_be_bytes(copy_u8_array(bytes))
    }
}

#[derive(Default)]
struct RawLedgerTransactionDbCodec {}

impl DbCodec<RawLedgerTransaction> for RawLedgerTransactionDbCodec {
    fn encode(&self, value: &RawLedgerTransaction) -> Vec<u8> {
        value.0.to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> RawLedgerTransaction {
        RawLedgerTransaction(bytes.to_vec())
    }
}

struct HashDbCodec<T: IsHash> {
    type_parameters_phantom: PhantomData<T>,
}

impl<T: IsHash> Default for HashDbCodec<T> {
    fn default() -> Self {
        Self {
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<T: IsHash> DbCodec<T> for HashDbCodec<T> {
    fn encode(&self, value: &T) -> Vec<u8> {
        value.as_slice().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> T {
        T::from_bytes(copy_u8_array(bytes))
    }
}

#[derive(Default)]
struct SubstateKeyDbCodec {}

impl DbCodec<DbSubstateKey> for SubstateKeyDbCodec {
    fn encode(&self, value: &DbSubstateKey) -> Vec<u8> {
        let (partition_key, sort_key) = value;
        encode_to_rocksdb_bytes(partition_key, sort_key)
    }

    fn decode(&self, bytes: &[u8]) -> DbSubstateKey {
        decode_from_rocksdb_bytes(bytes)
    }
}

#[derive(Default)]
struct NodeKeyDbCodec {}

impl DbCodec<NodeKey> for NodeKeyDbCodec {
    fn encode(&self, value: &NodeKey) -> Vec<u8> {
        encode_key(value)
    }

    fn decode(&self, _bytes: &[u8]) -> NodeKey {
        unimplemented!("no use-case for decoding hash tree's `NodeKey`s exists yet")
    }
}

struct PrefixGlobalAddressDbCodec<S, SC: DbCodec<S>> {
    suffix_codec: SC,
    type_parameters_phantom: PhantomData<S>,
}

impl<S, SC: DbCodec<S>> PrefixGlobalAddressDbCodec<S, SC> {
    pub fn new(suffix_codec: SC) -> Self {
        Self {
            suffix_codec,
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<S, SC: DbCodec<S>> DbCodec<(GlobalAddress, S)> for PrefixGlobalAddressDbCodec<S, SC> {
    fn encode(&self, (global_address, suffix): &(GlobalAddress, S)) -> Vec<u8> {
        let mut encoding = global_address.to_vec();
        encoding.extend_from_slice(self.suffix_codec.encode(suffix).as_slice());
        encoding
    }

    fn decode(&self, bytes: &[u8]) -> (GlobalAddress, S) {
        let global_address = GlobalAddress::new_or_panic(copy_u8_array(&bytes[..NodeId::LENGTH]));
        let suffix = self.suffix_codec.decode(&bytes[NodeId::LENGTH..]);
        (global_address, suffix)
    }
}

#[derive(Default)]
struct NodeIdDbCodec {}

impl DbCodec<NodeId> for NodeIdDbCodec {
    fn encode(&self, value: &NodeId) -> Vec<u8> {
        value.0.to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> NodeId {
        NodeId(copy_u8_array(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocksdb_key_encoding_is_invertible() {
        let partition_key = DbPartitionKey {
            node_key: vec![1, 2, 3, 4, 132],
            partition_num: 224,
        };
        let sort_key = DbSortKey(vec![13, 5]);
        let buffer = encode_to_rocksdb_bytes(&partition_key, &sort_key);

        let decoded = decode_from_rocksdb_bytes(&buffer);

        assert_eq!(partition_key, decoded.0);
        assert_eq!(sort_key, decoded.1);
    }

    /// This is needed for the iteration to work correctly
    #[test]
    fn rocksdb_key_encoding_respects_lexicographic_ordering_on_sort_keys() {
        let partition_key = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 1,
        };
        let sort_key = DbSortKey(vec![0, 4]);
        let iterator_start = encode_to_rocksdb_bytes(&partition_key, &sort_key);

        assert!(encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![0])) < iterator_start);
        assert!(encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![0, 3])) < iterator_start);
        assert_eq!(
            encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![0, 4])),
            iterator_start
        );
        assert!(iterator_start < encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![0, 5])));
        assert!(
            iterator_start < encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![0, 5, 7]))
        );
        assert!(iterator_start < encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![1, 51])));
    }
}
