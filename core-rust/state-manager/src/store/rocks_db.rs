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
use rocksdb::{ColumnFamilyDescriptor, Direction, Options, WriteBatch, DB};
use transaction::model::*;

use radix_engine_store_interface::interface::*;

use std::path::PathBuf;

use tracing::{error, info};

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::store::traits::scenario::{
    ExecutedGenesisScenario, ExecutedGenesisScenarioStore, ScenarioSequenceNumber,
    VersionedExecutedGenesisScenario,
};
use crate::store::typed_cf_api::*;
use crate::transaction::{
    LedgerTransactionHash, RawLedgerTransaction, TypedTransactionIdentifiers,
};

use super::traits::extensions::*;

/// Identifiers of column families used by the Node.
/// The name of each field should follow the `KeyTypeToValueType` convention, wherever possible.
/// The documentation of each field contains low-level details on the underlying RocksDB "schema"
/// (effectively: byte-level encodings of keys and values).
///
/// **An extra note on the key encoding used throughout all column families:**
/// We often rely on the RocksDB's unsurprising ability to efficiently list entries sorted
/// lexicographically by key. For this reason, our byte-level encoding of certain keys (e.g.
/// [`StateVersion`]) needs to reflect the business-level ordering of the represented concept (i.e.
/// since state versions grow, the "last" state version must have a lexicographically greatest key,
/// which means that we need to use a constant-length big-endian integer encoding).
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug, IntoStaticStr)]
enum RocksDBColumnFamily {
    /// Committed transactions.
    /// Schema: `StateVersion.to_bytes()` -> `RawLedgerTransaction.as_ref::<[u8]>()`
    /// Note: This table does not use explicit versioning wrapper, since the serialized content of
    /// [`RawLedgerTransaction`] is itself versioned.
    StateVersionToRawLedgerTransactionBytes,
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedCommittedTransactionIdentifiers)`
    StateVersionToCommittedTransactionIdentifiers,
    /// Ledger receipts of committed transactions.
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerTransactionReceipt)`
    StateVersionToLedgerTransactionReceipt,
    /// Off-ledger details of committed transaction results (stored only when configured via
    /// `enable_local_transaction_execution_index`).
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLocalTransactionExecution)`
    StateVersionToLocalTransactionExecution,
    /// Ledger proofs of committed transactions.
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
    StateVersionToLedgerProof,
    /// Ledger proofs of epochs.
    /// Schema: `Epoch.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
    /// Note: This duplicates a small subset of [`StateVersionToLedgerProof`]'s values.
    EpochToLedgerProof,
    /// DB "index" table for transaction's [`IntentHash`] resolution.
    /// Schema: `IntentHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
    /// Note: This table does not use explicit versioning wrapper, since the value represents a DB
    /// key of another table (and versioning DB keys is not useful).
    IntentHashToStateVersion,
    /// DB "index" table for transaction's [`NotarizedTransactionHash`] resolution.
    /// Schema: `NotarizedTransactionHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
    /// Note: This table does not use explicit versioning wrapper, since the value represents a DB
    /// key of another table (and versioning DB keys is not useful).
    NotarizedTransactionHashToStateVersion,
    /// DB "index" table for transaction's [`LedgerTransactionHash`] resolution.
    /// Schema: `LedgerTransactionHash.as_ref::<[u8]>()` -> `StateVersion.to_bytes()`
    /// Note: This table does not use explicit versioning wrapper, since the value represents a DB
    /// key of another table (and versioning DB keys is not useful).
    LedgerTransactionHashToStateVersion,
    /// Radix Engine's runtime Substate database.
    /// Schema: `encode_to_rocksdb_bytes(DbPartitionKey, DbSortKey)` -> `Vec<u8>`
    /// Note: This table does not use explicit versioning wrapper, since each serialized substate
    /// value is already versioned.
    Substates,
    /// Ancestor information for the RE Nodes of [`Substates`] (which is useful and can be computed,
    /// but is not provided by the Engine itself).
    /// Schema: `NodeId.0` -> `scrypto_encode(VersionedSubstateNodeAncestryRecord)`
    /// Note: we do not persist records of root Nodes (which do not have any ancestor).
    NodeIdToSubstateNodeAncestryRecord,
    /// Vertex store.
    /// Schema: `[]` -> `scrypto_encode(VersionedVertexStore)`
    /// Note: This is a single-entry table (i.e. the empty key is the only allowed key).
    VertexStore,
    /// Individual nodes of the Substate database's hash tree.
    /// Schema: `encode_key(NodeKey)` -> `scrypto_encode(VersionedTreeNode)`.
    NodeKeyToTreeNode,
    /// Parts of the Substate database's hash tree that became stale at a specific state version.
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedStaleTreeParts)`.
    StateVersionToStaleTreeParts,
    /// Transaction accumulator tree slices added at a specific state version.
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedTransactionAccuTreeSlice)`.
    StateVersionToTransactionAccuTreeSlice,
    /// Receipt accumulator tree slices added at a specific state version.
    /// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedReceiptAccuTreeSlice)`.
    StateVersionToReceiptAccuTreeSlice,
    /// Various data needed by extensions.
    /// Schema: `ExtensionsDataKeys.to_string().as_bytes() -> Vec<u8>`.
    /// Note: This table does not use explicit versioning wrapper, since each extension manages the
    /// serialization of their data (of its custom type).
    ExtensionsDataKeyToCustomValue,
    /// Account addresses and state versions at which they were changed.
    /// Schema: `[GlobalAddress.0, StateVersion.to_bytes()].concat() -> []`.
    /// Note: This is a key-only table (i.e. the empty value is the only allowed value). Given fast
    /// prefix iterator from RocksDB this emulates a `Map<Account, Set<StateVersion>>`.
    AccountChangeStateVersions,
    /// Additional details of "Scenarios" (and their transactions) executed as part of Genesis,
    /// keyed by their sequence number (i.e. their index in the list of Scenarios to execute).
    /// Schema: `ScenarioSequenceNumber.to_be_bytes()` -> `scrypto_encode(VersionedExecutedGenesisScenario)`
    ScenarioSequenceNumberToExecutedGenesisScenario,
}
use RocksDBColumnFamily::*;

const ALL_COLUMN_FAMILIES: [RocksDBColumnFamily; 19] = [
    StateVersionToRawLedgerTransactionBytes,
    StateVersionToCommittedTransactionIdentifiers,
    StateVersionToLedgerTransactionReceipt,
    StateVersionToLocalTransactionExecution,
    IntentHashToStateVersion,
    NotarizedTransactionHashToStateVersion,
    LedgerTransactionHashToStateVersion,
    StateVersionToLedgerProof,
    EpochToLedgerProof,
    Substates,
    NodeIdToSubstateNodeAncestryRecord,
    VertexStore,
    NodeKeyToTreeNode,
    StateVersionToStaleTreeParts,
    StateVersionToTransactionAccuTreeSlice,
    StateVersionToReceiptAccuTreeSlice,
    ExtensionsDataKeyToCustomValue,
    AccountChangeStateVersions,
    ScenarioSequenceNumberToExecutedGenesisScenario,
];

impl fmt::Display for RocksDBColumnFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.into())
    }
}

#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Debug)]
enum ExtensionsDataKey {
    AccountChangeIndexLastProcessedStateVersion,
    AccountChangeIndexEnabled,
    LocalTransactionExecutionIndexEnabled,
}

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

/// An entry-point to typed column family APIs ([`TypedCfApi`]s) of all tables used by Node.
struct NodeColumnFamilies {
    db: DB,
}

impl NodeColumnFamilies {
    /// Returns an API scoped at [`StateVersionToRawLedgerTransactionBytes`] column family.
    pub fn raw_ledger_transactions(&self) -> impl TypedCfApi<StateVersion, RawLedgerTransaction> {
        self.create_with_codecs(
            StateVersionToRawLedgerTransactionBytes,
            StateVersionDbCodec::default(),
            RawLedgerTransactionDbCodec::default(),
        )
    }

    /// Returns an API scoped at [`StateVersionToCommittedTransactionIdentifiers`] column family.
    pub fn committed_transaction_identifiers(
        &self,
    ) -> impl TypedCfApi<StateVersion, CommittedTransactionIdentifiers> {
        self.create_with_codecs(
            StateVersionToCommittedTransactionIdentifiers,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(
                SborDbCodec::<VersionedCommittedTransactionIdentifiers>::default(),
            ),
        )
    }

    /// Returns an API scoped at [`StateVersionToLedgerTransactionReceipt`] column family.
    pub fn transaction_receipts(&self) -> impl TypedCfApi<StateVersion, LedgerTransactionReceipt> {
        self.create_with_codecs(
            StateVersionToLedgerTransactionReceipt,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedLedgerTransactionReceipt>::default()),
        )
    }

    /// Returns an API scoped at [`StateVersionToLocalTransactionExecution`] column family.
    pub fn local_transaction_executions(
        &self,
    ) -> impl TypedCfApi<StateVersion, LocalTransactionExecution> {
        self.create_with_codecs(
            StateVersionToLocalTransactionExecution,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedLocalTransactionExecution>::default()),
        )
    }

    /// Returns an API scoped at [`StateVersionToLedgerProof`] column family.
    pub fn ledger_proofs(&self) -> impl TypedCfApi<StateVersion, LedgerProof> {
        self.create_with_codecs(
            StateVersionToLedgerProof,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedLedgerProof>::default()),
        )
    }

    /// Returns an API scoped at [`EpochToLedgerProof`] column family.
    pub fn epoch_ledger_proofs(&self) -> impl TypedCfApi<Epoch, LedgerProof> {
        self.create_with_codecs(
            EpochToLedgerProof,
            EpochDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedLedgerProof>::default()),
        )
    }

    /// Returns an API scoped at [`IntentHashToStateVersion`] column family.
    pub fn intent_hashes(&self) -> impl TypedCfApi<IntentHash, StateVersion> {
        self.create_with_codecs(
            IntentHashToStateVersion,
            HashDbCodec::default(),
            StateVersionDbCodec::default(),
        )
    }

    /// Returns an API scoped at [`NotarizedTransactionHashToStateVersion`] column family.
    pub fn notarized_transaction_hashes(
        &self,
    ) -> impl TypedCfApi<NotarizedTransactionHash, StateVersion> {
        self.create_with_codecs(
            NotarizedTransactionHashToStateVersion,
            HashDbCodec::default(),
            StateVersionDbCodec::default(),
        )
    }

    /// Returns an API scoped at [`LedgerTransactionHashToStateVersion`] column family.
    pub fn ledger_transaction_hashes(
        &self,
    ) -> impl TypedCfApi<LedgerTransactionHash, StateVersion> {
        self.create_with_codecs(
            LedgerTransactionHashToStateVersion,
            HashDbCodec::default(),
            StateVersionDbCodec::default(),
        )
    }

    /// Returns an API scoped at [`Substates`] column family.
    pub fn substates(&self) -> impl TypedCfApi<DbSubstateKey, DbSubstateValue> {
        self.create_with_codecs(
            Substates,
            SubstateKeyDbCodec::default(),
            DirectDbCodec::default(),
        )
    }

    /// Returns an API scoped at [`NodeIdToSubstateNodeAncestryRecord`] column family.
    pub fn substate_node_ancestry_records(
        &self,
    ) -> impl TypedCfApi<NodeId, SubstateNodeAncestryRecord> {
        self.create_with_codecs(
            NodeIdToSubstateNodeAncestryRecord,
            NodeIdDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedSubstateNodeAncestryRecord>::default()),
        )
    }

    /// Returns an API scoped at [`VertexStore`] column family.
    pub fn vertex_store(&self) -> impl TypedCfApi<(), VertexStoreBlob> {
        self.create_with_codecs(
            VertexStore,
            PredefinedDbCodec::for_unit(),
            VersionedDbCodec::new(SborDbCodec::<VersionedVertexStoreBlob>::default()),
        )
    }

    /// Returns an API scoped at [`NodeKeyToTreeNode`] column family.
    pub fn hash_tree_nodes(&self) -> impl TypedCfApi<NodeKey, TreeNode> {
        self.create_with_codecs(
            NodeKeyToTreeNode,
            NodeKeyDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedTreeNode>::default()),
        )
    }

    /// Returns an API scoped at [`StateVersionToStaleTreeParts`] column family.
    pub fn stale_hash_tree_parts(&self) -> impl TypedCfApi<StateVersion, StaleTreeParts> {
        self.create_with_codecs(
            StateVersionToStaleTreeParts,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedStaleTreeParts>::default()),
        )
    }

    /// Returns an API scoped at [`StateVersionToTransactionAccuTreeSlice`] column family.
    pub fn transaction_accu_tree_slices(
        &self,
    ) -> impl TypedCfApi<StateVersion, TransactionAccuTreeSlice> {
        self.create_with_codecs(
            StateVersionToTransactionAccuTreeSlice,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedTransactionAccuTreeSlice>::default()),
        )
    }

    /// Returns an API scoped at [`StateVersionToReceiptAccuTreeSlice`] column family.
    pub fn receipt_accu_tree_slices(&self) -> impl TypedCfApi<StateVersion, ReceiptAccuTreeSlice> {
        self.create_with_codecs(
            StateVersionToReceiptAccuTreeSlice,
            StateVersionDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedReceiptAccuTreeSlice>::default()),
        )
    }

    /// Returns an API scoped at [`ExtensionsDataKeyToCustomValue`] column family.
    pub fn extension_data(&self) -> impl TypedCfApi<ExtensionsDataKey, Vec<u8>> {
        self.create_with_codecs(
            ExtensionsDataKeyToCustomValue,
            PredefinedDbCodec::new_from_string_representations(vec![
                ExtensionsDataKey::AccountChangeIndexEnabled,
                ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
                ExtensionsDataKey::LocalTransactionExecutionIndexEnabled,
            ]),
            DirectDbCodec::default(),
        )
    }

    /// Returns an API scoped at [`AccountChangeStateVersions`] column family.
    pub fn account_change_state_versions(
        &self,
    ) -> impl TypedCfApi<(GlobalAddress, StateVersion), ()> {
        self.create_with_codecs(
            AccountChangeStateVersions,
            PrefixGlobalAddressDbCodec::new(StateVersionDbCodec::default()),
            PredefinedDbCodec::for_unit(),
        )
    }

    /// Returns an API scoped at [`ScenarioSequenceNumberToExecutedGenesisScenario`] column family.
    pub fn executed_genesis_scenarios(
        &self,
    ) -> impl TypedCfApi<ScenarioSequenceNumber, ExecutedGenesisScenario> {
        self.create_with_codecs(
            ScenarioSequenceNumberToExecutedGenesisScenario,
            ScenarioSequenceNumberDbCodec::default(),
            VersionedDbCodec::new(SborDbCodec::<VersionedExecutedGenesisScenario>::default()),
        )
    }

    /// Commits the given batch.
    // TODO(next potential refactor): this method only exists here since this structure hides the
    // `DB` under higher-level APIs, while our batch-handling logic still uses the low-level
    // `WriteBatch` directly. This can be refactored e.g. by introducing a "batch guard".
    pub fn commit_batch(&self, batch: WriteBatch) {
        self.db.write(batch).expect("committing database batch");
    }

    fn create_with_codecs<'db, K: 'db, KC: DbCodec<K> + 'db, V: 'db, VC: DbCodec<V> + 'db>(
        &'db self,
        cf: RocksDBColumnFamily,
        key_codec: KC,
        value_codec: VC,
    ) -> impl TypedCfApi<'db, K, V> {
        CodecBasedCfApi::new(&self.db, &cf.to_string(), key_codec, value_codec)
    }
}

pub struct RocksDBStore {
    config: DatabaseFlags,
    cfs: NodeColumnFamilies,
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

        let cfs = NodeColumnFamilies { db };
        let mut rocks_db_store = RocksDBStore { config, cfs };

        let current_database_config = rocks_db_store.read_flags_state();
        rocks_db_store.config.validate(&current_database_config)?;

        if rocks_db_store.config.enable_account_change_index {
            rocks_db_store.catchup_account_change_index();
        }

        Ok(rocks_db_store)
    }

    fn add_transaction_to_write_batch(
        &self,
        batch: &mut WriteBatch,
        transaction_bundle: CommittedTransactionBundle,
    ) {
        if self.is_account_change_index_enabled() {
            self.batch_update_account_change_index_from_committed_transaction(
                batch,
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

            let maybe_existing_state_version = self.cfs.intent_hashes().get(intent_hash);
            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save intent hash {:?} which already exists at state version {:?}",
                    intent_hash, existing_state_version
                );
            }

            self.cfs
                .intent_hashes()
                .put_with_batch(batch, intent_hash, &state_version);
            self.cfs.notarized_transaction_hashes().put_with_batch(
                batch,
                notarized_transaction_hash,
                &state_version,
            );
        } else {
            let maybe_existing_state_version = self
                .cfs
                .ledger_transaction_hashes()
                .get(&ledger_transaction_hash);
            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save ledger transaction hash {:?} which already exists at state version {:?}",
                    ledger_transaction_hash,
                    existing_state_version
                );
            }
        }

        self.cfs.ledger_transaction_hashes().put_with_batch(
            batch,
            &ledger_transaction_hash,
            &state_version,
        );
        self.cfs
            .raw_ledger_transactions()
            .put_with_batch(batch, &state_version, &raw);
        self.cfs.committed_transaction_identifiers().put_with_batch(
            batch,
            &state_version,
            &identifiers,
        );
        self.cfs
            .transaction_receipts()
            .put_with_batch(batch, &state_version, &receipt.on_ledger);

        if self.is_local_transaction_execution_index_enabled() {
            self.cfs.local_transaction_executions().put_with_batch(
                batch,
                &state_version,
                &receipt.local_execution,
            );
        }
    }
}

impl ConfigurableDatabase for RocksDBStore {
    fn read_flags_state(&self) -> DatabaseFlagsState {
        let extension_data_cf = self.cfs.extension_data();
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

    fn write_flags(&mut self, database_config: &DatabaseFlags) {
        let mut batch = WriteBatch::default();
        let extension_data_cf = self.cfs.extension_data();
        extension_data_cf.put_with_batch(
            &mut batch,
            &ExtensionsDataKey::AccountChangeIndexEnabled,
            &scrypto_encode(&database_config.enable_account_change_index).unwrap(),
        );
        extension_data_cf.put_with_batch(
            &mut batch,
            &ExtensionsDataKey::LocalTransactionExecutionIndexEnabled,
            &scrypto_encode(&database_config.enable_local_transaction_execution_index).unwrap(),
        );
        self.cfs.commit_batch(batch);
    }

    fn is_account_change_index_enabled(&self) -> bool {
        self.config.enable_account_change_index
    }

    fn is_local_transaction_execution_index_enabled(&self) -> bool {
        self.config.enable_local_transaction_execution_index
    }
}

impl CommitStore for RocksDBStore {
    fn commit(&mut self, commit_bundle: CommitBundle) {
        let mut batch = WriteBatch::default();

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
            self.add_transaction_to_write_batch(&mut batch, transaction_bundle);
        }

        if processed_intent_hashes.len() != user_transactions_count {
            panic!("Commit request contains duplicate intent hashes");
        }

        if processed_ledger_transaction_hashes.len() != transactions_count {
            panic!("Commit request contains duplicate ledger transaction hashes");
        }

        self.cfs.ledger_proofs().put_with_batch(
            &mut batch,
            &commit_state_version,
            &commit_bundle.proof,
        );
        if let Some(next_epoch) = &commit_ledger_header.next_epoch {
            self.cfs.epoch_ledger_proofs().put_with_batch(
                &mut batch,
                &next_epoch.epoch,
                &commit_bundle.proof,
            );
        }

        let substates_cf = self.cfs.substates();
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
                                    substates_cf.put_with_batch(
                                        &mut batch,
                                        &substate_key,
                                        &substate_value,
                                    );
                                }
                                DatabaseUpdate::Delete => {
                                    substates_cf.delete_with_batch(&mut batch, &substate_key);
                                }
                            }
                        }
                    }
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    } => {
                        substates_cf.delete_range_with_batch(
                            &mut batch,
                            &(partition_key.clone(), DbSortKey(vec![])),
                            &(partition_key.next(), DbSortKey(vec![])),
                        );
                        for (sort_key, substate_value) in new_substate_values {
                            substates_cf.put_with_batch(
                                &mut batch,
                                &(partition_key.clone(), sort_key),
                                &substate_value,
                            );
                        }
                    }
                }
            }
        }

        if let Some(vertex_store) = commit_bundle.vertex_store {
            self.cfs
                .vertex_store()
                .put_with_batch(&mut batch, &(), &vertex_store);
        }

        let state_hash_tree_update = commit_bundle.state_tree_update;
        for (key, node) in state_hash_tree_update.new_nodes {
            self.cfs
                .hash_tree_nodes()
                .put_with_batch(&mut batch, &key, &node);
        }
        for (version, stale_parts) in state_hash_tree_update.stale_tree_parts_at_state_version {
            self.cfs
                .stale_hash_tree_parts()
                .put_with_batch(&mut batch, &version, &stale_parts);
        }

        for (node_ids, record) in commit_bundle.new_substate_node_ancestry_records {
            for node_id in node_ids {
                self.cfs
                    .substate_node_ancestry_records()
                    .put_with_batch(&mut batch, &node_id, &record);
            }
        }

        self.cfs.transaction_accu_tree_slices().put_with_batch(
            &mut batch,
            &commit_state_version,
            &commit_bundle.transaction_tree_slice,
        );
        self.cfs.receipt_accu_tree_slices().put_with_batch(
            &mut batch,
            &commit_state_version,
            &commit_bundle.receipt_tree_slice,
        );

        self.cfs.commit_batch(batch);
    }
}

impl ExecutedGenesisScenarioStore for RocksDBStore {
    fn put_scenario(&mut self, number: ScenarioSequenceNumber, scenario: ExecutedGenesisScenario) {
        self.cfs
            .executed_genesis_scenarios()
            .put(&number, &scenario);
    }

    fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedGenesisScenario)> {
        self.cfs
            .executed_genesis_scenarios()
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
    fn new(from_state_version: StateVersion, store: &'db RocksDBStore) -> Self {
        Self {
            state_version: from_state_version,
            txns_iter: store
                .cfs
                .raw_ledger_transactions()
                .iterate_from(&from_state_version, Direction::Forward),
            ledger_receipts_iter: store
                .cfs
                .transaction_receipts()
                .iterate_from(&from_state_version, Direction::Forward),
            local_executions_iter: store
                .cfs
                .local_transaction_executions()
                .iterate_from(&from_state_version, Direction::Forward),
            identifiers_iter: store
                .cfs
                .committed_transaction_identifiers()
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
            self,
        ))
    }
}

impl QueryableTransactionStore for RocksDBStore {
    fn get_committed_transaction(
        &self,
        state_version: StateVersion,
    ) -> Option<RawLedgerTransaction> {
        self.cfs.raw_ledger_transactions().get(&state_version)
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: StateVersion,
    ) -> Option<CommittedTransactionIdentifiers> {
        self.cfs
            .committed_transaction_identifiers()
            .get(&state_version)
    }

    fn get_committed_ledger_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LedgerTransactionReceipt> {
        self.cfs.transaction_receipts().get(&state_version)
    }

    fn get_committed_local_transaction_execution(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionExecution> {
        self.cfs.local_transaction_executions().get(&state_version)
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
        self.cfs.intent_hashes().get(intent_hash)
    }
}

impl TransactionIndex<&NotarizedTransactionHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<StateVersion> {
        self.cfs
            .notarized_transaction_hashes()
            .get(notarized_transaction_hash)
    }
}

impl TransactionIndex<&LedgerTransactionHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        ledger_transaction_hash: &LedgerTransactionHash,
    ) -> Option<StateVersion> {
        self.cfs
            .ledger_transaction_hashes()
            .get(ledger_transaction_hash)
    }
}

impl TransactionIdentifierLoader for RocksDBStore {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)> {
        self.cfs.committed_transaction_identifiers().get_last()
    }
}

impl IterableProofStore for RocksDBStore {
    fn get_proof_iter(
        &self,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = LedgerProof> + '_> {
        Box::new(
            self.cfs
                .ledger_proofs()
                .iterate_from(&from_state_version, Direction::Forward)
                .map(|(_, proof)| proof),
        )
    }
}

impl QueryableProofStore for RocksDBStore {
    fn max_state_version(&self) -> StateVersion {
        self.cfs
            .raw_ledger_transactions()
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
            .cfs
            .ledger_proofs()
            .iterate_from(&start_state_version_inclusive, Direction::Forward);
        let mut txns_iter = self
            .cfs
            .raw_ledger_transactions()
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
        self.cfs.ledger_proofs().get_first_value()
    }

    fn get_post_genesis_epoch_proof(&self) -> Option<LedgerProof> {
        self.cfs.epoch_ledger_proofs().get_first_value()
    }

    fn get_epoch_proof(&self, epoch: Epoch) -> Option<LedgerProof> {
        self.cfs.epoch_ledger_proofs().get(&epoch)
    }

    fn get_last_proof(&self) -> Option<LedgerProof> {
        self.cfs.ledger_proofs().get_last_value()
    }

    fn get_last_epoch_proof(&self) -> Option<LedgerProof> {
        self.cfs.epoch_ledger_proofs().get_last_value()
    }
}

impl SubstateDatabase for RocksDBStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.cfs
            .substates()
            .get(&(partition_key.clone(), sort_key.clone()))
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        Box::new(
            self.cfs
                .substates()
                .iterate_from(
                    &(partition_key.clone(), DbSortKey(vec![])),
                    Direction::Forward,
                )
                .take_while(move |((next_key, _), _)| next_key == &partition_key)
                .map(|((_, sort_key), value)| (sort_key, value)),
        )
    }
}

impl SubstateNodeAncestryStore for RocksDBStore {
    fn batch_get_ancestry<'a>(
        &self,
        node_ids: impl IntoIterator<Item = &'a NodeId>,
    ) -> Vec<Option<SubstateNodeAncestryRecord>> {
        self.cfs
            .substate_node_ancestry_records()
            .get_many(Vec::from_iter(node_ids))
    }
}

impl ReadableTreeStore for RocksDBStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.cfs.hash_tree_nodes().get(key)
    }
}

impl ReadableAccuTreeStore<StateVersion, TransactionTreeHash> for RocksDBStore {
    fn get_tree_slice(
        &self,
        state_version: &StateVersion,
    ) -> Option<TreeSlice<TransactionTreeHash>> {
        self.cfs
            .transaction_accu_tree_slices()
            .get(state_version)
            .map(|slice| slice.0)
    }
}

impl ReadableAccuTreeStore<StateVersion, ReceiptTreeHash> for RocksDBStore {
    fn get_tree_slice(&self, state_version: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.cfs
            .receipt_accu_tree_slices()
            .get(state_version)
            .map(|slice| slice.0)
    }
}

impl WriteableVertexStore for RocksDBStore {
    fn save_vertex_store(&mut self, blob: VertexStoreBlob) {
        self.cfs.vertex_store().put(&(), &blob)
    }
}

impl RecoverableVertexStore for RocksDBStore {
    fn get_vertex_store(&self) -> Option<VertexStoreBlob> {
        self.cfs.vertex_store().get(&())
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
        batch: &mut WriteBatch,
        state_version: StateVersion,
        execution: &LocalTransactionExecution,
    ) {
        for address in execution
            .global_balance_summary
            .global_balance_changes
            .keys()
            .filter(|address| address.is_account())
        {
            self.cfs.account_change_state_versions().put_with_batch(
                batch,
                &(*address, state_version),
                &(),
            );
        }
    }

    fn batch_update_account_change_index_from_committed_transaction(
        &self,
        batch: &mut WriteBatch,
        state_version: StateVersion,
        transaction_bundle: &CommittedTransactionBundle,
    ) {
        self.batch_update_account_change_index_from_receipt(
            batch,
            state_version,
            &transaction_bundle.receipt.local_execution,
        );

        self.cfs.extension_data().put_with_batch(
            batch,
            &ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            &state_version.to_bytes().to_vec(),
        );
    }

    fn update_account_change_index_from_store(
        &mut self,
        start_state_version_inclusive: StateVersion,
        limit: u64,
    ) -> StateVersion {
        let mut executions_iter = self
            .cfs
            .local_transaction_executions()
            .iterate_from(&start_state_version_inclusive, Direction::Forward);

        let mut batch = WriteBatch::default();

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
                        &mut batch,
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

        self.cfs.extension_data().put_with_batch(
            &mut batch,
            &ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion,
            &last_state_version.to_bytes().to_vec(),
        );
        self.cfs.commit_batch(batch);

        last_state_version
    }
}

impl AccountChangeIndexExtension for RocksDBStore {
    fn account_change_index_last_processed_state_version(&self) -> StateVersion {
        self.cfs
            .extension_data()
            .get(&ExtensionsDataKey::AccountChangeIndexLastProcessedStateVersion)
            .map(StateVersion::from_bytes)
            .unwrap_or(StateVersion::pre_genesis())
    }

    fn catchup_account_change_index(&mut self) {
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
            self.cfs
                .account_change_state_versions()
                .iterate_from(&(account, from_state_version), Direction::Forward)
                .take_while(move |((next_account, _), _)| next_account == &account)
                .map(|((_, state_version), _)| state_version),
        )
    }
}

// Concrete DB-level codecs of keys/values:

#[derive(Clone, Default)]
struct StateVersionDbCodec {}

impl DbCodec<StateVersion> for StateVersionDbCodec {
    fn encode(&self, value: &StateVersion) -> Vec<u8> {
        value.to_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> StateVersion {
        StateVersion::from_bytes(bytes)
    }
}

#[derive(Clone, Default)]
struct EpochDbCodec {}

impl DbCodec<Epoch> for EpochDbCodec {
    fn encode(&self, value: &Epoch) -> Vec<u8> {
        value.number().to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> Epoch {
        Epoch::of(u64::from_be_bytes(copy_u8_array(bytes)))
    }
}

#[derive(Clone, Default)]
struct ScenarioSequenceNumberDbCodec {}

impl DbCodec<ScenarioSequenceNumber> for ScenarioSequenceNumberDbCodec {
    fn encode(&self, value: &ScenarioSequenceNumber) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> ScenarioSequenceNumber {
        ScenarioSequenceNumber::from_be_bytes(copy_u8_array(bytes))
    }
}

#[derive(Clone, Default)]
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

impl<T: IsHash> Clone for HashDbCodec<T> {
    fn clone(&self) -> Self {
        Self::default()
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

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
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

impl<S, SC: DbCodec<S>> Clone for PrefixGlobalAddressDbCodec<S, SC> {
    fn clone(&self) -> Self {
        Self::new(self.suffix_codec.clone())
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

#[derive(Clone, Default)]
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
