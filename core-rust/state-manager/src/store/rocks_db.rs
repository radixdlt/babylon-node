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
    TransactionTreeHash,
};
use node_common::utils::IsAccountExt;
use radix_engine::types::*;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use radix_engine_stores::hash_tree::tree_store::{
    encode_key, NodeKey, ReadableTreeStore, TreeNode,
};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBIteratorWithThreadMode, Direction, IteratorMode,
    Options, WriteBatch, DB,
};
use transaction::model::*;

use radix_engine_store_interface::interface::*;

use std::path::PathBuf;

use tracing::{error, info};

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use crate::transaction::{
    LedgerTransactionHash, RawLedgerTransaction, TypedTransactionIdentifiers,
};

use super::traits::extensions::*;

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
enum RocksDBColumnFamily {
    /// Committed transactions
    TxnByStateVersion,
    TxnIdentifiersByStateVersion,
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
    /// Ancestor information for the [`Substates`]' RE Nodes (which is useful and can be computed,
    /// but is not provided by the Engine itself).
    /// Schema: `NodeId.0` -> `scrypto_encode(SubstateNodeAncestryRecord)`
    /// Note: we do not persist records of root Nodes (which do not have any ancestor).
    SubstateNodeAncestryRecords,
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
    /// Additional details of "Scenarios" (and their transactions) executed as part of Genesis,
    /// keyed by their sequence number (i.e. their index in the list of Scenarios to execute).
    /// Schema: `ScenarioSequenceNumber.to_be_byte()` -> `scrypto_encode(ExecutedGenesisScenario)`
    ExecutedGenesisScenarios,
}

use crate::store::traits::scenario::{
    ExecutedGenesisScenario, ExecutedGenesisScenarioStore, ScenarioSequenceNumber,
};
use RocksDBColumnFamily::*;

const ALL_COLUMN_FAMILIES: [RocksDBColumnFamily; 19] = [
    TxnByStateVersion,
    TxnIdentifiersByStateVersion,
    LedgerReceiptByStateVersion,
    LocalTransactionExecutionByStateVersion,
    StateVersionByTxnIntentHash,
    StateVersionByTxnUserPayloadHash,
    StateVersionByTxnLedgerPayloadHash,
    LedgerProofByStateVersion,
    LedgerProofByEpoch,
    Substates,
    SubstateNodeAncestryRecords,
    VertexStore,
    StateHashTreeNodes,
    StaleStateHashTreeNodeKeysByStateVersion,
    TransactionAccuTreeSliceByStateVersion,
    ReceiptAccuTreeSliceByStateVersion,
    ExtensionsData,
    AccountChangeStateVersions,
    ExecutedGenesisScenarios,
];

impl fmt::Display for RocksDBColumnFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            TxnByStateVersion => "txn_by_state_version",
            TxnIdentifiersByStateVersion => "txn_identifiers_by_state_version",
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
            SubstateNodeAncestryRecords => "substate_node_ancestry_records",
            VertexStore => "vertex_store",
            StateHashTreeNodes => "state_hash_tree_nodes",
            StaleStateHashTreeNodeKeysByStateVersion => "stale_state_hash_tree_node_keys",
            TransactionAccuTreeSliceByStateVersion => {
                "transaction_accu_tree_slice_by_state_version"
            }
            ReceiptAccuTreeSliceByStateVersion => "receipt_accu_tree_slice_by_state_version",
            ExtensionsData => "extensions_data",
            AccountChangeStateVersions => "account_change_state_versions",
            ExecutedGenesisScenarios => "executed_genesis_scenarios",
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
        let ledger_payload_hash = identifiers.payload.ledger_payload_hash;

        // TEMPORARY until this is handled in the engine: we store both an intent lookup and the transaction itself
        if let TypedTransactionIdentifiers::User {
            intent_hash,
            notarized_transaction_hash,
            ..
        } = &identifiers.payload.typed
        {
            /* For user transactions we only need to check for duplicate intent hashes to know
            that user payload hash and ledger payload hash are also unique. */

            let maybe_existing_state_version = self
                .db
                .get_cf(self.cf_handle(&StateVersionByTxnIntentHash), intent_hash)
                .unwrap();

            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save intent hash {:?} which already exists at state version {:?}",
                    intent_hash,
                    StateVersion::from_bytes(existing_state_version)
                );
            }

            batch.put_cf(
                self.cf_handle(&StateVersionByTxnIntentHash),
                intent_hash,
                state_version.to_bytes(),
            );

            batch.put_cf(
                self.cf_handle(&StateVersionByTxnUserPayloadHash),
                notarized_transaction_hash,
                state_version.to_bytes(),
            );
        } else {
            let maybe_existing_state_version = self
                .db
                .get_cf(
                    self.cf_handle(&StateVersionByTxnLedgerPayloadHash),
                    ledger_payload_hash,
                )
                .unwrap();

            if let Some(existing_state_version) = maybe_existing_state_version {
                panic!(
                    "Attempted to save ledger payload hash {:?} which already exists at state version {:?}",
                    ledger_payload_hash,
                    StateVersion::from_bytes(existing_state_version)
                );
            }
        }

        batch.put_cf(
            self.cf_handle(&StateVersionByTxnLedgerPayloadHash),
            ledger_payload_hash,
            state_version.to_bytes(),
        );

        batch.put_cf(
            self.cf_handle(&TxnByStateVersion),
            state_version.to_bytes(),
            &raw.0,
        );

        batch.put_cf(
            self.cf_handle(&TxnIdentifiersByStateVersion),
            state_version.to_bytes(),
            scrypto_encode(&identifiers).unwrap(),
        );

        batch.put_cf(
            self.cf_handle(&LedgerReceiptByStateVersion),
            state_version.to_bytes(),
            scrypto_encode(&receipt.on_ledger).unwrap(),
        );

        if self.is_local_transaction_execution_index_enabled() {
            batch.put_cf(
                self.cf_handle(&LocalTransactionExecutionByStateVersion),
                state_version.to_bytes(),
                scrypto_encode(&receipt.local_execution).unwrap(),
            );
        }
    }

    fn cf_handle(&self, cf: &RocksDBColumnFamily) -> &ColumnFamily {
        self.db.cf_handle(&cf.to_string()).unwrap()
    }

    fn get_first<T: ScryptoDecode>(&self, cf: &RocksDBColumnFamily) -> Option<T> {
        self.db
            .iterator_cf(self.cf_handle(cf), IteratorMode::Start)
            .map(|res| res.unwrap())
            .next()
            .map(|(_, value)| scrypto_decode(value.as_ref()).unwrap())
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
        let mut processed_payload_hashes = HashSet::new();

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
            processed_payload_hashes.insert(payload_identifiers.ledger_payload_hash);
            self.add_transaction_to_write_batch(&mut batch, transaction_bundle);
        }

        if processed_intent_hashes.len() != user_transactions_count {
            panic!("Commit request contains duplicate intent hashes");
        }

        if processed_payload_hashes.len() != transactions_count {
            panic!("Commit request contains duplicate payload hashes");
        }

        let encoded_proof = scrypto_encode(&commit_bundle.proof).unwrap();
        batch.put_cf(
            self.cf_handle(&LedgerProofByStateVersion),
            commit_state_version.to_bytes(),
            &encoded_proof,
        );

        if let Some(next_epoch) = &commit_ledger_header.next_epoch {
            batch.put_cf(
                self.cf_handle(&LedgerProofByEpoch),
                next_epoch.epoch.number().to_be_bytes(),
                &encoded_proof,
            );
        }

        for (node_key, node_updates) in commit_bundle.substate_store_update.updates.node_updates {
            for (partition_num, partition_updates) in node_updates.partition_updates {
                let partition_key = DbPartitionKey {
                    node_key: node_key.clone(),
                    partition_num,
                };
                match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        for (sort_key, update) in substate_updates {
                            let key_bytes = encode_to_rocksdb_bytes(&partition_key, &sort_key);
                            match update {
                                DatabaseUpdate::Set(value_bytes) => {
                                    batch.put_cf(self.cf_handle(&Substates), key_bytes, value_bytes)
                                }
                                DatabaseUpdate::Delete => {
                                    batch.delete_cf(self.cf_handle(&Substates), key_bytes)
                                }
                            }
                        }
                    }
                    PartitionDatabaseUpdates::Batch(batch_update) => match batch_update {
                        BatchPartitionDatabaseUpdate::Reset {
                            new_substate_values,
                        } => {
                            let empty_key = DbSortKey(vec![]);
                            batch.delete_range_cf(
                                self.cf_handle(&Substates),
                                encode_to_rocksdb_bytes(&partition_key, &empty_key),
                                encode_to_rocksdb_bytes(&partition_key.next(), &empty_key),
                            );
                            for (sort_key, value_bytes) in new_substate_values {
                                batch.put_cf(
                                    self.cf_handle(&Substates),
                                    encode_to_rocksdb_bytes(&partition_key, &sort_key),
                                    value_bytes,
                                );
                            }
                        }
                    },
                }
            }
        }

        if let Some(vertex_store) = commit_bundle.vertex_store {
            batch.put_cf(self.cf_handle(&VertexStore), [], vertex_store);
        }

        let state_hash_tree_update = commit_bundle.state_tree_update;
        for (key, node) in state_hash_tree_update.new_nodes {
            batch.put_cf(
                self.cf_handle(&StateHashTreeNodes),
                encode_key(&key),
                scrypto_encode(&node).unwrap(),
            );
        }
        for (version, stale_parts) in state_hash_tree_update.stale_tree_parts_at_state_version {
            batch.put_cf(
                self.cf_handle(&StaleStateHashTreeNodeKeysByStateVersion),
                version.to_bytes(),
                scrypto_encode(&stale_parts).unwrap(),
            )
        }

        for (node_ids, record) in commit_bundle.new_substate_node_ancestry_records {
            let encoded_record = scrypto_encode(&record).unwrap();
            for node_id in node_ids {
                batch.put_cf(
                    self.cf_handle(&SubstateNodeAncestryRecords),
                    node_id.0,
                    &encoded_record,
                );
            }
        }

        batch.put_cf(
            self.cf_handle(&TransactionAccuTreeSliceByStateVersion),
            commit_state_version.to_bytes(),
            scrypto_encode(&commit_bundle.transaction_tree_slice).unwrap(),
        );
        batch.put_cf(
            self.cf_handle(&ReceiptAccuTreeSliceByStateVersion),
            commit_state_version.to_bytes(),
            scrypto_encode(&commit_bundle.receipt_tree_slice).unwrap(),
        );

        self.db.write(batch).expect("Commit failed");
    }
}

impl ExecutedGenesisScenarioStore for RocksDBStore {
    fn put_scenario(&mut self, number: ScenarioSequenceNumber, scenario: ExecutedGenesisScenario) {
        self.db
            .put_cf(
                self.cf_handle(&ExecutedGenesisScenarios),
                number.to_be_bytes(),
                scrypto_encode(&scenario).unwrap(),
            )
            .expect("Executed scenario write failed");
    }

    fn list_all_scenarios(&self) -> Vec<(ScenarioSequenceNumber, ExecutedGenesisScenario)> {
        self.db
            .iterator_cf(
                self.cf_handle(&ExecutedGenesisScenarios),
                IteratorMode::Start,
            )
            .map(|result| result.unwrap())
            .map(|kv| {
                (
                    u32::from_be_bytes(kv.0.as_ref().try_into().unwrap()),
                    scrypto_decode(kv.1.as_ref()).unwrap(),
                )
            })
            .collect()
    }
}

pub struct RocksDBCommittedTransactionBundleIterator<'a> {
    state_version: StateVersion,
    txns_iter: DBIteratorWithThreadMode<'a, DB>,
    ledger_receipts_iter: DBIteratorWithThreadMode<'a, DB>,
    local_executions_iter: DBIteratorWithThreadMode<'a, DB>,
    identifiers_iter: DBIteratorWithThreadMode<'a, DB>,
}

impl<'a> RocksDBCommittedTransactionBundleIterator<'a> {
    fn new(from_state_version: StateVersion, store: &'a RocksDBStore) -> Self {
        let start_state_version_bytes = from_state_version.to_bytes();
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
            identifiers_iter: store.db.iterator_cf(
                store.cf_handle(&TxnIdentifiersByStateVersion),
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
                let identifiers_kv = self
                    .identifiers_iter
                    .next()
                    .expect("Missing txn hashes")
                    .unwrap();

                let current_state_version = self.state_version;
                for (other_key_description, other_key_bytes) in [
                    ("transaction version", txn_kv.0),
                    ("ledger receipt version", ledger_receipt_kv.0),
                    ("local execution version", local_execution_kv.0),
                    ("identifiers version", identifiers_kv.0),
                ] {
                    let other_row_version = StateVersion::from_bytes(other_key_bytes);
                    if other_row_version != current_state_version {
                        panic!("DB inconsistency! {other_key_description} ({other_row_version}) doesn't match expected state version ({current_state_version})");
                    }
                }

                let txn = RawLedgerTransaction(txn_kv.1.to_vec());
                let ledger_receipt = scrypto_decode(ledger_receipt_kv.1.as_ref()).unwrap();
                let local_execution = scrypto_decode(local_execution_kv.1.as_ref()).unwrap();
                let complete_receipt = LocalTransactionReceipt {
                    on_ledger: ledger_receipt,
                    local_execution,
                };
                let identifiers = scrypto_decode(identifiers_kv.1.as_ref()).unwrap();

                self.state_version = self
                    .state_version
                    .next()
                    .expect("Invalid next state version!");

                Some(CommittedTransactionBundle {
                    state_version: current_state_version,
                    raw: txn,
                    receipt: complete_receipt,
                    identifiers,
                })
            }
        }
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
        self.db
            .get_cf(self.cf_handle(&TxnByStateVersion), state_version.to_bytes())
            .expect("DB error loading transaction")
            .map(RawLedgerTransaction)
    }

    fn get_committed_transaction_identifiers(
        &self,
        state_version: StateVersion,
    ) -> Option<CommittedTransactionIdentifiers> {
        self.db
            .get_cf(
                self.cf_handle(&TxnIdentifiersByStateVersion),
                state_version.to_bytes(),
            )
            .expect("DB error loading identifiers")
            .map(|v| scrypto_decode(&v).expect("Failed to decode identifiers"))
    }

    fn get_committed_ledger_transaction_receipt(
        &self,
        state_version: StateVersion,
    ) -> Option<LedgerTransactionReceipt> {
        self.get_by_key(&LedgerReceiptByStateVersion, &state_version.to_bytes())
    }

    fn get_committed_local_transaction_execution(
        &self,
        state_version: StateVersion,
    ) -> Option<LocalTransactionExecution> {
        self.get_by_key(
            &LocalTransactionExecutionByStateVersion,
            &state_version.to_bytes(),
        )
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
    fn get_txn_state_version_by_identifier(&self, identifier: &IntentHash) -> Option<StateVersion> {
        self.db
            .get_cf(self.cf_handle(&StateVersionByTxnIntentHash), identifier)
            .expect("DB error reading state version for intent hash")
            .map(StateVersion::from_bytes)
    }
}

impl TransactionIndex<&NotarizedTransactionHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        identifier: &NotarizedTransactionHash,
    ) -> Option<StateVersion> {
        self.db
            .get_cf(
                self.cf_handle(&StateVersionByTxnUserPayloadHash),
                identifier,
            )
            .expect("DB error reading state version for user payload hash")
            .map(StateVersion::from_bytes)
    }
}

impl TransactionIndex<&LedgerTransactionHash> for RocksDBStore {
    fn get_txn_state_version_by_identifier(
        &self,
        identifier: &LedgerTransactionHash,
    ) -> Option<StateVersion> {
        self.db
            .get_cf(
                self.cf_handle(&StateVersionByTxnLedgerPayloadHash),
                identifier,
            )
            .expect("DB error reading state version for ledger payload hash")
            .map(StateVersion::from_bytes)
    }
}

impl TransactionIdentifierLoader for RocksDBStore {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)> {
        self.db
            .iterator_cf(
                self.cf_handle(&TxnIdentifiersByStateVersion),
                IteratorMode::End,
            )
            .map(|res| res.unwrap())
            .next()
            .map(|(key, value)| {
                (
                    StateVersion::from_bytes(key),
                    scrypto_decode(&value).expect("Failed to decode identifiers"),
                )
            })
    }
}

impl QueryableProofStore for RocksDBStore {
    fn max_state_version(&self) -> StateVersion {
        self.db
            .iterator_cf(self.cf_handle(&TxnByStateVersion), IteratorMode::End)
            .next()
            .map(|res| res.unwrap())
            .map(|(key, _)| StateVersion::from_bytes(key))
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

        let mut proofs_iter = self.db.iterator_cf(
            self.cf_handle(&LedgerProofByStateVersion),
            IteratorMode::From(
                &start_state_version_inclusive.to_bytes(),
                Direction::Forward,
            ),
        );

        let mut txns_iter = self.db.iterator_cf(
            self.cf_handle(&TxnByStateVersion),
            IteratorMode::From(
                &start_state_version_inclusive.to_bytes(),
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
                    let next_proof_state_version = StateVersion::from_bytes(next_proof_kv.0);
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
                                    StateVersion::from_bytes(next_txn_kv.0);
                                let next_txn_payload = next_txn_kv.1.to_vec();

                                payload_size_including_next_proof_txns +=
                                    next_txn_payload.len() as u32;
                                next_proof_txns.push(RawLedgerTransaction(next_txn_payload));

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
        self.get_first(&LedgerProofByStateVersion)
    }

    fn get_post_genesis_epoch_proof(&self) -> Option<LedgerProof> {
        self.get_first(&LedgerProofByEpoch)
    }

    fn get_epoch_proof(&self, epoch: Epoch) -> Option<LedgerProof> {
        self.db
            .get_cf(
                self.cf_handle(&LedgerProofByEpoch),
                epoch.number().to_be_bytes(),
            )
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

impl SubstateDatabase for RocksDBStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let encoded_key_bytes = encode_to_rocksdb_bytes(partition_key, sort_key);
        self.db
            .get_cf(self.cf_handle(&Substates), encoded_key_bytes)
            .unwrap()
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        let start = encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![]));
        let iter = self
            .db
            .iterator_cf(
                self.cf_handle(&Substates),
                IteratorMode::From(&start, Direction::Forward),
            )
            .map(|kv| {
                let (k, v) = kv.unwrap();
                let (partition_key, sort_key) = decode_from_rocksdb_bytes(k.as_ref());
                ((partition_key, sort_key), v)
            })
            .take_while(move |((next_partition_key, _), _)| next_partition_key.eq(&partition_key))
            .map(|((_, sort_key), value)| (sort_key, value.as_ref().to_vec()));

        Box::new(iter)
    }
}

impl SubstateNodeAncestryStore for RocksDBStore {
    fn batch_get_ancestry<'a>(
        &self,
        node_ids: impl IntoIterator<Item = &'a NodeId>,
    ) -> Vec<Option<SubstateNodeAncestryRecord>> {
        self.db
            .multi_get_cf(
                node_ids
                    .into_iter()
                    .map(|node_id| (self.cf_handle(&SubstateNodeAncestryRecords), node_id.0)),
            )
            .into_iter()
            .map(|result| {
                result
                    .unwrap()
                    .map(|bytes| scrypto_decode::<SubstateNodeAncestryRecord>(&bytes).unwrap())
            })
            .collect()
    }
}

impl ReadableTreeStore for RocksDBStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.get_by_key(&StateHashTreeNodes, &encode_key(key))
    }
}

impl ReadableAccuTreeStore<StateVersion, TransactionTreeHash> for RocksDBStore {
    fn get_tree_slice(
        &self,
        state_version: &StateVersion,
    ) -> Option<TreeSlice<TransactionTreeHash>> {
        self.get_by_key(
            &TransactionAccuTreeSliceByStateVersion,
            &state_version.to_bytes(),
        )
    }
}

impl ReadableAccuTreeStore<StateVersion, ReceiptTreeHash> for RocksDBStore {
    fn get_tree_slice(&self, state_version: &StateVersion) -> Option<TreeSlice<ReceiptTreeHash>> {
        self.get_by_key(
            &ReceiptAccuTreeSliceByStateVersion,
            &state_version.to_bytes(),
        )
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
        receipt: &LocalTransactionReceipt,
    ) {
        for address in receipt
            .local_execution
            .global_balance_summary
            .global_balance_changes
            .keys()
        {
            if !address.is_account() {
                continue;
            }
            let mut key = address.to_vec();
            key.extend(state_version.to_bytes());
            batch.put_cf(self.cf_handle(&AccountChangeStateVersions), key, []);
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
            &transaction_bundle.receipt,
        );

        batch.put_cf(
            self.cf_handle(&ExtensionsData),
            ExtensionsDataKeys::AccountChangeIndexLastProcessedStateVersion
                .to_string()
                .as_bytes(),
            state_version.to_bytes(),
        );
    }

    fn update_account_change_index_from_store(
        &mut self,
        start_state_version_inclusive: StateVersion,
        limit: u64,
    ) -> StateVersion {
        let start_state_version_bytes = start_state_version_inclusive.to_bytes();
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
                    let next_receipt_state_version = StateVersion::from_bytes(next_receipt_kv.0);

                    let expected_state_version = start_state_version_inclusive
                        .relative(index)
                        .expect("Invalid relative state version!");
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
            last_state_version.to_bytes(),
        );

        self.db
            .write(batch)
            .expect("Account change index build failed");

        last_state_version
    }
}

impl AccountChangeIndexExtension for RocksDBStore {
    fn account_change_index_last_processed_state_version(&self) -> StateVersion {
        self.db
            .get_pinned_cf(
                self.cf_handle(&ExtensionsData),
                ExtensionsDataKeys::AccountChangeIndexLastProcessedStateVersion
                    .to_string()
                    .as_bytes(),
            )
            .unwrap()
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

pub struct RocksDBAccountChangeIndexIterator<'a> {
    account_bytes: Vec<u8>,
    account_change_iter: DBIteratorWithThreadMode<'a, DB>,
}

impl<'a> RocksDBAccountChangeIndexIterator<'a> {
    fn new(
        from_state_version: StateVersion,
        account: GlobalAddress,
        store: &'a RocksDBStore,
    ) -> Self {
        let mut key = account.to_vec();
        key.extend(from_state_version.to_bytes());
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
    type Item = StateVersion;

    fn next(&mut self) -> Option<StateVersion> {
        match self.account_change_iter.next() {
            Some(entry) => {
                let (key, _value) = entry.unwrap();
                let (address_bytes, state_version_bytes) =
                    key.split_at(key.len() - StateVersion::BYTE_LEN);
                let state_version = StateVersion::from_bytes(state_version_bytes);
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
    fn get_state_versions_for_account_iter(
        &self,
        account: GlobalAddress,
        from_state_version: StateVersion,
    ) -> Box<dyn Iterator<Item = StateVersion> + '_> {
        Box::new(RocksDBAccountChangeIndexIterator::new(
            from_state_version,
            account,
            self,
        ))
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
