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

use crate::accumulator_tree::slice_merger::AccuTreeSliceMerger;

use crate::query::*;
use crate::staging::{ExecutionCache, HashStructuresDiff, ReadableStore};
use crate::store::traits::*;
use crate::transaction::{
    ConfigType, ExecutionConfigurator, LedgerTransaction, LedgerTransactionValidator,
    UserTransactionValidator, ValidatorTransaction,
};
use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::*;
use crate::{CommittedTransactionIdentifiers, HasIntentHash, IntentHash};

use ::transaction::errors::TransactionValidationError;

use parking_lot::{Mutex, RwLock};
use prometheus::Registry;

use radix_engine::types::{Categorize, ComponentAddress, Decode, Encode};

use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::Arc;

use crate::staging::epoch_handling::AccuTreeEpochHandler;

use radix_engine::blueprints::epoch_manager::Validator;

use crate::mempool_manager::MempoolManager;
use radix_engine::system::bootstrap::{
    create_genesis_data_ingestion_transaction, create_genesis_wrap_up_transaction,
    create_system_bootstrap_transaction, GenesisDataChunk,
};
use radix_engine_common::crypto::Hash;
use radix_engine_interface::constants::GENESIS_HELPER;
use radix_engine_interface::data::manifest::manifest_encode;
use radix_engine_interface::network::NetworkDefinition;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info};

#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct LoggingConfig {
    pub engine_trace: bool,
    pub state_manager_config: StateManagerLoggingConfig,
}

// TODO: Replace this with better loglevel integration
#[derive(Debug, Categorize, Encode, Decode, Clone)]
pub struct StateManagerLoggingConfig {
    pub log_on_transaction_rejection: bool,
}

const TRANSACTION_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(500);

pub struct StateManager<S> {
    store: Arc<RwLock<S>>,
    mempool_manager: Arc<MempoolManager>,
    execution_configurator: Arc<ExecutionConfigurator>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    execution_cache: Mutex<ExecutionCache>,
    user_transaction_validator: UserTransactionValidator,
    ledger_transaction_validator: LedgerTransactionValidator,
    ledger_metrics: LedgerMetrics,
    logging_config: StateManagerLoggingConfig,
}

impl<S: TransactionIdentifierLoader> StateManager<S> {
    pub fn new(
        network: &NetworkDefinition,
        store: Arc<RwLock<S>>,
        mempool_manager: Arc<MempoolManager>,
        execution_configurator: Arc<ExecutionConfigurator>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
        logging_config: LoggingConfig,
        metric_registry: &Registry,
    ) -> StateManager<S> {
        let accumulator_hash = store
            .read()
            .get_top_transaction_identifiers()
            .accumulator_hash;

        StateManager {
            store,
            mempool_manager,
            execution_configurator,
            pending_transaction_result_cache,
            execution_cache: parking_lot::const_mutex(ExecutionCache::new(accumulator_hash)),
            user_transaction_validator: UserTransactionValidator::new(network),
            ledger_transaction_validator: LedgerTransactionValidator::new(network),
            logging_config: logging_config.state_manager_config,
            ledger_metrics: LedgerMetrics::new(metric_registry),
        }
    }
}

pub enum StateManagerRejectReason {
    TransactionValidationError(TransactionValidationError),
}

#[derive(Debug)]
enum AlreadyPreparedTransaction {
    Proposed,
    Prepared,
    Committed,
}

impl<S> StateManager<S>
where
    S: ReadableStore,
    S: for<'a> TransactionIndex<&'a IntentHash>,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    pub fn prepare_genesis(
        &self,
        genesis_transaction: LedgerTransaction,
    ) -> (LedgerHashes, Option<NextEpoch>) {
        let read_store = self.store.read();
        let base_transaction_identifiers = read_store.get_top_transaction_identifiers();
        let epoch_identifiers = read_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);

        let mut state_tracker = StateTracker::initial(base_transaction_identifiers);

        let executable = self
            .ledger_transaction_validator
            .validate_and_create_executable(&genesis_transaction)
            .expect("Invalid genesis transaction");

        let hash = genesis_transaction.get_hash();
        let logged_description = format!("prepare genesis {}", hash);
        let mut lock_execution_cache = self.execution_cache.lock();
        let processed = lock_execution_cache.execute_transaction(
            self.store.read().deref(),
            &epoch_identifiers,
            state_tracker.latest_transaction_identifiers(),
            &hash,
            &self
                .execution_configurator
                .wrap(executable, ConfigType::Genesis)
                .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, &logged_description),
        );

        let commit = processed.expect_commit("prepare genesis");
        state_tracker.update(&commit.hash_structures_diff);

        (*state_tracker.latest_ledger_hashes(), commit.next_epoch())
    }

    pub fn prepare(&self, prepare_request: PrepareRequest) -> PrepareResult {
        let read_store = self.store.read();
        let base_transaction_identifiers = read_store.get_top_transaction_identifiers();
        let epoch_identifiers = read_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);

        debug_assert_eq!(
            base_transaction_identifiers.accumulator_hash,
            prepare_request.parent_accumulator
        );

        // This hashmap is used to check for any proposed intents which have already been commited (or prepared)
        // in order to exclude them. This check will eventually live in the engine/executor.
        let mut already_committed_or_prepared_intent_hashes =
            HashMap::<IntentHash, AlreadyPreparedTransaction>::new();

        let already_committed_proposed_intent_hashes = prepare_request
            .proposed_payloads
            .iter()
            .filter_map(|proposed_payload| {
                UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
                    proposed_payload,
                )
                .ok()
                .map(|validated_transaction| validated_transaction.intent_hash())
                .and_then(|intent_hash| {
                    read_store
                        .get_txn_state_version_by_identifier(&intent_hash)
                        .map(|_| (intent_hash, AlreadyPreparedTransaction::Committed))
                })
            });

        already_committed_or_prepared_intent_hashes
            .extend(already_committed_proposed_intent_hashes);

        let pending_transaction_base_state = AtState::PendingPreparingVertices {
            base_committed_state_version: base_transaction_identifiers.state_version,
        };

        let mut state_tracker = StateTracker::initial(base_transaction_identifiers);

        let already_prepared_payloads: Vec<_> = prepare_request
            .prepared_vertices
            .into_iter()
            .flat_map(|v| v.transaction_payloads)
            .collect();

        for prepared in already_prepared_payloads {
            let parsed_transaction =
                LedgerTransactionValidator::parse_unvalidated_transaction_from_slice(&prepared)
                    .expect("Already prepared transactions should be decodeable");

            let executable = match &parsed_transaction {
                LedgerTransaction::User(notarized_transaction) => {
                    already_committed_or_prepared_intent_hashes.insert(
                        notarized_transaction.intent_hash(),
                        AlreadyPreparedTransaction::Prepared,
                    );
                    self.ledger_transaction_validator
                        .validate_and_create_executable(&parsed_transaction)
                }
                LedgerTransaction::Validator(..) => self
                    .ledger_transaction_validator
                    .validate_and_create_executable(&parsed_transaction),
                LedgerTransaction::System(..) => {
                    panic!("System Transactions should not be prepared");
                }
            }
            .expect("Already prepared transactions should be valid");

            let transaction_hash = parsed_transaction.get_hash();
            let logged_description = format!("already prepared {}", transaction_hash);
            let mut lock_execution_cache = self.execution_cache.lock();
            let processed = lock_execution_cache.execute_transaction(
                read_store.deref(),
                &epoch_identifiers,
                state_tracker.latest_transaction_identifiers(),
                &transaction_hash,
                &self
                    .execution_configurator
                    .wrap(executable, ConfigType::Regular)
                    .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, &logged_description),
            );

            let commit = processed.expect_commit(logged_description);
            // TODO: Do we need to check that next epoch request has been prepared?
            state_tracker.update(&commit.hash_structures_diff);
            drop(lock_execution_cache);
        }

        let mut committed = Vec::new();

        // Round Update
        // TODO: Unify this with the proposed payloads execution
        let round_update = ValidatorTransaction::RoundUpdate {
            proposer_timestamp_ms: prepare_request.proposer_timestamp_ms,
            consensus_epoch: prepare_request.consensus_epoch,
            round_in_epoch: prepare_request.round_number,
        };
        let ledger_round_update = LedgerTransaction::Validator(round_update);

        let logged_description = format!("round update {}", prepare_request.round_number);
        let executable = round_update.prepare().to_executable();
        let mut lock_execution_cache = self.execution_cache.lock();
        let processed_round_update = lock_execution_cache.execute_transaction(
            read_store.deref(),
            &epoch_identifiers,
            state_tracker.latest_transaction_identifiers(),
            &ledger_round_update.get_hash(),
            &self
                .execution_configurator
                .wrap(executable, ConfigType::Regular)
                .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, &logged_description),
        );

        let round_update_commit = processed_round_update.expect_commit(&logged_description);
        round_update_commit.check_success(logged_description);
        state_tracker.update(&round_update_commit.hash_structures_diff);
        let mut next_epoch = round_update_commit.next_epoch();
        drop(lock_execution_cache);

        committed.push(manifest_encode(&ledger_round_update).unwrap());

        let mut rejected_payloads = Vec::new();
        let pending_transaction_timestamp = SystemTime::now();
        let mut pending_transaction_results = Vec::new();

        for proposed_payload in prepare_request.proposed_payloads {
            // Don't process any additional transactions if next epoch has occurred
            if next_epoch.is_some() {
                break;
            }

            let parsing_result =
                UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
                    &proposed_payload,
                );
            let parsed = match parsing_result {
                Ok(parsed) => parsed,
                Err(error) => {
                    rejected_payloads.push((proposed_payload, format!("{error:?}")));
                    continue;
                }
            };

            let intent_hash = parsed.intent_hash();
            let user_payload_hash = parsed.user_payload_hash();
            let invalid_at_epoch = parsed.signed_intent.intent.header.end_epoch_exclusive;
            if let Some(state) = already_committed_or_prepared_intent_hashes.get(&intent_hash) {
                rejected_payloads.push((
                    proposed_payload,
                    format!(
                        "Duplicate intent hash: {:?}, state: {:?}",
                        &intent_hash, state
                    ),
                ));
                pending_transaction_results.push(PendingTransactionResult {
                    intent_hash,
                    user_payload_hash,
                    invalid_at_epoch,
                    rejection_reason: Some(RejectionReason::IntentHashCommitted),
                });
                continue;
            }

            let validate_result = self
                .user_transaction_validator
                .validate_and_create_executable(&parsed, proposed_payload.len());

            let executable = match validate_result {
                Ok(executable) => executable,
                Err(error) => {
                    rejected_payloads.push((proposed_payload, format!("{:?}", &error)));
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        user_payload_hash,
                        invalid_at_epoch,
                        rejection_reason: Some(RejectionReason::ValidationError(error)),
                    });
                    continue;
                }
            };

            let (payload, hash) = LedgerTransaction::User(parsed.clone())
                .create_payload_and_hash()
                .unwrap();

            let logged_description = format!("newly proposed {}", hash);
            let mut lock_execution_cache = self.execution_cache.lock();
            let processed = lock_execution_cache.execute_transaction(
                read_store.deref(),
                &epoch_identifiers,
                state_tracker.latest_transaction_identifiers(),
                &hash,
                &self
                    .execution_configurator
                    .wrap(executable, ConfigType::Regular)
                    .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, &logged_description),
            );

            match processed.expect_commit_or_reject(logged_description) {
                Ok(commit) => {
                    state_tracker.update(&commit.hash_structures_diff);
                    next_epoch = commit.next_epoch();
                    drop(lock_execution_cache);

                    already_committed_or_prepared_intent_hashes
                        .insert(intent_hash, AlreadyPreparedTransaction::Proposed);
                    committed.push(payload);
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        user_payload_hash,
                        invalid_at_epoch,
                        rejection_reason: None,
                    });
                }
                Err(reject) => {
                    let error = reject.error.clone();
                    drop(lock_execution_cache);

                    rejected_payloads.push((proposed_payload, format!("{:?}", error)));
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        user_payload_hash,
                        invalid_at_epoch,
                        rejection_reason: Some(RejectionReason::FromExecution(Box::new(error))),
                    });
                }
            }
        }

        if self.logging_config.log_on_transaction_rejection {
            for rejection in rejected_payloads.iter() {
                info!("TXN INVALID: {}", rejection.1);
            }
        }

        let pending_rejected_transactions = pending_transaction_results
            .iter()
            .filter(|pending_result| pending_result.rejection_reason.is_some())
            .map(|pending_result| {
                (
                    &pending_result.intent_hash,
                    &pending_result.user_payload_hash,
                )
            })
            .collect::<Vec<_>>();
        self.mempool_manager
            .remove_rejected(&pending_rejected_transactions);

        let mut write_pending_transaction_result_cache =
            self.pending_transaction_result_cache.write();
        for pending_transaction_result in pending_transaction_results {
            let attempt = TransactionAttempt {
                rejection: pending_transaction_result.rejection_reason,
                against_state: pending_transaction_base_state.clone(),
                timestamp: pending_transaction_timestamp,
            };
            write_pending_transaction_result_cache.track_transaction_result(
                pending_transaction_result.intent_hash,
                pending_transaction_result.user_payload_hash,
                pending_transaction_result.invalid_at_epoch,
                attempt,
            );
        }
        drop(write_pending_transaction_result_cache);

        PrepareResult {
            committed,
            rejected: rejected_payloads,
            next_epoch,
            ledger_hashes: *state_tracker.latest_ledger_hashes(),
        }
    }
}

struct StateTracker {
    transaction_identifiers: CommittedTransactionIdentifiers,
    ledger_hashes: Option<LedgerHashes>,
}

impl StateTracker {
    pub fn initial(base_transaction_identifiers: CommittedTransactionIdentifiers) -> Self {
        Self {
            transaction_identifiers: base_transaction_identifiers,
            ledger_hashes: None,
        }
    }

    pub fn latest_transaction_identifiers(&self) -> &CommittedTransactionIdentifiers {
        &self.transaction_identifiers
    }

    pub fn update(&mut self, hash_structures_diff: &HashStructuresDiff) {
        self.transaction_identifiers.state_version += 1;
        self.transaction_identifiers.accumulator_hash =
            hash_structures_diff.transaction_accumulator_hash;
        self.ledger_hashes = Some(hash_structures_diff.ledger_hashes);
    }

    pub fn latest_ledger_hashes(&self) -> &LedgerHashes {
        self.ledger_hashes.as_ref().expect("no update yet")
    }
}

impl<S> StateManager<S>
where
    S: CommitStore,
    S: ReadableStore,
    S: for<'a> TransactionIndex<&'a IntentHash>,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    pub fn execute_genesis(
        &self,
        genesis_data_chunks: Vec<GenesisDataChunk>,
        initial_epoch: u64,
        max_validators: u32,
        rounds_per_epoch: u64,
        num_unstake_epochs: u64,
    ) -> LedgerProof {
        let mut curr_state_version = 0;
        let mut curr_accumulator_hash = AccumulatorHash::pre_genesis();

        // System bootstrap
        let system_bootstrap_transaction = create_system_bootstrap_transaction(
            initial_epoch,
            max_validators,
            rounds_per_epoch,
            num_unstake_epochs,
        );

        let system_bootstrap_ledger_transaction =
            LedgerTransaction::System(system_bootstrap_transaction);

        let (system_bootstrap_ledger_payload, system_bootstrap_txn_hash) =
            system_bootstrap_ledger_transaction
                .create_payload_and_hash()
                .unwrap();

        let (ledger_hashes, next_epoch) = self.prepare_genesis(system_bootstrap_ledger_transaction);

        curr_state_version += 1;
        curr_accumulator_hash = curr_accumulator_hash.accumulate(&system_bootstrap_txn_hash);

        let system_bootstrap_commit_request = CommitRequest {
            transaction_payloads: vec![system_bootstrap_ledger_payload],
            proof: LedgerProof {
                opaque: Hash([0; Hash::LENGTH]),
                ledger_header: LedgerHeader {
                    epoch: 0, // TODO(genesis): use genesis epoch
                    round: 0,
                    accumulator_state: AccumulatorState {
                        state_version: curr_state_version,
                        accumulator_hash: curr_accumulator_hash,
                    },
                    hashes: ledger_hashes,
                    consensus_parent_round_timestamp_ms: 0, /* TODO(genesis): use genesis timestamp */
                    proposer_timestamp_ms: 0, /* TODO(genesis): use genesis timestamp */
                    next_epoch,
                },
                timestamped_signatures: vec![],
            },
            vertex_store: None,
        };

        let system_bootstrap_commit_receipts = self
            .commit(system_bootstrap_commit_request)
            .expect("System bootstrap commit failed");

        let system_bootstrap_receipt = system_bootstrap_commit_receipts
            .get(0)
            .expect("Missing system bootstrap local receipt");

        match system_bootstrap_receipt.on_ledger.outcome {
            LedgerTransactionOutcome::Success => {}
            LedgerTransactionOutcome::Failure => {
                panic!("Genesis system bootstrap txn didn't succeed"); // TODO(genesis): better error handling?
            }
        }

        let mut next_nonce = 1;

        // Data ingestion
        for chunk in genesis_data_chunks {
            let genesis_data_ingestion_transaction =
                create_genesis_data_ingestion_transaction(&GENESIS_HELPER, chunk, next_nonce);
            next_nonce += 1;

            let genesis_data_ingestion_ledger_transaction =
                LedgerTransaction::System(genesis_data_ingestion_transaction);

            let (genesis_data_ingestion_ledger_payload, genesis_data_ingestion_txn_hash) =
                genesis_data_ingestion_ledger_transaction
                    .create_payload_and_hash()
                    .unwrap();

            let (ledger_hashes, next_epoch) =
                self.prepare_genesis(genesis_data_ingestion_ledger_transaction);

            curr_state_version += 1;
            curr_accumulator_hash =
                curr_accumulator_hash.accumulate(&genesis_data_ingestion_txn_hash);

            let genesis_data_ingestion_commit_request = CommitRequest {
                transaction_payloads: vec![genesis_data_ingestion_ledger_payload],
                proof: LedgerProof {
                    opaque: Hash([0; Hash::LENGTH]),
                    ledger_header: LedgerHeader {
                        epoch: 0, // TODO(genesis): use genesis epoch
                        round: 0,
                        accumulator_state: AccumulatorState {
                            state_version: curr_state_version,
                            accumulator_hash: curr_accumulator_hash,
                        },
                        hashes: ledger_hashes,
                        consensus_parent_round_timestamp_ms: 0, /* TODO(genesis): use genesis timestamp */
                        proposer_timestamp_ms: 0, /* TODO(genesis): use genesis timestamp */
                        next_epoch,
                    },
                    timestamped_signatures: vec![],
                },
                vertex_store: None,
            };

            let genesis_data_ingestion_commit_receipt = self
                .commit(genesis_data_ingestion_commit_request)
                .expect("Genesis data ingestion commit failed")
                .remove(0);

            match genesis_data_ingestion_commit_receipt.on_ledger.outcome {
                LedgerTransactionOutcome::Success => {}
                LedgerTransactionOutcome::Failure => {
                    panic!("Genesis data ingestion txn didn't succeed"); // TODO(genesis): better error handling?
                }
            }
        }

        // Wrap up

        let genesis_wrap_up_transaction = create_genesis_wrap_up_transaction(next_nonce);

        let genesis_wrap_up_ledger_transaction =
            LedgerTransaction::System(genesis_wrap_up_transaction);

        let (genesis_wrap_up_ledger_payload, genesis_wrap_up_txn_hash) =
            genesis_wrap_up_ledger_transaction
                .create_payload_and_hash()
                .unwrap();

        let (ledger_hashes, next_epoch) = self.prepare_genesis(genesis_wrap_up_ledger_transaction);

        curr_state_version += 1;
        curr_accumulator_hash = curr_accumulator_hash.accumulate(&genesis_wrap_up_txn_hash);

        let genesis_wrap_up_ledger_header = LedgerHeader {
            epoch: 0, // TODO(genesis): use genesis epoch
            round: 0,
            accumulator_state: AccumulatorState {
                state_version: curr_state_version,
                accumulator_hash: curr_accumulator_hash,
            },
            hashes: ledger_hashes,
            consensus_parent_round_timestamp_ms: 0, /* TODO(genesis): use genesis timestamp */
            proposer_timestamp_ms: 0,               /* TODO(genesis): use genesis timestamp */
            next_epoch,
        };

        let genesis_wrap_up_commit_request = CommitRequest {
            transaction_payloads: vec![genesis_wrap_up_ledger_payload],
            proof: LedgerProof {
                opaque: Hash([0; Hash::LENGTH]),
                ledger_header: genesis_wrap_up_ledger_header.clone(),
                timestamped_signatures: vec![],
            },
            vertex_store: None,
        };

        let genesis_wrap_up_commit_receipts = self
            .commit(genesis_wrap_up_commit_request)
            .expect("Genesis wrap up commit failed");

        let genesis_wrap_up_receipt = genesis_wrap_up_commit_receipts
            .get(0)
            .expect("Missing genesis wrap up local receipt");

        match genesis_wrap_up_receipt.on_ledger.outcome {
            LedgerTransactionOutcome::Success => {}
            LedgerTransactionOutcome::Failure => {
                panic!("Genesis wrap up txn didn't succeed"); // TODO(genesis): better error handling?
            }
        }

        LedgerProof {
            opaque: Hash([0; Hash::LENGTH]),
            ledger_header: genesis_wrap_up_ledger_header,
            timestamped_signatures: vec![],
        }
    }

    pub fn commit(
        &self,
        commit_request: CommitRequest,
    ) -> Result<Vec<LocalTransactionReceipt>, CommitError> {
        let commit_transactions_len = commit_request.transaction_payloads.len();
        if commit_transactions_len == 0 {
            panic!("cannot commit 0 transactions from request {commit_request:?}");
        }

        let commit_ledger_header = &commit_request.proof.ledger_header;
        let commit_accumulator_state = &commit_ledger_header.accumulator_state;
        let commit_state_version = commit_accumulator_state.state_version;
        let commit_request_start_state_version =
            commit_state_version - (commit_transactions_len as u64);

        // Whilst we should probably validate intent hash duplicates here, these are checked by validators on prepare already,
        // and the check will move into the engine at some point and we'll get it for free then...

        let parsed_transactions =
            commit_request
                .transaction_payloads
                .into_iter()
                .map(|payload| {
                    LedgerTransactionValidator::parse_unvalidated_transaction_from_slice(&payload)
                        .unwrap_or_else(|error| {
                            panic!("Committed transaction cannot be decoded - likely byzantine quorum: {error:?}");
                        })
                    // TODO - will want to validate when non-user transactions (eg round/epoch change intents) occur
                })
                .collect::<Vec<_>>();

        let mut write_store = self.store.write();
        let base_transaction_identifiers = write_store.get_top_transaction_identifiers();
        let epoch_identifiers = write_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);
        let base_state_version = base_transaction_identifiers.state_version;
        if base_state_version != commit_request_start_state_version {
            panic!(
                "Mismatched state versions - the commit request claims {} but the database thinks we're at {}",
                commit_request_start_state_version, base_state_version
            );
        }

        let mut state_tracker = StateTracker::initial(base_transaction_identifiers);
        let mut committed_transaction_bundles = Vec::new();
        let mut substate_store_update = SubstateStoreUpdate::new();
        let mut state_tree_update = HashTreeUpdate::new();
        let transaction_tree_len =
            AccuTreeEpochHandler::new(epoch_identifiers.state_version, base_state_version)
                .current_accu_tree_len();
        let mut transaction_tree_slice_merger = AccuTreeSliceMerger::new(transaction_tree_len);
        let mut receipt_tree_slice_merger = AccuTreeSliceMerger::new(transaction_tree_len);
        let mut intent_hashes = Vec::new();

        let mut result_receipts = vec![];
        for (i, transaction) in parsed_transactions.into_iter().enumerate() {
            // TODO: add some system transaction logic?
            /*
            if let LedgerTransaction::System(..) = transaction {
                if commit_state_version != 1 && i != 0 {
                    panic!("Non Genesis system transaction cannot be committed.");
                }
            }
             */

            let executable = self
                .ledger_transaction_validator
                .validate_and_create_executable(&transaction)
                .unwrap_or_else(|error| {
                    panic!(
                        "Committed transaction is not valid - likely byzantine quorum: {error:?}"
                    );
                });

            let transaction_hash = transaction.get_hash();
            let logged_description = format!("committing {}", transaction_hash);

            let mut lock_execution_cache = self.execution_cache.lock();
            let processed = lock_execution_cache.execute_transaction(
                write_store.deref(),
                &epoch_identifiers,
                state_tracker.latest_transaction_identifiers(),
                &transaction_hash,
                &self
                    .execution_configurator
                    .wrap(executable, ConfigType::Regular)
                    .warn_after(TRANSACTION_RUNTIME_WARN_THRESHOLD, &logged_description),
            );
            let commit = processed.expect_commit(logged_description);

            let hash_structures_diff = &commit.hash_structures_diff;
            state_tracker.update(hash_structures_diff);
            let next_epoch = commit.next_epoch();
            let state_hash_tree_diff = hash_structures_diff.state_hash_tree_diff.clone();
            let transaction_tree_slice = hash_structures_diff.transaction_tree_diff.slice.clone();
            let receipt_tree_slice = hash_structures_diff.receipt_tree_diff.slice.clone();
            let local_receipt = commit.local_receipt.clone();
            let database_updates = commit.database_updates.clone();
            drop(lock_execution_cache);

            Self::check_epoch_proof_match(
                commit_ledger_header,
                next_epoch,
                i == (commit_transactions_len - 1),
            )?;

            if let LedgerTransaction::User(notarized_transaction) = &transaction {
                let intent_hash = notarized_transaction.intent_hash();
                intent_hashes.push(intent_hash);
            }

            let transaction_identifiers = state_tracker.latest_transaction_identifiers().clone();

            substate_store_update.apply(database_updates);
            state_tree_update.add(transaction_identifiers.state_version, state_hash_tree_diff);
            transaction_tree_slice_merger.append(transaction_tree_slice);
            receipt_tree_slice_merger.append(receipt_tree_slice);

            result_receipts.push(local_receipt.clone());

            committed_transaction_bundles.push((
                transaction,
                local_receipt,
                transaction_identifiers,
            ));
        }

        let commit_ledger_hashes = &commit_ledger_header.hashes;
        let final_ledger_hashes = state_tracker.latest_ledger_hashes();
        if *final_ledger_hashes != *commit_ledger_hashes {
            error!(
                "computed ledger hashes at version {} differ from the ones in proof ({:?} != {:?})",
                commit_accumulator_state.state_version, final_ledger_hashes, commit_ledger_hashes
            );
            return Err(CommitError::LedgerHashesMismatch);
        }

        let final_transaction_identifiers = state_tracker.latest_transaction_identifiers().clone();

        let mut lock_execution_cache = self.execution_cache.lock();
        lock_execution_cache.progress_root(&final_transaction_identifiers.accumulator_hash);
        drop(lock_execution_cache);

        write_store.commit(CommitBundle {
            transactions: committed_transaction_bundles,
            proof: commit_request.proof,
            substate_store_update,
            vertex_store: commit_request.vertex_store,
            state_tree_update,
            transaction_tree_slice: transaction_tree_slice_merger.into_slice(),
            receipt_tree_slice: receipt_tree_slice_merger.into_slice(),
        });
        drop(write_store);

        self.ledger_metrics
            .state_version
            .set(final_transaction_identifiers.state_version as i64);
        self.ledger_metrics
            .transactions_committed
            .inc_by(commit_transactions_len as u64);
        self.ledger_metrics.last_update_epoch_second.set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );

        self.mempool_manager.remove_committed(&intent_hashes);

        let mut write_pending_transaction_result_cache =
            self.pending_transaction_result_cache.write();
        write_pending_transaction_result_cache.track_committed_transactions(
            SystemTime::now(),
            commit_request_start_state_version,
            intent_hashes,
        );
        drop(write_pending_transaction_result_cache);

        Ok(result_receipts)
    }

    fn check_epoch_proof_match(
        commit_ledger_header: &LedgerHeader,
        opt_transaction_next_epoch: Option<NextEpoch>,
        is_last_transaction_in_request: bool,
    ) -> Result<(), CommitError> {
        if is_last_transaction_in_request {
            match &commit_ledger_header.next_epoch {
                Some(proof_next_epoch) => {
                    if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
                        if transaction_next_epoch != *proof_next_epoch {
                            error!(
                                "computed next epoch differs from the one in proof ({:?} != {:?})",
                                transaction_next_epoch, proof_next_epoch
                            );
                            return Err(CommitError::EpochProofMismatch);
                        }
                    } else {
                        error!(
                            "computed no next epoch, but proof contains {:?}",
                            proof_next_epoch
                        );
                        return Err(CommitError::SuperfluousEpochProof);
                    }
                }
                None => {
                    if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
                        error!(
                            "no next epoch in proof, but last transaction in batch computed {:?}",
                            transaction_next_epoch
                        );
                        return Err(CommitError::MissingEpochProof);
                    }
                }
            };
        } else if let Some(transaction_next_epoch) = opt_transaction_next_epoch {
            error!(
                "non-last transaction in batch computed {:?}",
                transaction_next_epoch
            );
            return Err(CommitError::MissingEpochProof);
        }
        Ok(())
    }
}

impl From<(BTreeMap<ComponentAddress, Validator>, u64)> for NextEpoch {
    fn from(next_epoch_result: (BTreeMap<ComponentAddress, Validator>, u64)) -> Self {
        NextEpoch {
            validator_set: next_epoch_result
                .0
                .into_iter()
                .map(|(address, validator)| ActiveValidatorInfo {
                    address: Some(address),
                    key: validator.key,
                    stake: validator.stake,
                })
                .collect(),
            epoch: next_epoch_result.1,
        }
    }
}

struct PendingTransactionResult {
    pub intent_hash: IntentHash,
    pub user_payload_hash: UserPayloadHash,
    pub invalid_at_epoch: u64,
    pub rejection_reason: Option<RejectionReason>,
}
