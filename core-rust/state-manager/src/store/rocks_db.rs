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
use std::collections::HashSet;
use std::fmt;

use crate::store::traits::*;
use crate::{
    AccumulatorHash, ChangeAction, CommittedTransactionIdentifiers, HasIntentHash,
    HasLedgerPayloadHash, HasUserPayloadHash, IntentHash, LedgerPayloadHash, LedgerProof,
    LocalTransactionReceipt, ReceiptTreeHash, TransactionTreeHash,
};
use radix_engine::types::{scrypto_decode, scrypto_encode};
use radix_engine_interface::data::manifest::manifest_decode;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use radix_engine_stores::hash_tree::tree_store::{
    encode_key, NodeKey, Payload, ReadableTreeStore, TreeNode,
};
use radix_engine_stores::interface::{decode_substate_id, encode_substate_id, DatabaseMapper, SubstateDatabase, DatabaseUpdate};
use radix_engine_stores::jmt_support::JmtMapper;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, Direction, IteratorMode, Options, WriteBatch, DB,
};
use std::path::PathBuf;
use tracing::{error, warn};
use utils::copy_u8_array;

use crate::transaction::LedgerTransaction;

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
}

use crate::accumulator_tree::storage::{ReadableAccuTreeStore, TreeSlice};
use crate::query::TransactionIdentifierLoader;
use RocksDBColumnFamily::*;

const ALL_COLUMN_FAMILIES: [RocksDBColumnFamily; 15] = [
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
        };
        write!(f, "{str}")
    }
}

pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    pub fn new(root: PathBuf) -> RocksDBStore {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let column_families: Vec<ColumnFamilyDescriptor> = ALL_COLUMN_FAMILIES
            .into_iter()
            .map(|cf| ColumnFamilyDescriptor::new(cf.to_string(), Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&db_opts, root.as_path(), column_families).unwrap();

        RocksDBStore { db }
    }

    fn add_transaction_to_write_batch(
        &self,
        batch: &mut WriteBatch,
        transaction_bundle: CommittedTransactionBundle,
    ) {
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

        batch.put_cf(
            self.cf_handle(&LocalTransactionExecutionByStateVersion),
            state_version.to_be_bytes(),
            scrypto_encode(&receipt.local_execution).unwrap(),
        );
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

        for ((db_partition_key, db_sort_key), database_update) in
            commit_bundle.substate_store_update.updates
        {
            let full_key_bytes = encode_to_rocksdb_bytes(&db_partition_key, &db_sort_key);

            match database_update {
                DatabaseUpdate::Set(value) => {
                    batch.put_cf(self.cf_handle(&Substates), full_key_bytes, value);
                }
                DatabaseUpdate::Delete => {
                    batch.delete_cf(self.cf_handle(&Substates), full_key_bytes);
                }
            }
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

impl QueryableTransactionStore for RocksDBStore {
    fn get_committed_transaction_bundles(
        &self,
        start_state_version_inclusive: u64,
        limit: usize,
    ) -> Vec<CommittedTransactionBundle> {
        let start_state_version_bytes = start_state_version_inclusive.to_be_bytes();
        let txns_iter = self.db.iterator_cf(
            self.cf_handle(&TxnByStateVersion),
            IteratorMode::From(&start_state_version_bytes, Direction::Forward),
        );

        let mut ledger_receipts_iter = self.db.iterator_cf(
            self.cf_handle(&LedgerReceiptByStateVersion),
            IteratorMode::From(&start_state_version_bytes, Direction::Forward),
        );

        let mut local_executions_iter = self.db.iterator_cf(
            self.cf_handle(&LocalTransactionExecutionByStateVersion),
            IteratorMode::From(&start_state_version_bytes, Direction::Forward),
        );

        let mut accumulator_hashes_iter = self.db.iterator_cf(
            self.cf_handle(&TxnAccumulatorHashByStateVersion),
            IteratorMode::From(&start_state_version_bytes, Direction::Forward),
        );

        let mut res = Vec::new();
        for (txn_index, next_txn_result) in txns_iter.take(limit).enumerate() {
            let next_state_version = start_state_version_inclusive + txn_index as u64;
            let next_txn_kv = next_txn_result.unwrap();

            let next_ledger_receipt_kv = ledger_receipts_iter
                .next()
                .expect("Missing ledger receipt")
                .unwrap();
            let next_local_execution_kv = local_executions_iter
                .next()
                .expect("Missing local transaction execution")
                .unwrap();
            let next_accumulator_hash_kv = accumulator_hashes_iter
                .next()
                .expect("Missing txn accumulator hash")
                .unwrap();

            for (other_key_description, other_key_bytes) in [
                ("transaction version", next_txn_kv.0),
                ("ledger receipt version", next_ledger_receipt_kv.0),
                ("local execution version", next_local_execution_kv.0),
                ("accumulator hash version", next_accumulator_hash_kv.0),
            ] {
                let other_row_version = u64::from_be_bytes((*other_key_bytes).try_into().unwrap());
                if other_row_version != next_state_version {
                    panic!("DB inconsistency! {other_key_description} ({other_row_version}) doesn't match expected state version ({next_state_version})");
                }
            }

            let next_txn = manifest_decode(next_txn_kv.1.as_ref()).unwrap();
            let next_ledger_receipt = scrypto_decode(next_ledger_receipt_kv.1.as_ref()).unwrap();
            let next_local_execution = scrypto_decode(next_local_execution_kv.1.as_ref()).unwrap();
            let next_complete_receipt = LocalTransactionReceipt {
                on_ledger: next_ledger_receipt,
                local_execution: next_local_execution,
            };
            let next_accumulator_hash =
                AccumulatorHash::from_raw_bytes((*next_accumulator_hash_kv.1).try_into().unwrap());
            let next_identifiers = CommittedTransactionIdentifiers {
                state_version: next_state_version,
                accumulator_hash: next_accumulator_hash,
            };
            res.push((next_txn, next_complete_receipt, next_identifiers));
        }

        res
    }

    fn get_committed_transaction(&self, state_version: u64) -> Option<LedgerTransaction> {
        self.db
            .get_cf(
                self.cf_handle(&TxnByStateVersion),
                state_version.to_be_bytes(),
            )
            .expect("DB error loading transaction")
            .map(|v| manifest_decode(&v).expect("Failed to decode a committed transaction"))
    }

    fn get_committed_transaction_receipt(
        &self,
        state_version: u64,
    ) -> Option<LocalTransactionReceipt> {
        let ledger_transaction_receipt =
            self.get_by_key(&LedgerReceiptByStateVersion, &state_version.to_be_bytes());
        let local_transaction_execution = self.get_by_key(
            &LocalTransactionExecutionByStateVersion,
            &state_version.to_be_bytes(),
        );
        match (ledger_transaction_receipt, local_transaction_execution) {
            (Some(on_ledger), Some(local_execution)) => Some(LocalTransactionReceipt {
                on_ledger,
                local_execution,
            }),
            (None, Some(_)) => panic!("missing ledger receipt at state version {state_version}"),
            // TODO: in future, we might want to support returning only the on-ledger part:
            (Some(_), None) => panic!("missing local execution at state version {state_version}"),
            (None, None) => None,
        }
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

impl SubstateDatabase for RocksDBStore {
    fn get_substate(&self, index_id: &Vec<u8>, key: &Vec<u8>) -> Option<Vec<u8>> {
        let substate_id = encode_substate_id(index_id, key);
        self.db
            .get_cf(self.cf_handle(&Substates), substate_id)
            .unwrap()
    }

    fn list_substates(
        &self,
        index_id: &Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        let index_id = index_id.clone();
        let start = encode_substate_id(&index_id, &vec![0]);
        let iter = self
            .db
            .iterator_cf(
                self.cf_handle(&Substates),
                IteratorMode::From(&start, Direction::Forward),
            )
            .map(|kv| {
                let (k, v) = kv.unwrap();
                let (db_partition_key, db_sort_key) = decode_from_rocksdb_bytes(k.as_ref());
                ((db_partition_key, db_sort_key), v)
            })
            .take_while(move |((db_partition_key, _), _)| index_id.eq(db_partition_key))
            .map(|((_, db_sort_key), value)| {
                (db_sort_key, value.as_ref().to_vec())
            });

        Box::new(iter)
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



// TODO: use types instead of vec u8
fn encode_to_rocksdb_bytes(partition_key: &Vec<u8>, sort_key: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(u32::try_from(partition_key.len()).unwrap().to_be_bytes());
    buffer.extend(partition_key.clone());
    buffer.extend(sort_key.clone());
    buffer
}

// TODO: use types instead of vec u8
fn decode_from_rocksdb_bytes(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) /* DbSubstateKey */ {
    let partition_key_len =
        usize::try_from(u32::from_be_bytes(copy_u8_array(&buffer[..4]))).unwrap();
    let sort_key_offset = 4 + partition_key_len;
    let partition_key = buffer[4..sort_key_offset].to_vec();
    let sort_key = buffer[sort_key_offset..].to_vec();
    (partition_key, sort_key)
}
