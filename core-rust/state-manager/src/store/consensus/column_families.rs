use std::fmt;

use radix_common::crypto::Hash;
use radix_common::prelude::{EntityType, Epoch, GlobalAddress, NodeId, PackageAddress};
use radix_substate_store_impls::state_tree::tree_store::{
    StoredTreeNodeKey, TreeNode, VersionedTreeNode,
};
use radix_substate_store_interface::interface::{DbSubstateKey, DbSubstateValue};
use radix_transactions::model::{IntentHash, NotarizedTransactionHash};

use crate::{
    CommittedTransactionIdentifiers, LedgerProof, LedgerTransactionReceipt,
    LocalTransactionExecution, StateVersion, VersionedCommittedTransactionIdentifiers,
    VersionedLedgerProof, VersionedLedgerTransactionReceipt, VersionedLocalTransactionExecution,
};
use crate::consensus::traits::{
    ReceiptAccuTreeSlice, StaleTreeParts, SubstateNodeAncestryRecord, TransactionAccuTreeSlice,
    VersionedReceiptAccuTreeSlice, VersionedStaleTreeParts, VersionedSubstateNodeAncestryRecord,
    VersionedTransactionAccuTreeSlice, VersionedVertexStoreBlob, VertexStoreBlob,
};
use crate::consensus::traits::gc::{LedgerProofsGcProgress, VersionedLedgerProofsGcProgress};
use crate::consensus::traits::indices::{
    CreationId, EntityBlueprintId, ObjectBlueprintName, VersionedEntityBlueprintId,
    VersionedObjectBlueprintName,
};
use crate::consensus::traits::scenario::{
    ExecutedScenario, ScenarioSequenceNumber, VersionedExecutedScenario,
};
use crate::store::codecs::{
    BlueprintAndCreationIndexKeyDbCodec, EpochDbCodec, HashDbCodec, NodeIdDbCodec,
    PrefixGlobalAddressDbCodec, RawLedgerTransactionDbCodec, ScenarioSequenceNumberDbCodec,
    StateVersionDbCodec, StoredTreeNodeKeyDbCodec, SubstateKeyDbCodec,
    TypeAndCreationIndexKeyDbCodec,
};
use node_common::store::typed_cf_api::{
    DefaultCf, DirectDbCodec,
    PredefinedDbCodec, TypedCf, UnitDbCodec, VersionedCf,
};
use crate::transaction::{LedgerTransactionHash, RawLedgerTransaction};

/// Committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `RawLedgerTransaction.as_ref::<[u8]>()`
/// Note: This table does not use explicit versioning wrapper, since the serialized content of
/// [`RawLedgerTransaction`] is itself versioned.
pub struct RawLedgerTransactionsCf;
impl DefaultCf for RawLedgerTransactionsCf {
    type Key = StateVersion;
    type Value = RawLedgerTransaction;

    const DEFAULT_NAME: &'static str = "raw_ledger_transactions";
    type KeyCodec = StateVersionDbCodec;
    type ValueCodec = RawLedgerTransactionDbCodec;
}

/// Identifiers of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedCommittedTransactionIdentifiers)`
pub struct CommittedTransactionIdentifiersCf;
impl VersionedCf for CommittedTransactionIdentifiersCf {
    type Key = StateVersion;
    type Value = CommittedTransactionIdentifiers;

    const VERSIONED_NAME: &'static str = "committed_transaction_identifiers";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedCommittedTransactionIdentifiers;
}

/// Ledger receipts of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerTransactionReceipt)`
pub struct TransactionReceiptsCf;
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
pub struct LocalTransactionExecutionsCf;
impl VersionedCf for LocalTransactionExecutionsCf {
    type Key = StateVersion;
    type Value = LocalTransactionExecution;

    const VERSIONED_NAME: &'static str = "local_transaction_executions";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedLocalTransactionExecution;
}

/// Ledger proofs of committed transactions.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedLedgerProof)`
pub struct LedgerProofsCf;
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
pub struct EpochLedgerProofsCf;
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
pub struct ProtocolUpdateInitLedgerProofsCf;
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
pub struct ProtocolUpdateExecutionLedgerProofsCf;
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
pub struct IntentHashesCf;
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
pub struct NotarizedTransactionHashesCf;
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
pub struct LedgerTransactionHashesCf;
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
pub struct SubstatesCf;
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
pub struct SubstateNodeAncestryRecordsCf;
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
pub struct VertexStoreCf;
impl VersionedCf for VertexStoreCf {
    type Key = ();
    type Value = VertexStoreBlob;

    const VERSIONED_NAME: &'static str = "vertex_store";
    type KeyCodec = UnitDbCodec;
    type VersionedValue = VersionedVertexStoreBlob;
}

/// Individual nodes of the Substate database's hash tree.
/// Schema: `encode_key(StoredTreeNodeKey)` -> `scrypto_encode(VersionedTreeNode)`.
pub struct StateTreeNodesCf;
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
pub struct StaleStateTreePartsCf;
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
pub struct TransactionAccuTreeSlicesCf;
impl VersionedCf for TransactionAccuTreeSlicesCf {
    type Key = StateVersion;
    type Value = TransactionAccuTreeSlice;

    const VERSIONED_NAME: &'static str = "transaction_accu_tree_slices";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedTransactionAccuTreeSlice;
}

/// Receipt accumulator tree slices added at a specific state version.
/// Schema: `StateVersion.to_bytes()` -> `scrypto_encode(VersionedReceiptAccuTreeSlice)`.
pub struct ReceiptAccuTreeSlicesCf;
impl VersionedCf for ReceiptAccuTreeSlicesCf {
    type Key = StateVersion;
    type Value = ReceiptAccuTreeSlice;

    const VERSIONED_NAME: &'static str = "receipt_accu_tree_slices";
    type KeyCodec = StateVersionDbCodec;
    type VersionedValue = VersionedReceiptAccuTreeSlice;
}

/// An enum key for [`ExtensionsDataCf`].
#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Debug)]
pub enum ExtensionsDataKey {
    AccountChangeIndexLastProcessedStateVersion,
    AccountChangeIndexEnabled,
    LocalTransactionExecutionIndexEnabled,
    December2023LostSubstatesRestored,
    StateTreeAssociatedValuesStatus,
    EntityListingIndicesLastProcessedStateVersion,
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
            Self::EntityListingIndicesLastProcessedStateVersion => {
                "entity_listing_indices_last_processed_state_version"
            }
        };
        write!(f, "{str}")
    }
}

/// Various data needed by extensions.
/// Schema: `ExtensionsDataKeys.to_string().as_bytes() -> Vec<u8>`.
/// Note: This table does not use explicit versioning wrapper, since each extension manages the
/// serialization of their data (of its custom type).
pub struct ExtensionsDataCf;
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
            ExtensionsDataKey::EntityListingIndicesLastProcessedStateVersion,
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
pub struct AccountChangeStateVersionsCf;
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
/// Schema: `ScenarioSequenceNumber.to_be_bytes()` -> `scrypto_encode(VersionedExecutedScenario)`
pub struct ExecutedScenariosCf;
impl VersionedCf for ExecutedScenariosCf {
    type Key = ScenarioSequenceNumber;
    type Value = ExecutedScenario;

    // Note: a legacy name is still used here, even though we now have scenarios run outside Genesis
    const VERSIONED_NAME: &'static str = "executed_genesis_scenarios";
    type KeyCodec = ScenarioSequenceNumberDbCodec;
    type VersionedValue = VersionedExecutedScenario;
}

/// Progress of the GC process pruning the [`LedgerProofsCf`].
/// Schema: `[]` -> `scrypto_encode(VersionedLedgerProofsGcProgress)`
/// Note: This is a single-entry table (i.e. the empty key is the only allowed key).
pub struct LedgerProofsGcProgressCf;
impl VersionedCf for LedgerProofsGcProgressCf {
    type Key = ();
    type Value = LedgerProofsGcProgress;

    const VERSIONED_NAME: &'static str = "ledger_proofs_gc_progress";
    type KeyCodec = UnitDbCodec;
    type VersionedValue = VersionedLedgerProofsGcProgress;
}

/// Node IDs and blueprints of all entities, indexed by their type and creation order.
/// Schema: `[EntityType as u8, StateVersion.to_be_bytes(), (index_within_txn as u32).to_be_bytes()].concat()` -> `scrypto_encode(VersionedEntityBlueprintId)`
pub struct TypeAndCreationIndexedEntitiesCf;
impl VersionedCf for TypeAndCreationIndexedEntitiesCf {
    type Key = (EntityType, CreationId);
    type Value = EntityBlueprintId;

    const VERSIONED_NAME: &'static str = "type_and_creation_indexed_entities";
    type KeyCodec = TypeAndCreationIndexKeyDbCodec;
    type VersionedValue = VersionedEntityBlueprintId;
}

/// Node IDs and blueprints of all objects, indexed by their blueprint ID and creation order.
/// Schema: `[PackageAddress.0, hash(blueprint_name), StateVersion.to_be_bytes(), (index_within_txn as u32).to_be_bytes()].concat()` -> `scrypto_encode(VersionedObjectBlueprintName)`
pub struct BlueprintAndCreationIndexedObjectsCf;
impl VersionedCf for BlueprintAndCreationIndexedObjectsCf {
    type Key = (PackageAddress, Hash, CreationId);
    type Value = ObjectBlueprintName;

    const VERSIONED_NAME: &'static str = "blueprint_and_creation_indexed_objects";
    type KeyCodec = BlueprintAndCreationIndexKeyDbCodec;
    type VersionedValue = VersionedObjectBlueprintName;
}

/// Substate values associated with leaf nodes of the state hash tree's Substate Tier.
/// Needed for [`LeafSubstateValueStore`].
/// Note: This table does not use explicit versioning wrapper, since each serialized substate
/// value is already versioned.
pub struct AssociatedStateTreeValuesCf;
impl DefaultCf for AssociatedStateTreeValuesCf {
    type Key = StoredTreeNodeKey;
    type Value = DbSubstateValue;

    const DEFAULT_NAME: &'static str = "associated_state_tree_values";
    type KeyCodec = StoredTreeNodeKeyDbCodec;
    type ValueCodec = DirectDbCodec;
}
