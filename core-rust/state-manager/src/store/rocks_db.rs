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

use std::cmp::max;
use std::collections::HashSet;
use std::fmt;

use crate::engine_prelude::*;
use crate::store::traits::*;
use crate::{
    CommittedTransactionIdentifiers, LedgerProof, LedgerProofOrigin, LedgerTransactionReceipt,
    LocalTransactionExecution, LocalTransactionReceipt, ReceiptTreeHash, StateVersion,
    SubstateChangeAction, TransactionTreeHash, VersionedCommittedTransactionIdentifiers,
    VersionedLedgerProof, VersionedLedgerTransactionReceipt, VersionedLocalTransactionExecution,
};
use node_common::utils::IsAccountExt;
use rocksdb::checkpoint::Checkpoint;
use rocksdb::{
    AsColumnFamilyRef, ColumnFamily, ColumnFamilyDescriptor, DBPinnableSlice, Direction,
    IteratorMode, Options, Snapshot, WriteBatch, DB,
};

use std::path::PathBuf;

use node_common::locks::Snapshottable;
use tracing::{error, info, warn};

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::store::codecs::*;
use crate::store::historical_state::StateTreeBasedSubstateDatabase;
use crate::store::traits::gc::{
    LedgerProofsGcProgress, LedgerProofsGcStore, StateTreeGcStore, VersionedLedgerProofsGcProgress,
};
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
const ALL_COLUMN_FAMILIES: [&str; 23] = [
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
    ExecutedGenesisScenariosCf::VERSIONED_NAME,
    LedgerProofsGcProgressCf::VERSIONED_NAME,
    AssociatedStateTreeValuesCf::DEFAULT_NAME,
];

/// Committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `RawLedgerTransaction.as_ref::<[u8]>()`
/// Note: This table does not use explicit versioning wrapper, since the serialized content of
/// [`RawLedgerTransaction`] is itself versioned.
struct RawLedgerTransactionsCf;
impl DefaultCf for RawLedgerTransactionsCf {
    type Key = StateVersion;
    type Value = RawLedgerTransaction;

    const DEFAULT_NAME: &'static str = "raw_ledger_transactions";
    type KeyCodec = StateVersionDbCodec;
    type ValueCodec = RawLedgerTransactionDbCodec;
}

/// Identifiers of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedCommittedTransactionIdentifiers)`
struct CommittedTransactionIdentifiersCf;
impl VersionedCf for CommittedTransactionIdentifiersCf {
    type Key = StateVersion;
    type Value = CommittedTransactionIdentifiers;

    const VERSIONED_NAME: &'static str = "committed_transaction_identifiers";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedCommittedTransactionIdentifiers;
}

/// Ledger receipts of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerTransactionReceipt)`
struct TransactionReceiptsCf;
impl VersionedCf for TransactionReceiptsCf {
    type Key = StateVersion;
    type Value = LedgerTransactionReceipt;

    const VERSIONED_NAME: &'static str = "transaction_receipts";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLedgerTransactionReceipt;
}

/// Off-ledger details of committed transaction results (stored only when configured via
/// `enable_local_transaction_execution_index`).
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLocalTransactionExecution)`
struct LocalTransactionExecutionsCf;
impl VersionedCf for LocalTransactionExecutionsCf {
    type Key = StateVersion;
    type Value = LocalTransactionExecution;

    const VERSIONED_NAME: &'static str = "local_transaction_executions";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLocalTransactionExecution;
}

/// Ledger proofs of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
struct LedgerProofsCf;
impl VersionedCf for LedgerProofsCf {
    type Key = StateVersion;
    type Value = LedgerProof;

    const VERSIONED_NAME: &'static str = "ledger_proofs";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLedgerProof;
}

/// Ledger proofs of new epochs - i.e. the proofs which trigger the given `next_epoch`.
/// Schema: `Epoch.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
/// Note: This duplicates a small subset of [`LedgerProofsCf`]'s values.
struct EpochLedgerProofsCf;
impl VersionedCf for EpochLedgerProofsCf {
    type Key = Epoch;
    type Value = LedgerProof;

    const VERSIONED_NAME: &'static str = "epoch_ledger_proofs";
    type KeyCodec = EpochDbCodec;
    type VersionedValue = VersionedLedgerProof;
}

/// Ledger proofs that initialize protocol updates, i.e. proofs of Consensus `origin`,
/// with headers containing a non-empty `next_protocol_version`.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
/// Note: This duplicates a small subset of [`LedgerProofsCf`]'s values.
struct ProtocolUpdateInitLedgerProofsCf;
impl VersionedCf for ProtocolUpdateInitLedgerProofsCf {
    type Key = StateVersion;
    type Value = LedgerProof;

    const VERSIONED_NAME: &'static str = "protocol_update_init_ledger_proofs";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLedgerProof;
}

/// Ledger proofs of ProtocolUpdate `origin`, i.e. proofs created locally
/// while protocol update state modifications were being applied.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
/// Note: This duplicates a small subset of [`LedgerProofsCf`]'s values.
struct ProtocolUpdateExecutionLedgerProofsCf;
impl VersionedCf for ProtocolUpdateExecutionLedgerProofsCf {
    type Key = StateVersion;
    type Value = LedgerProof;

    const VERSIONED_NAME: &'static str = "protocol_update_execution_ledger_proofs";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLedgerProof;
}

/// DB "index" table for transaction's [`IntentHash`] resolution.
/// Schema: `IntentHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
/// Note: This table does not use explicit versioning wrapper, since the value represents a DB
/// key of another table (and versioning DB keys is not useful).
struct IntentHashesCf;
impl DefaultCf for IntentHashesCf {
    type Key = IntentHash;
    type Value = StateVersion;

    const DEFAULT_NAME: &'static str = "intent_hashes";
    type KeyCodec = HashDbCodec<IntentHash>;
    type ValueCodec = StateVersionDbCodec;
}

/// DB "index" table for transaction's [`NotarizedTransactionHash`] resolution.
/// Schema: `NotarizedTransactionHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
/// Note: This table does not use explicit versioning wrapper, since the value represents a DB
/// key of another table (and versioning DB keys is not useful).
struct NotarizedTransactionHashesCf;
impl DefaultCf for NotarizedTransactionHashesCf {
    type Key = NotarizedTransactionHash;
    type Value = StateVersion;

    const DEFAULT_NAME: &'static str = "notarized_transaction_hashes";
    type KeyCodec = HashDbCodec<NotarizedTransactionHash>;
    type ValueCodec = StateVersionDbCodec;
}

/// DB "index" table for transaction's [`LedgerTransactionHash`] resolution.
/// Schema: `LedgerTransactionHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
/// Note: This table does not use explicit versioning wrapper, since the value represents a DB
/// key of another table (and versioning DB keys is not useful).
struct LedgerTransactionHashesCf;
impl DefaultCf for LedgerTransactionHashesCf {
    type Key = LedgerTransactionHash;
    type Value = StateVersion;

    const DEFAULT_NAME: &'static str = "ledger_transaction_hashes";
    type KeyCodec = HashDbCodec<LedgerTransactionHash>;
    type ValueCodec = StateVersionDbCodec;
}

/// Radix Engine's runtime Substate database.
/// Schema: `encode_to_rocksdb_bytes(DbPartitionKey, DbSortKey)` -> `Vec<u8>`
/// Note: This table does not use explicit versioning wrapper, since each serialized substate
/// value is already versioned.
struct SubstatesCf;
impl DefaultCf for SubstatesCf {
    type Key = DbSubstateKey;
    type Value = DbSubstateValue;

    const DEFAULT_NAME: &'static str = "substates";
    type KeyCodec = SubstateKeyDbCodec;
    type ValueCodec = DirectDbCodec;
}

/// Ancestor information for the RE Nodes of [`Substates`] (which is useful and can be computed,
/// but is not provided by the Engine itself).
/// Schema: `NodeId.0` -> `scrypto_encode(VersionedSubstateNodeAncestryRecord)`
/// Note: we do not persist records of root Nodes (which do not have any ancestor).
struct SubstateNodeAncestryRecordsCf;
impl VersionedCf for SubstateNodeAncestryRecordsCf {
    type Key = NodeId;
    type Value = SubstateNodeAncestryRecord;

    const VERSIONED_NAME: &'static str = "substate_node_ancestry_records";
    type KeyCodec = NodeIdDbCodec;
    type VersionedValue = VersionedSubstateNodeAncestryRecord;
}

/// Vertex store.
/// Schema: `[]` -> `scrypto_encode(VersionedVertexStoreBlob)`
/// Note: This is a single-entry table (i.e. the empty key is the only allowed key).
struct VertexStoreCf;
impl VersionedCf for VertexStoreCf {
    type Key = ();
    type Value = VertexStoreBlob;

    const VERSIONED_NAME: &'static str = "vertex_store";
    type KeyCodec = UnitDbCodec;
    type VersionedValue = VersionedVertexStoreBlob;
}

/// Individual nodes of the Substate database's hash tree.
/// Schema: `encode_key(StoredTreeNodeKey)` -> `scrypto_encode(VersionedTreeNode)`.
struct StateTreeNodesCf;
impl VersionedCf for StateTreeNodesCf {
    type Key = StoredTreeNodeKey;
    type Value = TreeNode;

    // Note: the legacy `state_hash_tree` name lives on here because it already got persisted.
    const VERSIONED_NAME: &'static str = "state_hash_tree_nodes";
    type KeyCodec = StoredTreeNodeKeyDbCodec;
    type VersionedValue = VersionedTreeNode;
}

/// Parts of the Substate database's hash tree that became stale at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedStaleTreeParts)`.
struct StaleStateTreePartsCf;
impl VersionedCf for StaleStateTreePartsCf {
    type Key = StateVersion;
    type Value = StaleTreeParts;

    // Note: the legacy `state_hash_tree` name lives on here because it already got persisted.
    const VERSIONED_NAME: &'static str = "stale_state_hash_tree_parts";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedStaleTreeParts;
}

/// Transaction accumulator tree slices added at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedTransactionAccuTreeSlice)`.
struct TransactionAccuTreeSlicesCf;
impl VersionedCf for TransactionAccuTreeSlicesCf {
    type Key = StateVersion;
    type Value = TransactionAccuTreeSlice;

    const VERSIONED_NAME: &'static str = "transaction_accu_tree_slices";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedTransactionAccuTreeSlice;
}

/// Receipt accumulator tree slices added at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedReceiptAccuTreeSlice)`.
struct ReceiptAccuTreeSlicesCf;
impl VersionedCf for ReceiptAccuTreeSlicesCf {
    type Key = StateVersion;
    type Value = ReceiptAccuTreeSlice;

    const VERSIONED_NAME: &'static str = "receipt_accu_tree_slices";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedReceiptAccuTreeSlice;
}

/// Various data needed by extensions.
/// Schema: `ExtensionsDataKeys.to_string().as_bytes() -> Vec<u8>`.
/// Note: This table does not use explicit versioning wrapper, since each extension manages the
/// serialization of their data (of its custom type).
struct ExtensionsDataCf;
impl TypedCf for ExtensionsDataCf {
    type Key = ExtensionsDataKey;
    type Value = Vec<u8>;

    type KeyCodec = PredefinedDbCodec<ExtensionsDataKey>;
    type ValueCodec = DirectDbCodec;

    const NAME: &'static str = "extensions_data";

    fn key_codec(&self) -> PredefinedDbCodec<ExtensionsDataKey> {
        PredefinedDbCodec::new_from_string_representations(vec![
            ExtensionsDataKey::AccountChangeIndexEnabled,
            ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            ExtensionsDataKey::LocalTransactionExecutionIndexEnabled,
            ExtensionsDataKey::December2023LostSubstatesRestored,
            ExtensionsDataKey::StateTreeAssociatedValuesStatus,
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
impl TypedCf for AccountChangeStateVersionsCf {
    type Key = (GlobalAddress, StateVersion);
    type Value = ();

    type KeyCodec = PrefixGlobalAddressDbCodec<StateVersion, StateVersionDbCodec>;
    type ValueCodec = UnitDbCodec;

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
impl VersionedCf for ExecutedGenesisScenariosCf {
    type Key = ScenarioSequenceNumber;
    type Value = ExecutedGenesisScenario;

    const VERSIONED_NAME: &'static str = "executed_genesis_scenarios";
    type KeyCodec = ScenarioSequenceNumberDbCodec;
    type VersionedValue = VersionedExecutedGenesisScenario;
}

/// A progress of the GC process pruning the [`LedgerProofsCf`].
/// Schema: `[]` -> `scrypto_encode(VersionedLedgerProofsGcProgress)`
/// Note: This is a single-entry table (i.e. the empty key is the only allowed key).
struct LedgerProofsGcProgressCf;
impl VersionedCf for LedgerProofsGcProgressCf {
    type Key = ();
    type Value = LedgerProofsGcProgress;

    const VERSIONED_NAME: &'static str = "ledger_proofs_gc_progress";
    type KeyCodec = UnitDbCodec;
    type VersionedValue = VersionedLedgerProofsGcProgress;
}

/// Substate values associated with leaf nodes of the state hash tree's Substate Tier.
/// Needed for [`LeafSubstateValueStore`].
/// Note: This table does not use explicit versioning wrapper, since each serialized substate
/// value is already versioned.
struct AssociatedStateTreeValuesCf;
impl DefaultCf for AssociatedStateTreeValuesCf {
    type Key = StoredTreeNodeKey;
    type Value = DbSubstateValue;

    const DEFAULT_NAME: &'static str = "associated_state_tree_values";
    type KeyCodec = StoredTreeNodeKeyDbCodec;
    type ValueCodec = DirectDbCodec;
}

/// An enum key for [`ExtensionsDataCf`].
#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Debug)]
enum ExtensionsDataKey {
    AccountChangeIndexLastProcessedStateVersion,
    AccountChangeIndexEnabled,
    LocalTransactionExecutionIndexEnabled,
    December2023LostSubstatesRestored,
    StateTreeAssociatedValuesStatus,
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
            Self::December2023LostSubstatesRestored => "december_2023_lost_substates_restored",
            Self::StateTreeAssociatedValuesStatus => "state_tree_associated_values_status",
        };
        write!(f, "{str}")
    }
}

/// A redefined RocksDB's "key and value bytes" tuple (the original one lives in a private module).
pub type KVBytes = (Box<[u8]>, Box<[u8]>);

/// A trait capturing the common read methods present both in a "direct" RocksDB instance and in its
/// snapshots.
///
/// The library we use (a thin C wrapper, really) does not introduce this trivial and natural trait
/// itself, while we desperately need it to abstract the DB-reading code from the actual source of
/// data.
///
/// A note on changed error handling:
/// The original methods typically return [`Result`]s. Our trait assumes panics instead, since we
/// treat all database access errors as fatal anyways.
pub trait ReadableRocks {
    /// Resolves the column family by name.
    fn cf_handle(&self, name: &str) -> &ColumnFamily;

    /// Starts iteration over key-value pairs, according to the given [`IteratorMode`].
    fn iterator_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item = KVBytes> + '_>;

    /// Gets a single value by key.
    fn get_pinned_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: impl AsRef<[u8]>,
    ) -> Option<DBPinnableSlice>;

    /// Gets multiple values by keys.
    ///
    /// Syntax note:
    /// The `<'a>` here is not special at all: it could technically be 100% inferred. Just the
    /// compiler feature allowing to skip it from within the `<Item = &...>` is not yet stable.
    /// TODO(when the rustc feature mentioned above becomes stable): get rid of the `<'a>`.
    fn multi_get_cf<'a>(
        &'a self,
        keys: impl IntoIterator<Item = (&'a (impl AsColumnFamilyRef + 'a), impl AsRef<[u8]>)>,
    ) -> Vec<Option<Vec<u8>>>;
}

/// A write-supporting extension of the [`ReadableRocks`].
///
/// Naturally, it is expected that only a "direct" RocksDB instance can implement this one.
pub trait WriteableRocks: ReadableRocks {
    /// Atomically writes the given batch of updates.
    fn write(&self, batch: WriteBatch);

    /// Returns a snapshot of the current state.
    fn snapshot(&self) -> SnapshotRocks;
}

/// A [`ReadableRocks`] instance opened as secondary instance.
pub trait SecondaryRocks: ReadableRocks {
    /// Tries to catch up with the primary by reading as much as possible from the
    /// log files.
    fn try_catchup_with_primary(&self);
}

/// RocksDB checkpoint support.
pub trait CheckpointableRocks {
    fn create_checkpoint(&self, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error>;
}

/// Direct RocksDB instance.
pub struct DirectRocks {
    db: DB,
}

impl ReadableRocks for DirectRocks {
    fn cf_handle(&self, name: &str) -> &ColumnFamily {
        self.db.cf_handle(name).expect(name)
    }

    fn iterator_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item = KVBytes> + '_> {
        Box::new(
            self.db
                .iterator_cf(cf, mode)
                .map(|result| result.expect("reading from DB iterator")),
        )
    }

    fn get_pinned_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: impl AsRef<[u8]>,
    ) -> Option<DBPinnableSlice> {
        self.db.get_pinned_cf(cf, key).expect("DB get by key")
    }

    fn multi_get_cf<'a>(
        &'a self,
        keys: impl IntoIterator<Item = (&'a (impl AsColumnFamilyRef + 'a), impl AsRef<[u8]>)>,
    ) -> Vec<Option<Vec<u8>>> {
        self.db
            .multi_get_cf(keys)
            .into_iter()
            .map(|result| result.expect("batch DB get by key"))
            .collect()
    }
}

impl WriteableRocks for DirectRocks {
    fn write(&self, batch: WriteBatch) {
        self.db.write(batch).expect("DB write batch");
    }

    fn snapshot(&self) -> SnapshotRocks {
        SnapshotRocks {
            db: &self.db,
            snapshot: self.db.snapshot(),
        }
    }
}

impl SecondaryRocks for DirectRocks {
    fn try_catchup_with_primary(&self) {
        self.db
            .try_catch_up_with_primary()
            .expect("secondary DB catchup");
    }
}

impl CheckpointableRocks for DirectRocks {
    fn create_checkpoint(&self, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error> {
        create_checkpoint(&self.db, checkpoint_path)
    }
}

impl<'db> CheckpointableRocks for SnapshotRocks<'db> {
    fn create_checkpoint(&self, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error> {
        create_checkpoint(self.db, checkpoint_path)
    }
}

fn create_checkpoint(db: &DB, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error> {
    let checkpoint = Checkpoint::new(db)?;
    checkpoint.create_checkpoint(checkpoint_path)?;
    Ok(())
}

/// Snapshot of RocksDB.
///
/// Implementation note:
/// The original [`DB`] reference is interestingly kept internally by the [`Snapshot`] as well.
/// However, we need direct access to it for the [`Self::cf_handle()`] reasons.
pub struct SnapshotRocks<'db> {
    db: &'db DB,
    snapshot: Snapshot<'db>,
}

impl<'db> ReadableRocks for SnapshotRocks<'db> {
    fn cf_handle(&self, name: &str) -> &ColumnFamily {
        self.db.cf_handle(name).expect(name)
    }

    fn iterator_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item = KVBytes> + '_> {
        Box::new(
            self.snapshot
                .iterator_cf(cf, mode)
                .map(|result| result.expect("reading from snapshot DB iterator")),
        )
    }

    fn get_pinned_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: impl AsRef<[u8]>,
    ) -> Option<DBPinnableSlice> {
        self.snapshot
            .get_pinned_cf(cf, key)
            .expect("snapshot DB get by key")
    }

    fn multi_get_cf<'a>(
        &'a self,
        keys: impl IntoIterator<Item = (&'a (impl AsColumnFamilyRef + 'a), impl AsRef<[u8]>)>,
    ) -> Vec<Option<Vec<u8>>> {
        self.snapshot
            .multi_get_cf(keys)
            .into_iter()
            .map(|result| result.expect("batch snapshot DB get by key"))
            .collect()
    }
}

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

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_COLUMN_FAMILIES
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

        Ok(state_manager_database)
    }
}

impl<R: ReadableRocks> StateManagerDatabase<R> {
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
    pub fn new_read_only(root_path: PathBuf) -> StateManagerDatabase<impl ReadableRocks> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(false);
        db_opts.create_missing_column_families(false);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_COLUMN_FAMILIES
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
            },
            rocks: DirectRocks { db },
        }
    }
}

impl<R: SecondaryRocks> StateManagerDatabase<R> {
    /// Creates a [`StateManagerDatabase`] as a secondary instance which may catch up with the
    /// primary.
    pub fn new_as_secondary(
        root_path: PathBuf,
        temp_path: PathBuf,
        column_families: Vec<&str>,
    ) -> StateManagerDatabase<impl SecondaryRocks> {
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
                scrypto_decode::<VersionedStateTreeAssociatedValuesStatus>(&bytes)
                    .unwrap()
                    .into_latest()
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
                    values_associated_from: current_version,
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
                    "Disabling historical Substate values (were enabled since {:?})",
                    status.values_associated_from
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
        let substate_leaf_keys =
            StateTreeBasedSubstateDatabase::new(self, current_version).iter_substate_leaf_keys();
        for (tree_node_key, (partition_key, sort_key)) in substate_leaf_keys {
            let value = self
                .get_substate(&partition_key, &sort_key)
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

    fn get_first_stored_historical_state_version(&self) -> Option<StateVersion> {
        if !self.config.enable_historical_substate_values {
            return None; // state history feature disabled explicitly
        }

        let first_state_tree_version = self
            .open_read_context()
            .cf(StaleStateTreePartsCf)
            .get_first_key();
        let Some(first_state_tree_version) = first_state_tree_version else {
            return None; // JMT past gets immediately GC'ed - the history length must be 0
        };

        // we also need to take the "still collecting the max history length" case into account:
        let values_associated_from = self
            .open_read_context()
            .cf(ExtensionsDataCf)
            .get(&ExtensionsDataKey::StateTreeAssociatedValuesStatus)
            .map(|bytes| {
                scrypto_decode::<VersionedStateTreeAssociatedValuesStatus>(&bytes)
                    .unwrap()
                    .into_latest()
            })
            .expect("state history feature enabled, but its metadata not found")
            .values_associated_from;

        Some(max(first_state_tree_version, values_associated_from))
    }
}

impl MeasurableDatabase for ActualStateManagerDatabase {
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
        let live_files = match self.rocks.db.live_files() {
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
            for new_leaf_substate_key in commit_bundle.new_leaf_substate_keys {
                let LeafSubstateKeyAssociation {
                    tree_node_key,
                    substate_key,
                    cause,
                } = new_leaf_substate_key;
                let substate_value = match cause {
                    AssociationCause::SubstateUpsert => commit_bundle
                        .substate_store_update
                        .get_upserted_value(&substate_key)
                        .map(Cow::Borrowed)
                        .expect("upserted value not found in database updates"),
                    AssociationCause::TreeRestructuring => db_context
                        .cf(SubstatesCf)
                        .get(&substate_key)
                        .map(Cow::Owned)
                        .expect("unchanged value not found in substate database"),
                };
                associated_values_cf.put(&tree_node_key, substate_value.as_ref());
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

impl<R: WriteableRocks> ExecutedGenesisScenarioStore for StateManagerDatabase<R> {
    fn put_scenario(&self, number: ScenarioSequenceNumber, scenario: ExecutedGenesisScenario) {
        self.open_rw_context()
            .cf(ExecutedGenesisScenariosCf)
            .put(&number, &scenario);
    }

    fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedGenesisScenario)> {
        self.open_read_context()
            .cf(ExecutedGenesisScenariosCf)
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

impl<R: ReadableRocks> TransactionIndex<&IntentHash> for StateManagerDatabase<R> {
    fn get_txn_state_version_by_identifier(
        &self,
        intent_hash: &IntentHash,
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
        let mut encountered_genesis_proof = None;
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
                        LedgerProofOrigin::Genesis { .. } => {
                            encountered_genesis_proof = Some(next_proof);
                            break 'proof_loop;
                        }
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
                if let Some(genesis_proof) = encountered_genesis_proof {
                    GetSyncableTxnsAndProofError::RefusedToServeGenesis {
                        refused_proof: Box::new(genesis_proof),
                    }
                } else if let Some(protocol_update_proof) = encountered_protocol_update_proof {
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
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.open_read_context()
            .cf(SubstatesCf)
            .get(&(partition_key.clone(), sort_key.clone()))
    }

    fn list_entries_from(
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

    fn batch_delete_stale_tree_part<'a>(
        &self,
        state_versions: impl IntoIterator<Item = &'a StateVersion>,
    ) {
        let db_context = self.open_rw_context();
        let stale_tree_parts_cf = db_context.cf(StaleStateTreePartsCf);
        for state_version in state_versions {
            stale_tree_parts_cf.delete(state_version);
        }
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

impl<R: ReadableRocks> LeafSubstateValueStore for StateManagerDatabase<R> {
    fn get_associated_value(&self, tree_node_key: &StoredTreeNodeKey) -> Option<DbSubstateValue> {
        self.open_read_context()
            .cf(AssociatedStateTreeValuesCf)
            .get(tree_node_key)
    }
}
