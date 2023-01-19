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

use crate::mempool::simple_mempool::SimpleMempool;
use crate::query::*;
use crate::store::traits::*;
use crate::transaction::{
    LedgerTransaction, LedgerTransactionValidator, UserTransactionValidator, ValidatorTransaction,
};
use crate::types::{CommitRequest, PrepareRequest, PrepareResult, PreviewRequest};
use crate::*;
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, IntentHash, LedgerTransactionReceipt,
    MempoolAddError, PendingTransaction,
};
use ::transaction::errors::TransactionValidationError;
use ::transaction::model::{
    NotarizedTransaction, PreviewFlags, PreviewIntent, TransactionHeader, TransactionIntent,
};
use ::transaction::signing::EcdsaSecp256k1PrivateKey;
use ::transaction::validation::{TestIntentHashManager, ValidationConfig};
use prometheus::Registry;
use radix_engine::engine::ScryptoInterpreter;
use radix_engine::state_manager::deprecated_staging::StagedSubstateStoreManager;
use radix_engine::transaction::{
    execute_and_commit_transaction, execute_preview, execute_transaction, ExecutionConfig,
    FeeReserveConfig, PreviewError, PreviewResult, TransactionOutcome, TransactionReceipt,
    TransactionResult,
};
use radix_engine::types::{
    scrypto_encode, Categorize, ComponentAddress, Decimal, Decode, Encode, GlobalAddress,
    PublicKey, RENodeId, ResourceAddress,
};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_MAX_CALL_DEPTH;
use radix_engine_interface::node::NetworkDefinition;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

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

pub struct StateManager<S> {
    pub mempool: SimpleMempool,
    pub network: NetworkDefinition,
    pub store: S,
    pub user_transaction_validator: UserTransactionValidator,
    pub ledger_transaction_validator: LedgerTransactionValidator,
    pub pending_transaction_result_cache: PendingTransactionResultCache,
    pub metrics: StateManagerMetrics,
    pub prometheus_registry: Registry,
    execution_config: ExecutionConfig,
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: FeeReserveConfig,
    intent_hash_manager: TestIntentHashManager,
    logging_config: StateManagerLoggingConfig,
}

impl<S> StateManager<S> {
    pub fn new(
        network: NetworkDefinition,
        mempool: SimpleMempool,
        store: S,
        logging_config: LoggingConfig,
    ) -> StateManager<S> {
        let user_transaction_validator = UserTransactionValidator {
            validation_config: ValidationConfig::default(network.id),
            intent_hash_manager: TestIntentHashManager::new(),
        };

        let committed_transaction_validator = LedgerTransactionValidator {
            validation_config: ValidationConfig::default(network.id),
            intent_hash_manager: TestIntentHashManager::new(),
        };

        let metrics = StateManagerMetrics::new();
        let prometheus_registry = Registry::new();
        metrics.register_with(&prometheus_registry);

        StateManager {
            network,
            mempool,
            store,
            user_transaction_validator,
            ledger_transaction_validator: committed_transaction_validator,
            execution_config: ExecutionConfig {
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                trace: logging_config.engine_trace,
                max_sys_call_trace_depth: 1,
            },
            scrypto_interpreter: ScryptoInterpreter {
                wasm_engine: DefaultWasmEngine::default(),
                wasm_instrumenter: WasmInstrumenter::default(),
                wasm_metering_config: WasmMeteringConfig::default(),
            },
            fee_reserve_config: FeeReserveConfig::standard(),
            intent_hash_manager: TestIntentHashManager::new(),
            logging_config: logging_config.state_manager_config,
            pending_transaction_result_cache: PendingTransactionResultCache::new(10000, 10000),
            metrics,
            prometheus_registry,
        }
    }
}

impl<S> StateManager<S>
where
    S: ReadableSubstateStore,
{
    pub fn preview(&self, preview_request: PreviewRequest) -> Result<PreviewResult, PreviewError> {
        let notary = preview_request.notary_public_key.unwrap_or_else(|| {
            PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key())
        });

        let preview_intent = PreviewIntent {
            intent: TransactionIntent {
                header: TransactionHeader {
                    version: 1,
                    network_id: self.network.id,
                    start_epoch_inclusive: preview_request.start_epoch_inclusive,
                    end_epoch_exclusive: preview_request.end_epoch_exclusive,
                    nonce: preview_request.nonce,
                    notary_public_key: notary,
                    notary_as_signatory: preview_request.notary_as_signatory,
                    cost_unit_limit: preview_request.cost_unit_limit,
                    tip_percentage: preview_request.tip_percentage,
                },
                manifest: preview_request.manifest,
            },
            signer_public_keys: preview_request.signer_public_keys,
            flags: PreviewFlags {
                unlimited_loan: preview_request.flags.unlimited_loan,
                assume_all_signature_proofs: preview_request.flags.assume_all_signature_proofs,
                permit_duplicate_intent_hash: preview_request.flags.permit_duplicate_intent_hash,
                permit_invalid_header_epoch: preview_request.flags.permit_invalid_header_epoch,
            },
        };

        execute_preview(
            &self.store,
            &self.scrypto_interpreter,
            &self.intent_hash_manager,
            &self.network,
            preview_intent,
        )
    }
}

pub const VALIDATION_MAX_EXECUTION_MS: u32 = 500;

pub enum StateManagerRejectReason {
    TransactionValidationError(TransactionValidationError),
    /// This is temporary until we get better execution limits
    ExecutionTookTooLong {
        time_taken_ms: u32,
        time_limit_ms: u32,
    },
}

#[derive(Debug)]
enum AlreadyPreparedTransaction {
    Proposed,
    Prepared,
    Committed,
}

impl<S> StateManager<S>
where
    S: ReadableSubstateStore,
    S: for<'a> TransactionIndex<&'a IntentHash> + TransactionIndex<u64> + QueryableTransactionStore,
    S: ReadableSubstateStore + QueryableSubstateStore, // Temporary - can remove when epoch validation moves to executor
    S: QueryableProofStore,
{
    /// Performs static-validation, and then executes the transaction.
    /// By checking the TransactionReceipt, you can see if the transaction is presently commitable.
    fn validate_and_test_execute_transaction(
        &self,
        transaction: &NotarizedTransaction,
        payload_size: usize,
    ) -> Result<TransactionReceipt, StateManagerRejectReason> {
        let executable = self
            .user_transaction_validator
            .validate_and_create_executable(transaction, payload_size)
            .map_err(StateManagerRejectReason::TransactionValidationError)?;

        let start = std::time::Instant::now();

        let receipt = execute_transaction(
            &self.store,
            &self.scrypto_interpreter,
            &self.fee_reserve_config,
            &self.execution_config,
            &executable,
        );

        let elapsed_millis: u32 = start.elapsed().as_millis().try_into().unwrap_or(u32::MAX);

        if elapsed_millis > VALIDATION_MAX_EXECUTION_MS {
            return Err(StateManagerRejectReason::ExecutionTookTooLong {
                time_taken_ms: elapsed_millis,
                time_limit_ms: VALIDATION_MAX_EXECUTION_MS,
            });
        }

        Ok(receipt)
    }

    /// Checking if the transaction should be rejected requires full validation, ie:
    /// * Static Validation
    /// * Executing the transaction (up to loan repayment)
    ///
    /// We look for cached rejections first, to avoid this heavy lifting where we can
    pub fn check_for_rejection_and_add_to_mempool(
        &mut self,
        mempool_add_source: MempoolAddSource,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        self.check_for_rejection_and_add_to_mempool_internal(unvalidated_transaction)
            .map(|_| {
                self.metrics
                    .mempool_current_transactions
                    .set(self.mempool.get_count() as i64);
                self.metrics
                    .mempool_submission_added
                    .with_label(mempool_add_source)
                    .inc();
            })
            .map_err(|err| {
                self.metrics
                    .mempool_submission_rejected
                    .with_two_labels(mempool_add_source, &err)
                    .inc();

                err
            })
    }

    /// Checking if the transaction should be rejected requires full validation, ie:
    /// * Static Validation
    /// * Executing the transaction (up to loan replatment)
    ///
    /// We look for cached rejections first, to avoid this heavy lifting where we can
    fn check_for_rejection_and_add_to_mempool_internal(
        &mut self,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        // Quick check to avoid transaction validation if it couldn't be added to the mempool anyway
        self.mempool
            .check_add_would_be_possible(&unvalidated_transaction.user_payload_hash())?;

        let (record, was_cached) = self.check_for_rejection_with_caching(&unvalidated_transaction);
        let last_attempt = &record.last_attempt;

        match &last_attempt.rejection {
            // Note - we purposefully don't save a validated transaction in the mempool:
            // * Currently (Nov 2022) static validation isn't sufficiently static, as it includes EG epoch validation
            // * Moreover, the engine expects the validated transaction to be presently valid, else panics
            // * Once epoch validation is moved to the executor, we can persist validated transactions in the mempool
            None => self.mempool.add_transaction(unvalidated_transaction.into()),
            Some(reason) => Err(MempoolAddError::Rejected(MempoolAddRejection {
                reason: reason.clone(),
                against_state: last_attempt.against_state.clone(),
                was_cached,
            })),
        }
    }

    /// Reads the transaction rejection status from the cache, else calculates it fresh, by
    /// statically validating the transaction and then attempting to run it.
    ///
    /// The result is stored in the cache. If the transaction is freshly rejected,
    /// it is also removed from the mempool if it exists.
    ///
    /// Its pending transaction record is returned, along with a boolean about whether the last attempt was cached.
    pub fn check_for_rejection_with_caching(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> (PendingTransactionRecord, bool) {
        let current_time = Instant::now();
        let intent_hash = transaction.intent_hash();
        let payload_hash = transaction.user_payload_hash();

        let record_option = self
            .pending_transaction_result_cache
            .get_pending_transaction_record(&intent_hash, &payload_hash);

        if let Some(record) = record_option {
            if !record.should_recalculate(current_time) {
                return (record, true);
            }
        }

        // TODO: Remove and use some sort of cache to store size
        let payload_size = scrypto_encode(transaction).unwrap().len();
        let new_status = self.check_for_rejection_uncached(transaction, payload_size);

        if new_status.is_err() {
            // If it's been rejected, let's remove it from the mempool, if it's present
            if self
                .mempool
                .remove_transaction(&intent_hash, &payload_hash)
                .is_some()
            {
                self.metrics
                    .mempool_current_transactions
                    .set(self.mempool.get_count() as i64);
            }
        }

        let attempt = TransactionAttempt {
            rejection: new_status.as_ref().err().cloned(),
            against_state: AtState::Committed {
                state_version: self.store.max_state_version(),
            },
            timestamp: current_time,
        };
        self.pending_transaction_result_cache
            .track_transaction_result(intent_hash, payload_hash, attempt);

        // Unwrap allowed as we've just put it in the cache, and unless the cache has size 0 it must be there
        (
            self.pending_transaction_result_cache
                .get_pending_transaction_record(&intent_hash, &payload_hash)
                .unwrap(),
            false,
        )
    }

    pub fn check_for_rejection_uncached(
        &self,
        transaction: &NotarizedTransaction,
        payload_size: usize,
    ) -> Result<(), RejectionReason> {
        if self
            .store
            .get_payload_hash(&transaction.intent_hash())
            .is_some()
        {
            return Err(RejectionReason::IntentHashCommitted);
        }

        // TODO: Only run transaction up to the loan
        let receipt = self
            .validate_and_test_execute_transaction(transaction, payload_size)
            .map_err(|reason| match reason {
                StateManagerRejectReason::TransactionValidationError(validation_error) => {
                    RejectionReason::ValidationError(validation_error)
                }
                StateManagerRejectReason::ExecutionTookTooLong {
                    time_taken_ms,
                    time_limit_ms,
                } => {
                    warn!(
                        "Transaction execution took {}ms, above limit of {}ms, so rejecting",
                        time_taken_ms, time_limit_ms
                    );
                    RejectionReason::ExecutionTookTooLong { time_limit_ms }
                }
            })?;

        match receipt.result {
            TransactionResult::Reject(result) => {
                Err(RejectionReason::FromExecution(Box::new(result.error)))
            }
            TransactionResult::Commit(..) => Ok(()),
        }
    }

    pub fn get_relay_transactions(&mut self) -> Vec<PendingTransaction> {
        let mut mempool_txns = self.mempool.get_transactions();

        let mut txns_to_remove = Vec::new();
        for data in mempool_txns.values() {
            let (record, was_cached) =
                self.check_for_rejection_with_caching(&data.transaction.payload);
            if !was_cached && record.last_attempt.rejection.is_some() {
                txns_to_remove.push((data.transaction.intent_hash, data.transaction.payload_hash));
            }
        }

        for txn_to_remove in txns_to_remove {
            mempool_txns.remove(&txn_to_remove.1);
            if self
                .mempool
                .remove_transaction(&txn_to_remove.0, &txn_to_remove.1)
                .is_some()
            {
                self.metrics
                    .mempool_current_transactions
                    .set(self.mempool.get_count() as i64);
            }
        }

        mempool_txns
            .into_values()
            .map(|data| data.transaction)
            .collect()
    }

    // TODO: Update to prepare_system_transaction when we start to support forking
    pub fn prepare_genesis(&mut self, genesis: PrepareGenesisRequest) -> PrepareGenesisResult {
        let parsed_transaction =
            LedgerTransactionValidator::parse_unvalidated_transaction_from_slice(&genesis.genesis)
                .expect("Already prepared transactions should be decodeable");
        let executable = self
            .ledger_transaction_validator
            .validate_and_create_executable(&parsed_transaction)
            .expect("Failed to validate genesis");
        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);
        let mut staged_store = staged_store_manager.get_output_store(staged_node);
        let receipt = execute_and_commit_transaction(
            &mut staged_store,
            &self.scrypto_interpreter,
            &self.fee_reserve_config,
            &self.execution_config,
            &executable,
        );
        match receipt.result {
            TransactionResult::Commit(commit) => PrepareGenesisResult {
                validator_set: commit.next_epoch.map(|(validator_set, _)| validator_set),
            },
            TransactionResult::Reject(reject_result) => {
                panic!("Genesis rejected. Result: {:?}", reject_result)
            }
        }
    }

    pub fn prepare(&mut self, prepare_request: PrepareRequest) -> PrepareResult {
        // This intent hash check, and current epoch should eventually live in the executor
        let pending_transaction_base_state_version = self.store.max_state_version();
        let mut already_committed_or_prepared_intent_hashes: HashMap<
            IntentHash,
            AlreadyPreparedTransaction,
        > = HashMap::new();

        let already_committed_proposed_payload_hashes = prepare_request
            .proposed_payloads
            .iter()
            .filter_map(|proposed_payload| {
                UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
                    proposed_payload,
                )
                .ok()
                .map(|validated_transaction| validated_transaction.intent_hash())
                .and_then(|intent_hash| {
                    self.store
                        .get_payload_hash(&intent_hash)
                        .map(|_| (intent_hash, AlreadyPreparedTransaction::Committed))
                })
            });

        already_committed_or_prepared_intent_hashes
            .extend(already_committed_proposed_payload_hashes);

        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);
        let mut staged_store = staged_store_manager.get_output_store(staged_node);

        for prepared in prepare_request.already_prepared_payloads {
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
            .expect("Already prepared tranasctions should be valid");

            let receipt = execute_and_commit_transaction(
                &mut staged_store,
                &self.scrypto_interpreter,
                &self.fee_reserve_config,
                &self.execution_config,
                &executable,
            );
            match receipt.result {
                TransactionResult::Commit(_) => {
                    // TODO: Do we need to check that next epoch request has been prepared?
                }
                TransactionResult::Reject(reject_result) => {
                    panic!(
                        "Already prepared transactions should be committable. Reject result: {:?}",
                        reject_result
                    )
                }
            }
        }

        let mut committed = Vec::new();

        // Round Update
        // TODO: Unify this with the proposed payloads execution
        let validator_transaction = ValidatorTransaction::RoundUpdate {
            proposer_timestamp_ms: prepare_request.proposer_timestamp_ms,
            consensus_epoch: prepare_request.consensus_epoch,
            round_in_epoch: prepare_request.round_number,
        };
        let prepared_txn = validator_transaction.prepare();
        let executable = prepared_txn.to_executable();
        let receipt = execute_transaction(
            &staged_store,
            &self.scrypto_interpreter,
            &self.fee_reserve_config,
            &self.execution_config,
            &executable,
        );
        let mut next_epoch = match receipt.result {
            TransactionResult::Commit(commit_result) => {
                if let TransactionOutcome::Failure(error) = commit_result.outcome {
                    panic!("Validator txn failed: {:?}", error);
                }

                commit_result.state_updates.commit(&mut staged_store);
                let validator_txn = LedgerTransaction::Validator(validator_transaction);
                committed.push(scrypto_encode(&validator_txn).unwrap());

                commit_result.next_epoch.map(|e| NextEpoch {
                    validator_set: e.0,
                    epoch: e.1,
                })
            }
            TransactionResult::Reject(reject_result) => {
                panic!("Validator txn failed: {:?}", reject_result)
            }
        };

        let mut rejected_payloads = Vec::new();

        let pending_transaction_timestamp = Instant::now();
        let mut pending_transaction_results = Vec::new();

        // Don't process any additional transactions if next epoch has occurred
        if next_epoch.is_none() {
            for proposed_payload in prepare_request.proposed_payloads {
                let parsed =
                    match UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
                        &proposed_payload,
                    ) {
                        Ok(parsed) => parsed,
                        Err(error) => {
                            rejected_payloads.push((proposed_payload, format!("{:?}", error)));
                            continue;
                        }
                    };

                let intent_hash = parsed.intent_hash();
                let user_payload_hash = parsed.user_payload_hash();
                if let Some(state) = already_committed_or_prepared_intent_hashes.get(&intent_hash) {
                    rejected_payloads.push((
                        proposed_payload,
                        format!(
                            "Duplicate intent hash: {:?}, state: {:?}",
                            &intent_hash, state
                        ),
                    ));
                    pending_transaction_results.push((
                        intent_hash,
                        user_payload_hash,
                        Some(RejectionReason::IntentHashCommitted),
                    ));
                    continue;
                }

                let validate_result = self
                    .user_transaction_validator
                    .validate_and_create_executable(&parsed, proposed_payload.len());

                let executable = match validate_result {
                    Ok(executable) => executable,
                    Err(error) => {
                        rejected_payloads.push((proposed_payload, format!("{:?}", &error)));
                        pending_transaction_results.push((
                            intent_hash,
                            user_payload_hash,
                            Some(RejectionReason::ValidationError(error)),
                        ));
                        continue;
                    }
                };

                let receipt = execute_and_commit_transaction(
                    &mut staged_store,
                    &self.scrypto_interpreter,
                    &self.fee_reserve_config,
                    &self.execution_config,
                    &executable,
                );

                match receipt.result {
                    TransactionResult::Commit(result) => {
                        already_committed_or_prepared_intent_hashes
                            .insert(intent_hash, AlreadyPreparedTransaction::Proposed);
                        committed.push(LedgerTransaction::User(parsed).create_payload().unwrap());
                        pending_transaction_results.push((intent_hash, user_payload_hash, None));

                        if let Some(e) = result.next_epoch {
                            next_epoch = Some(NextEpoch {
                                validator_set: e.0,
                                epoch: e.1,
                            });
                            break;
                        }
                    }
                    TransactionResult::Reject(reject_result) => {
                        rejected_payloads.push((proposed_payload, format!("{:?}", &reject_result)));
                        pending_transaction_results.push((
                            intent_hash,
                            user_payload_hash,
                            Some(RejectionReason::FromExecution(Box::new(
                                reject_result.error,
                            ))),
                        ));
                    }
                };
            }
        }

        if self.logging_config.log_on_transaction_rejection {
            for rejection in rejected_payloads.iter() {
                info!("TXN INVALID: {}", rejection.1);
            }
        }

        for (intent_hash, user_payload_hash, rejection_option) in pending_transaction_results {
            if rejection_option.is_some() {
                // Removing transactions rejected during prepare from the mempool is a bit of overkill:
                // just because transactions were rejected in this history doesn't mean this history will be committed.
                //
                // But it'll do for now as a defensive measure until we can have a more intelligent mempool.
                if self
                    .mempool
                    .remove_transaction(&intent_hash, &user_payload_hash)
                    .is_some()
                {
                    // TODO - fix this metric to live inside the mempool
                    self.metrics
                        .mempool_current_transactions
                        .set(self.mempool.get_count() as i64);
                }
            }
            let attempt = TransactionAttempt {
                rejection: rejection_option,
                against_state: AtState::PendingPreparingVertices {
                    base_committed_state_version: pending_transaction_base_state_version,
                },
                timestamp: pending_transaction_timestamp,
            };
            self.pending_transaction_result_cache
                .track_transaction_result(intent_hash, user_payload_hash, attempt);
        }

        PrepareResult {
            committed,
            rejected: rejected_payloads,
            next_epoch,
        }
    }
}

impl<'db, S> StateManager<S>
where
    S: CommitStore<'db>,
    S: ReadableSubstateStore,
    S: QueryableProofStore + TransactionIndex<u64>,
{
    pub fn save_vertex_store(&'db mut self, vertex_store: Vec<u8>) {
        let mut db_transaction = self.store.create_db_transaction();
        db_transaction.save_vertex_store(vertex_store);
        db_transaction.commit();
    }

    pub fn commit(&'db mut self, commit_request: CommitRequest) -> Result<(), CommitError> {
        let mut to_store = Vec::new();
        let mut payload_hashes = Vec::new();
        let mut intent_hashes = Vec::new();
        let commit_request_start_state_version =
            commit_request.proof_state_version - (commit_request.transaction_payloads.len() as u64);

        // Whilst we should probably validate intent hash duplicates here, these are checked by validators on prepare already,
        // and the check will move into the engine at some point and we'll get it for free then...

        let parsed_transactions =
            commit_request
                .transaction_payloads
                .into_iter()
                .map(|payload| {
                    LedgerTransactionValidator::parse_unvalidated_transaction_from_slice(&payload)
                        .unwrap_or_else(|error| {
                            panic!("Committed transaction cannot be decoded - likely byzantine quorum: {:?}", error);
                        })
                    // TODO - will want to validate when non-user transactions (eg round/epoch change intents) occur
                })
                .collect::<Vec<_>>();

        let current_top_of_ledger = self
            .store
            .get_top_of_ledger_transaction_identifiers()
            .unwrap_or_else(CommittedTransactionIdentifiers::pre_genesis);
        if current_top_of_ledger.state_version != commit_request_start_state_version {
            panic!(
                "Mismatched state versions - the commit request claims {} but the database thinks we're at {}",
                commit_request_start_state_version,
                current_top_of_ledger.state_version
            );
        }

        let mut db_transaction = self.store.create_db_transaction();
        let mut current_state_version = current_top_of_ledger.state_version;
        let mut current_accumulator = current_top_of_ledger.accumulator_hash;
        let mut epoch_boundary = None;
        let mut receipts = Vec::new();

        for (i, transaction) in parsed_transactions.iter().enumerate() {
            if let LedgerTransaction::System(..) = transaction {
                // TODO: Cleanup and use real system transaction logic
                if commit_request.proof_state_version != 1 && i != 0 {
                    panic!("Non Genesis system transaction cannot be committed.");
                }
            }

            let executable = self
                .ledger_transaction_validator
                .validate_and_create_executable(transaction)
                .unwrap_or_else(|error| {
                    panic!(
                        "Committed transaction is not valid - likely byzantine quorum: {:?}",
                        error
                    );
                });

            let engine_receipt = execute_and_commit_transaction(
                &mut db_transaction,
                &self.scrypto_interpreter,
                &self.fee_reserve_config,
                &self.execution_config,
                &executable,
            );

            let ledger_receipt: LedgerTransactionReceipt = match engine_receipt.result {
                TransactionResult::Commit(result) => {
                    if let Some((_, next_epoch)) = result.next_epoch {
                        let is_last = i == (parsed_transactions.len() - 1);
                        if !is_last {
                            return Err(CommitError::MissingEpochProof);
                        }
                        // TODO: Use actual result and verify proof validator set matches transaction receipt validator set
                        epoch_boundary = Some(next_epoch);
                    }

                    (result, engine_receipt.execution.fee_summary).into()
                }
                TransactionResult::Reject(error) => {
                    panic!(
                        "Failed to commit a txn at state version {}: {:?}",
                        commit_request.proof_state_version, error
                    )
                }
            };
            receipts.push(ledger_receipt);
        }

        for (transaction, ledger_receipt) in
            parsed_transactions.into_iter().zip(receipts.into_iter())
        {
            let payload_hash = transaction.get_hash();
            if let LedgerTransaction::User(notarized_transaction) = &transaction {
                let intent_hash = notarized_transaction.intent_hash();
                intent_hashes.push(intent_hash);
            }

            current_accumulator = current_accumulator.accumulate(&payload_hash);
            current_state_version += 1;

            let identifiers = CommittedTransactionIdentifiers {
                state_version: current_state_version,
                accumulator_hash: current_accumulator,
            };

            to_store.push((transaction, ledger_receipt, identifiers));
            payload_hashes.push(payload_hash);
        }

        let committed_transactions_count = to_store.len();

        db_transaction.insert_committed_transactions(to_store);
        db_transaction.insert_tids_and_proof(
            commit_request.proof_state_version,
            epoch_boundary,
            payload_hashes,
            commit_request.proof,
        );
        if let Some(post_commit_vertex_store) = commit_request.post_commit_vertex_store {
            db_transaction.save_vertex_store(post_commit_vertex_store);
        }

        db_transaction.commit();
        self.metrics
            .ledger_state_version
            .set(current_state_version as i64);
        self.metrics
            .ledger_transactions_committed
            .inc_by(committed_transactions_count as u64);
        self.metrics.ledger_last_update_epoch_second.set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );
        self.mempool.handle_committed_transactions(&intent_hashes);
        self.metrics
            .mempool_current_transactions
            .set(self.mempool.get_count() as i64);

        self.pending_transaction_result_cache
            .track_committed_transactions(
                Instant::now(),
                commit_request_start_state_version,
                intent_hashes,
            );

        Ok(())
    }
}

impl<S: ReadableSubstateStore + QueryableSubstateStore> StateManager<S> {
    pub fn get_component_resources(
        &self,
        component_address: ComponentAddress,
    ) -> Option<HashMap<ResourceAddress, Decimal>> {
        let mut resource_accounter = ResourceAccounter::new(&self.store);
        resource_accounter
            .add_resources(RENodeId::Global(GlobalAddress::Component(
                component_address,
            )))
            .map_or(None, |()| Some(resource_accounter.into_map()))
    }

    pub fn get_epoch(&self) -> u64 {
        self.store.get_epoch()
    }
}
