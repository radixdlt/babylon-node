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

use crate::types::UserPayloadHash;
use crate::utils::IsAccountExt;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::mem::size_of;

use crate::store::traits::*;
use crate::{
    AccumulatorHash, CommittedTransactionIdentifiers, HasIntentHash, HasLedgerPayloadHash,
    HasUserPayloadHash, IntentHash, LedgerPayloadHash, LedgerProof, LedgerTransactionReceipt,
    LocalTransactionExecution, LocalTransactionReceipt, ReceiptTreeHash, TransactionTreeHash,
};
use radix_engine::ledger::{OutputValue, QueryableSubstateStore, ReadableSubstateStore};
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{
    scrypto_decode, scrypto_encode, Address, KeyValueStoreId, KeyValueStoreOffset, RENodeId,
    SubstateId, SubstateOffset,
};
use radix_engine_interface::api::types::NodeModuleId;
use radix_engine_interface::data::manifest::manifest_decode;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use radix_engine_stores::hash_tree::tree_store::{
    encode_key, NodeKey, Payload, ReadableTreeStore, TreeNode,
};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBIteratorWithThreadMode, Direction, IteratorMode,
    Options, WriteBatch, DB,
};
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::transaction::LedgerTransaction;

use super::traits::extensions::*;

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
enum RocksDBColumnFamily {
    /// Committed transactions
    TxnByStateVersion,
    TxnAccumulatorHashByStateVersion,
    /// Receipts of committed transactions
    LedgerReceiptByStateVersion,
    LocalTransactionExecutionByStateVersion,
    /// Transaction lookups
    StateVersionByTxnIntentHash,
    StateVersionByTxnUserPayloadHash,
    StateVersionByTxnLedgerPayloadHash,
    /// Ledger proofs
    LedgerProofByStateVersion,
    LedgerProofByEpoch,
    /// Radix Engine state
    Substates,
    /// Vertex store
    VertexStore,
    /// State hash tree: all nodes + keys of nodes that became stale by the given state version
    StateHashTreeNodes,
    StaleStateHashTreeNodeKeysByStateVersion,
    /// Transaction/Receipt accumulator tree slices keyed by state version of their ledger header
    TransactionAccuTreeSliceByStateVersion,
    ReceiptAccuTreeSliceByStateVersion,
    /// Various data needed by extensions
    ExtensionsData,
    /// Account to state versions (which changed the account)
    /// Key: concat(account_address, state_version), value: null
    /// Given fast prefix iterator from RocksDB this emulates a Map<Account, Set<StateVersion>>
    AccountChangeStateVersions,
}

use RocksDBColumnFamily::*;

const ALL_COLUMN_FAMILIES: [RocksDBColumnFamily; 17] = [
    TxnByStateVersion,
    TxnAccumulatorHashByStateVersion,
    LedgerReceiptByStateVersion,
    LocalTransactionExecutionByStateVersion,
    StateVersionByTxnIntentHash,
    StateVersionByTxnUserPayloadHash,
    StateVersionByTxnLedgerPayloadHash,
    LedgerProofByStateVersion,
    LedgerProofByEpoch,
    Substates,
    VertexStore,
    StateHashTreeNodes,
    StaleStateHashTreeNodeKeysByStateVersion,
    TransactionAccuTreeSliceByStateVersion,
    ReceiptAccuTreeSliceByStateVersion,
    ExtensionsData,
    AccountChangeStateVersions,
];

impl fmt::Display for RocksDBColumnFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            TxnByStateVersion => "txn_by_state_version",
            TxnAccumulatorHashByStateVersion => "txn_accumulator_hash_by_state_version",
            LedgerReceiptByStateVersion => "ledger_receipt_by_state_version",
            LocalTransactionExecutionByStateVersion => {
                "local_transaction_execution_by_state_version"
            }
            StateVersionByTxnIntentHash => "state_version_by_txn_intent_hash",
            StateVersionByTxnUserPayloadHash => "state_version_by_txn_user_payload_hash",
            StateVersionByTxnLedgerPayloadHash => "state_version_by_txn_ledger_payload_hash",
            LedgerProofByStateVersion => "ledger_proof_by_state_version",
            LedgerProofByEpoch => "ledger_proof_by_epoch",
            Substates => "substates",
            VertexStore => "vertex_store",
            StateHashTreeNodes => "state_hash_tree_nodes",
            StaleStateHashTreeNodeKeysByStateVersion => "stale_state_hash_tree_node_keys",
            TransactionAccuTreeSliceByStateVersion => {
                "transaction_accu_tree_slice_by_state_version"
            }
            ReceiptAccuTreeSliceByStateVersion => "receipt_accu_tree_slice_by_state_version",
            ExtensionsData => "extensions_data",
            AccountChangeStateVersions => "account_change_state_versions",
        };
        write!(f, "{str}")
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
enum ExtensionsDataKeys {
    AccountChangeIndexLastProcessedStateVersion,
    AccountChangeIndexEnabled,
    LocalTransactionExecutionIndexEnabled,
}

impl fmt::Display for ExtensionsDataKeys {
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
    db: DB,
    config: DatabaseFlags,
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
            .into_iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&db_opts, root.as_path(), column_families).unwrap();

        let mut rocks_db_store = RocksDBStore { db, config };

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
                &transaction_bundle,
            );
        }

        let (transaction, receipt, identifiers) = transaction_bundle;
        let state_version = identifiers.state_version;
        let ledger_payload_hash = transaction.ledger_payload_hash();

        // TEMPORARY until this is handled in the engine: we store both an intent lookup and the transaction itself
        if let LedgerTransaction::User(notarized_transaction) = &transaction {
            /* For user transactions we only need to check for duplicate intent hashes to know
            that user payload hash and ledger payload hash are also unique. */
            let intent_hash = notarized_transaction.intent_hash();

            let maybe_existing_intent_hash = self
                .db
                .get_cf(self.cf_handle(&StateVersionByTxnIntentHash), intent_hash)
                .unwrap();

            if let Some(state_version) = maybe_existing_intent_hash {
                warn!("Duplicate transaction intent hash {:?}", transaction);
                panic!(
                    "Attempted to save intent hash {:?} which already exists at state version {:?}",
                    intent_hash,
                    u64::from_be_bytes(state_version.try_into().unwrap())
                );
            }

            batch.put_cf(
                self.cf_handle(&StateVersionByTxnIntentHash),
                intent_hash,
                state_version.to_be_bytes(),
            );

            batch.put_cf(
                self.cf_handle(&StateVersionByTxnUserPayloadHash),
                notarized_transaction.user_payload_hash(),
                state_version.to_be_bytes(),
            );
        } else {
            let maybe_existing_ledger_payload_hash = self
                .db
                .get_cf(
                    self.cf_handle(&StateVersionByTxnIntentHash),
                    ledger_payload_hash,
                )
                .unwrap();

            if let Some(state_version) = maybe_existing_ledger_payload_hash {
                warn!("Duplicate transaction {:?}", transaction);
                panic!(
                    "Attempted to save ledger payload hash {:?} which already exists at state version {:?}",
                    ledger_payload_hash,
                    u64::from_be_bytes(state_version.try_into().unwrap())
                );
            }
        }

        batch.put_cf(
            self.cf_handle(&StateVersionByTxnLedgerPayloadHash),
            ledger_payload_hash,
            state_version.to_be_bytes(),
        );

        batch.put_cf(
            self.cf_handle(&TxnByStateVersion),
            state_version.to_be_bytes(),
            transaction.create_payload().unwrap(),
        );

        batch.put_cf(
            self.cf_handle(&TxnAccumulatorHashByStateVersion),
            state_version.to_be_bytes(),
            identifiers.accumulator_hash.into_bytes(),
        );

        batch.put_cf(
            self.cf_handle(&LedgerReceiptByStateVersion),
            state_version.to_be_bytes(),
            scrypto_encode(&receipt.on_ledger).unwrap(),
        );

        if self.is_local_transaction_execution_index_enabled() {
            batch.put_cf(
                self.cf_handle(&LocalTransactionExecutionByStateVersion),
                state_version.to_be_bytes(),
                scrypto_encode(&receipt.local_execution).unwrap(),
            );
        }
    }

    fn cf_handle(&self, cf: &RocksDBColumnFamily) -> &ColumnFamily {
        self.db.cf_handle(&cf.to_string()).unwrap()
    }

    fn get_last<T: ScryptoDecode>(&self, cf: &RocksDBColumnFamily) -> Option<T> {
        self.db
            .iterator_cf(self.cf_handle(cf), IteratorMode::End)
            .map(|res| res.unwrap())
            .next()
            .map(|(_, value)| scrypto_decode(value.as_ref()).unwrap())
    }

    fn get_by_key<T: ScryptoDecode>(&self, cf: &RocksDBColumnFamily, key: &[u8]) -> Option<T> {
        self.db
            .get_pinned_cf(self.cf_handle(cf), key)
            .unwrap()
            .map(|pinnable_slice| scrypto_decode(pinnable_slice.as_ref()).unwrap())
    }
}

impl ConfigurableDatabase for RocksDBStore {
    fn read_flags_state(&self) -> DatabaseFlagsState {
        let account_change_index_enabled = self.get_by_key::<bool>(
            &ExtensionsData,
            ExtensionsDataKeys::AccountChangeIndexEnabled
                .to_string()
                .as_bytes(),
        );
        let local_transaction_execution_index_enabled = self.get_by_key::<bool>(
            &ExtensionsData,
            ExtensionsDataKeys::LocalTransactionExecutionIndexEnabled
                .to_string()
                .as_bytes(),
        );
        DatabaseFlagsState {
            account_change_index_enabled,
            local_transaction_execution_index_enabled,
        }
    }

    fn write_flags(&mut self, database_config: &DatabaseFlags) {
        let mut batch = WriteBatch::default();
        batch.put_cf(
            self.cf_handle(&ExtensionsData),
            ExtensionsDataKeys::AccountChangeIndexEnabled
                .to_string()
                .as_bytes(),
            scrypto_encode(&database_config.enable_account_change_index).unwrap(),
        );
        batch.put_cf(
            self.cf_handle(&ExtensionsData),
            ExtensionsDataKeys::LocalTransactionExecutionIndexEnabled
                .to_string()
                .as_bytes(),
            scrypto_encode(&database_config.enable_local_transaction_execution_index).unwrap(),
        );
        self.db
            .write(batch)
            .expect("DB error writing database config");
    }

    fn is_local_transaction_execution_index_enabled(&self) -> bool {
        self.config.enable_local_transaction_execution_index
    }

    fn is_account_change_index_enabled(&self) -> bool {
        self.config.enable_account_change_index
    }
}

impl CommitStore for RocksDBStore {
    fn commit(&mut self, commit_bundle: CommitBundle) {
        let mut batch = WriteBatch::default();

        // Check for duplicate intent/payload hashes in the commit request
        let mut user_transactions_count = 0;
        let mut processed_intent_hashes = HashSet::new();
        let transactions_count = commit_bundle.transactions.len();
        let mut processed_payload_hashes = HashSet::new();

        for txn_bundle in commit_bundle.transactions {
            if let LedgerTransaction::User(notarized_transaction) = &txn_bundle.0 {
                processed_intent_hashes.insert(notarized_transaction.intent_hash());
                user_transactions_count += 1;
            }
            processed_payload_hashes.insert(txn_bundle.0.ledger_payload_hash());
            self.add_transaction_to_write_batch(&mut batch, txn_bundle);
        }

        if processed_intent_hashes.len() != user_transactions_count {
            panic!("Commit request contains duplicate intent hashes");
        }

        if processed_payload_hashes.len() != transactions_count {
            panic!("Commit request contains duplicate payload hashes");
        }

        let commit_state_version = commit_bundle
            .proof
            .ledger_header
            .accumulator_state
            .state_version;
        let encoded_proof = scrypto_encode(&commit_bundle.proof).unwrap();
        batch.put_cf(
            self.cf_handle(&LedgerProofByStateVersion),
            commit_state_version.to_be_bytes(),
            &encoded_proof,
        );

        if let Some(next_epoch) = commit_bundle.proof.ledger_header.next_epoch {
            batch.put_cf(
                self.cf_handle(&LedgerProofByEpoch),
                next_epoch.epoch.to_be_bytes(),
                &encoded_proof,
            );
        }

        for (substate_id, substate) in commit_bundle.substate_store_update.upserted {
            batch.put_cf(
                self.cf_handle(&Substates),
                scrypto_encode(&substate_id).unwrap(),
                scrypto_encode(&substate).unwrap(),
            );
        }
        for substate_id in commit_bundle.substate_store_update.deleted_ids {
            batch.delete_cf(
                self.cf_handle(&Substates),
                scrypto_encode(&substate_id).unwrap(),
            );
        }

        if let Some(vertex_store) = commit_bundle.vertex_store {
            batch.put_cf(self.cf_handle(&VertexStore), [], vertex_store);
        }

        let state_hash_tree_update = commit_bundle.state_tree_update;
        for (key, node) in state_hash_tree_update.new_re_node_layer_nodes {
            batch.put_cf(
                self.cf_handle(&StateHashTreeNodes),
                encode_key(&key),
                scrypto_encode(&node).unwrap(),
            );
        }
        for (key, node) in state_hash_tree_update.new_substate_layer_nodes {
            batch.put_cf(
                self.cf_handle(&StateHashTreeNodes),
                encode_key(&key),
                scrypto_encode(&node).unwrap(),
            );
        }
        for stale_node_keys in state_hash_tree_update.stale_node_keys_at_state_version {
            let encoded_node_keys = stale_node_keys.1.iter().map(encode_key).collect::<Vec<_>>();
            batch.put_cf(
                self.cf_handle(&StaleStateHashTreeNodeKeysByStateVersion),
                stale_node_keys.0.to_be_bytes(),
                scrypto_encode(&encoded_node_keys).unwrap(),
            )
        }

        batch.put_cf(
            self.cf_handle(&TransactionAccuTreeSliceByStateVersion),
            commit_state_version.to_be_bytes(),
            scrypto_encode(&commit_bundle.transaction_tree_slice).unwrap(),
        );
        batch.put_cf(
            self.cf_handle(&ReceiptAccuTreeSliceByStateVersion),
            commit_state_version.to_be_bytes(),
            scrypto_encode(&commit_bundle.receipt_tree_slice).unwrap(),
        );

        self.db.write(batch).expect("Commit failed");
    }
}

pub struct RocksDBCommittedTransactionBundleIterator<'a> {
    state_version: u64,
    txns_iter: DBIteratorWithThreadMode<'a, DB>,
    ledger_receipts_iter: DBIteratorWithThreadMode<'a, DB>,
    local_executions_iter: DBIteratorWithThreadMode<'a, DB>,
    accumulator_hashes_iter: DBIteratorWithThreadMode<'a, DB>,
}

impl<'a> RocksDBCommittedTransactionBundleIterator<'a> {
    fn new(from_state_version: u64, store: &'a RocksDBStore) -> Self {
        let start_state_version_bytes = from_state_version.to_be_bytes();
        Self {
            state_version: from_state_version,
            txns_iter: store.db.iterator_cf(
                store.cf_handle(&TxnByStateVersion),
                IteratorMode::From(&start_state_version_bytes, Direction::Forward),
            ),
            ledger_receipts_iter: store.db.iterator_cf(
                store.cf_handle(&LedgerReceiptByStateVersion),
                IteratorMode::From(&start_state_version_bytes, Direction::Forward),
            ),
            local_executions_iter: store.db.iterator_cf(
                store.cf_handle(&LocalTransactionExecutionByStateVersion),
                IteratorMode::From(&start_state_version_bytes, Direction::Forward),
            ),
            accumulator_hashes_iter: store.db.iterator_cf(
                store.cf_handle(&TxnAccumulatorHashByStateVersion),
                IteratorMode::From(&start_state_version_bytes, Direction::Forward),
            ),
        }
    }
}

impl Iterator for RocksDBCommittedTransactionBundleIterator<'_> {
    type Item = CommittedTransactionBundle;

    fn next(&mut self) -> Option<Self::Item> {
        match self.txns_iter.next() {
            None => None,
            Some(txn) => {
                let txn_kv = txn.unwrap();

                let ledger_receipt_kv = self
                    .ledger_receipts_iter
                    .next()
                    .expect("Missing ledger receipt")
                    .unwrap();
                let local_execution_kv = self
                    .local_executions_iter
                    .next()
                    .expect("Missing local transaction execution")
                    .unwrap();
                let accumulator_hash_kv = self
                    .accumulator_hashes_iter
                    .next()
                    .expect("Missing txn accumulator hash")
                    .unwrap();

                for (other_key_description, other_key_bytes) in [
                    ("transaction version", txn_kv.0),
                    ("ledger receipt version", ledger_receipt_kv.0),
                    ("local execution version", local_execution_kv.0),
                    ("accumulator hash version", accumulator_hash_kv.0),
                ] {
                    let other_row_version =
                        u64::from_be_bytes((*other_key_bytes).try_into().unwrap());
                    let expected_state_version = self.state_version;
                    if other_row_version != expected_state_version {
                        panic!("DB inconsistency! {other_key_description} ({other_row_version}) doesn't match expected state version ({expected_state_version})");
                    }
                }

                let txn = manifest_decode(txn_kv.1.as_ref()).unwrap();
                let ledger_receipt = scrypto_decode(ledger_receipt_kv.1.as_ref()).unwrap();
                let local_execution = scrypto_decode(local_execution_kv.1.as_ref()).unwrap();
                let complete_receipt = LocalTransactionReceipt {
                    on_ledger: ledger_receipt,
                    local_execution,
                };

                let identifiers = CommittedTransactionIdentifiers {
                    state_version: self.state_version,
                    accumulator_hash: AccumulatorHash::from_raw_bytes(
                        (*accumulator_hash_kv.1).try_into().unwrap(),
                    ),
                };

                self.state_version += 1;

                Some((txn, complete_receipt, identifiers))
            }
        }
    }
}

impl IterableTransactionStore for RocksDBStore {
    type CommittedTransactionBundleIterator<'a> = RocksDBCommittedTransactionBundleIterator<'a>;

    fn get_committed_transaction_bundle_iter(
        &self,
        from_state_version: u64,
    ) -> Self::CommittedTransactionBundleIterator<'_> {
        debug_assert!(self.is_local_transaction_execution_index_enabled());

        RocksDBCommittedTransactionBundleIterator::new(from_state_version, self)
    }
}

impl QueryableTransactionStore for RocksDBStore {
    fn get_committed_transaction(&self, state_version: u64) -> Option<LedgerTransaction> {
        self.db
            .get_cf(
                self.cf_handle(&TxnByStateVersion),
                state_version.to_be_bytes(),
            )
            .expect("DB error loading transaction")
            .map(|v| manifest_decode(&v).expect("Failed to decode a committed transaction"))
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: u64,
    ) -> Option<CommittedTransactionIdentifiers> {
        self.db
            .get_cf(
                self.cf_handle(&TxnAccumulatorHashByStateVersion),
                state_version.to_be_bytes(),
            )
            .expect("DB error loading transaction")
            .map(|v| CommittedTransactionIdentifiers {
                state_version,
                accumulator_hash: AccumulatorHash::from_raw_bytes(
                    v.try_into()
                        .expect("Failed to decode a committed transaction accumulator hash"),
                ),
            })
    }

    fn get_committed_ledger_transaction_receipt(
        &self,
        state_version: u64,
    ) -> Option<LedgerTransactionReceipt> {
        self.get_by_key(&LedgerReceiptByStateVersion, &state_version.to_be_bytes())
    }

    fn get_committed_local_transaction_execution(
        &self,
        state_version: u64,
    ) -> Option<LocalTransactionExecution> {
        self.get_by_key(
            &LocalTransactionExecutionByStateVersion,
            &state_version.to_be_bytes(),
        )
    }

    fn get_committed_local_transaction_receipt(
        &self,
        state_version: u64,
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
    fn get_txn_state_version_by_identifier(&self, identifier: &IntentHash) -> Option<u64> {
        self.db
            .get_cf(self.cf_handle(&StateVersionByTxnIntentHash), identifier)
            .expect("DB error reading state version for intent hash")
            .map(|b| u64::from_be_bytes(b.try_into().unwrap()))
    }
}

impl TransactionIndex<&UserPayloadHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(&self, identifier: &UserPayloadHash) -> Option<u64> {
        self.db
            .get_cf(
                self.cf_handle(&StateVersionByTxnUserPayloadHash),
                identifier,
            )
            .expect("DB error reading state version for user payload hash")
            .map(|b| u64::from_be_bytes(b.try_into().unwrap()))
    }
}

impl TransactionIndex<&LedgerPayloadHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(&self, identifier: &LedgerPayloadHash) -> Option<u64> {
        self.db
            .get_cf(
                self.cf_handle(&StateVersionByTxnLedgerPayloadHash),
                identifier,
            )
            .expect("DB error reading state version for ledger payload hash")
            .map(|b| u64::from_be_bytes(b.try_into().unwrap()))
    }
}

impl TransactionIdentifierLoader for RocksDBStore {
    fn get_top_transaction_identifiers(&self) -> CommittedTransactionIdentifiers {
        self.db
            .iterator_cf(
                self.cf_handle(&TxnAccumulatorHashByStateVersion),
                IteratorMode::End,
            )
            .map(|res| res.unwrap())
            .next()
            .map(|(key, value)| CommittedTransactionIdentifiers {
                state_version: u64::from_be_bytes((*key).try_into().unwrap()),
                accumulator_hash: AccumulatorHash::from_raw_bytes((*value).try_into().unwrap()),
            })
            .unwrap_or_else(CommittedTransactionIdentifiers::pre_genesis)
    }
}

impl QueryableProofStore for RocksDBStore {
    fn max_state_version(&self) -> u64 {
        self.db
            .iterator_cf(self.cf_handle(&TxnByStateVersion), IteratorMode::End)
            .next()
            .map(|res| res.unwrap())
            .map(|(key, _)| u64::from_be_bytes((*key).try_into().unwrap()))
            .unwrap_or(0)
    }

    fn get_txns_and_proof(
        &self,
        start_state_version_inclusive: u64,
        max_number_of_txns_if_more_than_one_proof: u32,
        max_payload_size_in_bytes: u32,
    ) -> Option<(Vec<Vec<u8>>, LedgerProof)> {
        let mut payload_size_so_far = 0;
        let mut latest_usable_proof: Option<LedgerProof> = None;
        let mut txns = Vec::new();

        let mut proofs_iter = self.db.iterator_cf(
            self.cf_handle(&LedgerProofByStateVersion),
            IteratorMode::From(
                &start_state_version_inclusive.to_be_bytes(),
                Direction::Forward,
            ),
        );

        let mut txns_iter = self.db.iterator_cf(
            self.cf_handle(&TxnByStateVersion),
            IteratorMode::From(
                &start_state_version_inclusive.to_be_bytes(),
                Direction::Forward,
            ),
        );

        'proof_loop: while payload_size_so_far <= max_payload_size_in_bytes
            && txns.len() <= (max_number_of_txns_if_more_than_one_proof as usize)
        {
            // Fetch next proof and see if all txns it includes can fit
            // If they do - add them to the output and update the latest usable proof then continue the iteration
            // If they don't - (sadly) ignore this proof's txns read so far and break the loop
            // If we're out of proofs (or some txns are missing): also break the loop
            match proofs_iter.next() {
                Some(next_proof_result) => {
                    let next_proof_kv = next_proof_result.unwrap();
                    let next_proof_state_version =
                        u64::from_be_bytes((*next_proof_kv.0).try_into().unwrap());
                    let next_proof: LedgerProof = scrypto_decode(next_proof_kv.1.as_ref()).unwrap();

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
                            Some(next_txn_result) => {
                                let next_txn_kv = next_txn_result.unwrap();
                                let next_txn_state_version =
                                    u64::from_be_bytes((*next_txn_kv.0).try_into().unwrap());
                                let next_txn_payload = next_txn_kv.1.to_vec();

                                payload_size_including_next_proof_txns +=
                                    next_txn_payload.len() as u32;
                                next_proof_txns.push(next_txn_payload);

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

    fn get_epoch_proof(&self, epoch: u64) -> Option<LedgerProof> {
        self.db
            .get_cf(self.cf_handle(&LedgerProofByEpoch), epoch.to_be_bytes())
            .unwrap()
            .map(|bytes| scrypto_decode(bytes.as_ref()).unwrap())
    }

    fn get_last_proof(&self) -> Option<LedgerProof> {
        self.get_last(&LedgerProofByStateVersion)
    }

    fn get_last_epoch_proof(&self) -> Option<LedgerProof> {
        self.get_last(&LedgerProofByEpoch)
    }
}

impl ReadableSubstateStore for RocksDBStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.get_by_key(&Substates, &scrypto_encode(substate_id).unwrap())
    }
}

impl<P: Payload> ReadableTreeStore<P> for RocksDBStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        self.get_by_key(&StateHashTreeNodes, &encode_key(key))
    }
}

impl ReadableAccuTreeStore<u64, TransactionTreeHash> for RocksDBStore {
    fn get_tree_slice(&self, state_version: &u64) -> Option<TreeSlice<TransactionTreeHash>> {
        self.get_by_key(
            &TransactionAccuTreeSliceByStateVersion,
            &state_version.to_be_bytes(),
        )
    }
}

impl ReadableAccuTreeStore<u64, ReceiptTreeHash> for RocksDBStore {
    fn get_tree_slice(&self, state_version: &u64) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.get_by_key(
            &ReceiptAccuTreeSliceByStateVersion,
            &state_version.to_be_bytes(),
        )
    }
}

impl QueryableSubstateStore for RocksDBStore {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        let id = scrypto_encode(&SubstateId(
            RENodeId::KeyValueStore(*kv_store_id),
            NodeModuleId::SELF,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(vec![])),
        ))
        .unwrap();

        let iter = self.db.iterator_cf(
            self.cf_handle(&Substates),
            IteratorMode::From(&id, Direction::Forward),
        );
        let mut items = HashMap::new();
        for res in iter {
            let (key, value) = res.unwrap();
            let substate: OutputValue = scrypto_decode(&value).unwrap();
            let substate_id: SubstateId = scrypto_decode(&key).unwrap();
            if let SubstateId(
                RENodeId::KeyValueStore(id),
                NodeModuleId::SELF,
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) = substate_id
            {
                if id == *kv_store_id {
                    items.insert(key, substate.substate)
                } else {
                    break;
                }
            } else {
                break;
            };
        }
        items
    }
}

impl WriteableVertexStore for RocksDBStore {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>) {
        self.db
            .put_cf(self.cf_handle(&VertexStore), [], vertex_store_bytes)
            .unwrap();
    }
}

impl RecoverableVertexStore for RocksDBStore {
    fn get_vertex_store(&self) -> Option<Vec<u8>> {
        self.db.get_cf(self.cf_handle(&VertexStore), []).unwrap()
    }
}

impl RocksDBStore {
    fn batch_update_account_change_index_from_receipt(
        &self,
        batch: &mut WriteBatch,
        state_version: u64,
        receipt: &LocalTransactionReceipt,
    ) {
        for (address, _) in receipt
            .local_execution
            .state_update_summary
            .balance_changes
            .iter()
        {
            if !address.is_account() {
                continue;
            }
            let mut key = address.to_vec();
            key.extend(state_version.to_be_bytes());
            batch.put_cf(self.cf_handle(&AccountChangeStateVersions), key, []);
        }
    }

    fn batch_update_account_change_index_from_committed_transaction(
        &self,
        batch: &mut WriteBatch,
        transaction_bundle: &CommittedTransactionBundle,
    ) {
        let state_version = transaction_bundle.2.state_version;

        self.batch_update_account_change_index_from_receipt(
            batch,
            state_version,
            &transaction_bundle.1,
        );

        batch.put_cf(
            self.cf_handle(&ExtensionsData),
            ExtensionsDataKeys::AccountChangeIndexLastProcessedStateVersion
                .to_string()
                .as_bytes(),
            state_version.to_be_bytes(),
        );
    }

    fn update_account_change_index_from_store(
        &mut self,
        start_state_version_inclusive: u64,
        limit: u64,
    ) -> u64 {
        let start_state_version_bytes = start_state_version_inclusive.to_be_bytes();
        let mut receipts_iter = self.db.iterator_cf(
            self.cf_handle(&LocalTransactionExecutionByStateVersion),
            IteratorMode::From(&start_state_version_bytes, Direction::Forward),
        );

        let mut batch = WriteBatch::default();

        let mut last_state_version = start_state_version_inclusive;
        let mut index = 0;
        while index < limit {
            match receipts_iter.next() {
                Some(next_receipt_result) => {
                    let next_receipt_kv = next_receipt_result.unwrap();

                    let next_receipt_state_version =
                        u64::from_be_bytes((*next_receipt_kv.0).try_into().unwrap());

                    let expected_state_version = start_state_version_inclusive + index;
                    if expected_state_version != next_receipt_state_version {
                        panic!(
                            "DB inconsistency! Missing receipt at state version {expected_state_version}"
                        );
                    }
                    last_state_version = expected_state_version;

                    let next_receipt: LocalTransactionReceipt =
                        scrypto_decode(next_receipt_kv.1.as_ref()).unwrap();
                    self.batch_update_account_change_index_from_receipt(
                        &mut batch,
                        last_state_version,
                        &next_receipt,
                    );

                    index += 1;
                }
                None => {
                    break;
                }
            }
        }

        batch.put_cf(
            self.cf_handle(&ExtensionsData),
            ExtensionsDataKeys::AccountChangeIndexLastProcessedStateVersion
                .to_string()
                .as_bytes(),
            last_state_version.to_be_bytes(),
        );

        self.db
            .write(batch)
            .expect("Account change index build failed");

        last_state_version
    }
}

impl AccountChangeIndexExtension for RocksDBStore {
    fn account_change_index_last_processed_state_version(&self) -> u64 {
        self.db
            .get_pinned_cf(
                self.cf_handle(&ExtensionsData),
                ExtensionsDataKeys::AccountChangeIndexLastProcessedStateVersion
                    .to_string()
                    .as_bytes(),
            )
            .unwrap()
            .map(|pinnable_slice| u64::from_be_bytes(pinnable_slice.as_ref().try_into().unwrap()))
            .unwrap_or(0)
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
                last_processed_state_version + 1,
                MAX_TRANSACTION_BATCH,
            );
            info!("Account Change Index updated to {last_processed_state_version}/{last_state_version}");
        }

        info!("Account Change Index catchup done!");
    }
}

pub struct RocksDBAccountChangeIndexIterator<'a> {
    account_bytes: Vec<u8>,
    account_change_iter: DBIteratorWithThreadMode<'a, DB>,
}

impl<'a> RocksDBAccountChangeIndexIterator<'a> {
    fn new(from_state_version: u64, account: Address, store: &'a RocksDBStore) -> Self {
        let mut key = account.to_vec();
        key.extend(from_state_version.to_be_bytes());
        Self {
            account_bytes: account.to_vec(),
            account_change_iter: store.db.iterator_cf(
                store.cf_handle(&AccountChangeStateVersions),
                IteratorMode::From(&key, Direction::Forward),
            ),
        }
    }
}

impl Iterator for RocksDBAccountChangeIndexIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        match self.account_change_iter.next() {
            Some(entry) => {
                let (key, _value) = entry.unwrap();
                let (address_bytes, state_version_bytes) =
                    key.split_at(key.len() - size_of::<u64>());
                let state_version = u64::from_be_bytes(state_version_bytes.try_into().unwrap());
                if address_bytes != self.account_bytes {
                    None
                } else {
                    Some(state_version)
                }
            }
            None => None,
        }
    }
}

impl IterableAccountChangeIndex for RocksDBStore {
    type AccountChangeIndexIterator<'a> = RocksDBAccountChangeIndexIterator<'a>;

    fn get_state_versions_for_account_iter(
        &self,
        account: Address,
        from_state_version: u64,
    ) -> Self::AccountChangeIndexIterator<'_> {
        RocksDBAccountChangeIndexIterator::new(from_state_version, account, self)
    }
}
