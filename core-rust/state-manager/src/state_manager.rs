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

use crate::jni::state_computer::JavaValidatorInfo;
use crate::mempool::simple_mempool::SimpleMempool;
use crate::query::*;
use crate::staging::{ExecutionCache, HashTreeDiff, ProcessedResult};
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
    Executable, NotarizedTransaction, PreviewFlags, PreviewIntent, TransactionHeader,
    TransactionIntent,
};
use ::transaction::signing::EcdsaSecp256k1PrivateKey;
use ::transaction::validation::{TestIntentHashManager, ValidationConfig};
use parking_lot::RwLock;
use prometheus::Registry;
use radix_engine::transaction::{
    execute_preview, execute_transaction, AbortReason, ExecutionConfig, FeeReserveConfig,
    PreviewError, PreviewResult, TransactionOutcome, TransactionReceipt, TransactionResult,
};
use radix_engine::types::{
    scrypto_encode, Categorize, ComponentAddress, Decimal, Decode, Encode, GlobalAddress,
    PublicKey, RENodeId, ResourceAddress,
};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_MAX_CALL_DEPTH;

use radix_engine::state_manager::StateDiff;
use radix_engine_interface::api::types::{
    NodeModuleId, SubstateId, SubstateOffset, ValidatorOffset,
};
use radix_engine_stores::hash_tree::tree_store::ReadableTreeStore;
use std::collections::HashMap;
use std::convert::TryInto;

use radix_engine::blueprints::epoch_manager::ValidatorSubstate;
use radix_engine::kernel::ScryptoInterpreter;
use radix_engine_interface::network::NetworkDefinition;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

const UP_TO_FEE_LOAN_TRANSACTION_WARN_TIME_LIMIT_MS: u32 = 100;
const FULL_TRANSACTION_WARN_TIME_LIMIT_MS: u32 = 500;

pub struct StateManager<S> {
    pub mempool: RwLock<SimpleMempool>,
    pub network: NetworkDefinition,
    store: S,
    execution_cache: ExecutionCache,
    pub user_transaction_validator: UserTransactionValidator,
    pub ledger_transaction_validator: LedgerTransactionValidator,
    pub pending_transaction_result_cache: PendingTransactionResultCache,
    pub metrics: StateManagerMetrics,
    pub prometheus_registry: Registry,
    execution_config: ExecutionConfig,
    execution_config_for_pending_transactions: ExecutionConfig,
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: FeeReserveConfig,
    intent_hash_manager: TestIntentHashManager,
    logging_config: StateManagerLoggingConfig,
}

impl<S> StateManager<S>
where
    S: ReadableSubstateStore + QueryableAccumulatorHash,
{
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

        let accumulator_hash = store.get_top_accumulator_hash();

        StateManager {
            network,
            mempool: RwLock::new(mempool),
            store,
            execution_cache: ExecutionCache::new(accumulator_hash),
            user_transaction_validator,
            ledger_transaction_validator: committed_transaction_validator,
            execution_config: ExecutionConfig {
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                trace: logging_config.engine_trace,
                max_sys_call_trace_depth: 1,
                abort_when_loan_repaid: false,
            },
            execution_config_for_pending_transactions: ExecutionConfig {
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                trace: logging_config.engine_trace,
                max_sys_call_trace_depth: 1,
                abort_when_loan_repaid: true,
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
    pub fn store(&self) -> &S {
        &self.store
    }

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

        let start = std::time::Instant::now();

        let result = execute_preview(
            &self.store,
            &self.scrypto_interpreter,
            &self.intent_hash_manager,
            &self.network,
            preview_intent,
        );

        let elapsed_millis: u32 = start.elapsed().as_millis().try_into().unwrap_or(u32::MAX);

        if elapsed_millis > FULL_TRANSACTION_WARN_TIME_LIMIT_MS {
            warn!(
                "Preview execution took {}ms, above warning threshold of {}ms",
                elapsed_millis, FULL_TRANSACTION_WARN_TIME_LIMIT_MS
            );
        }

        result
    }
}

impl<S> StateManager<S>
where
    S: ReadableSubstateStore + ReadableTreeStore,
{
    fn execute_for_staging_with_cache(
        &mut self,
        parent_accumulator_hash: &AccumulatorHash,
        executable: &Executable,
        payload_hash: &LedgerPayloadHash,
    ) -> (AccumulatorHash, &ProcessedResult) {
        let new_accumulator_hash = parent_accumulator_hash.accumulate(payload_hash);
        let processed_result = self.execution_cache.execute_transaction(
            &self.store,
            parent_accumulator_hash,
            &new_accumulator_hash,
            |store| {
                let start = Instant::now();

                let result = execute_transaction(
                    store,
                    &self.scrypto_interpreter,
                    &self.fee_reserve_config,
                    &self.execution_config,
                    executable,
                );

                let elapsed_millis: u32 = start.elapsed().as_millis().try_into().unwrap_or(u32::MAX);

                if elapsed_millis > FULL_TRANSACTION_WARN_TIME_LIMIT_MS {
                    warn!(
                        "Transaction execution took {}ms, above warning threshold of {}ms (ledger payload hash: {}, accumulator hash: {})",
                        elapsed_millis, FULL_TRANSACTION_WARN_TIME_LIMIT_MS, payload_hash, parent_accumulator_hash
                    );
                }

                result
            },
        );
        (new_accumulator_hash, processed_result)
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
    S: ReadableSubstateStore + ReadableTreeStore,
    S: for<'a> TransactionIndex<&'a IntentHash> + QueryableTransactionStore,
    S: QueryableProofStore + QueryableAccumulatorHash,
{
    /// Performs static-validation, and then executes the transaction.
    /// By checking the TransactionReceipt, you can see if the transaction is presently commitable.
    fn validate_and_test_execute_transaction_up_to_fee_loan(
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
            &self.execution_config_for_pending_transactions,
            &executable,
        );

        let elapsed_millis: u32 = start.elapsed().as_millis().try_into().unwrap_or(u32::MAX);

        if elapsed_millis > UP_TO_FEE_LOAN_TRANSACTION_WARN_TIME_LIMIT_MS {
            warn!(
                "Pending transaction execution up to fee loan took {}ms, above warning threshold of {}ms",
                elapsed_millis, UP_TO_FEE_LOAN_TRANSACTION_WARN_TIME_LIMIT_MS
            );
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
                    .set(self.mempool.read().get_count() as i64);
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
    /// * Executing the transaction (up to loan repayment)
    ///
    /// We look for cached rejections first, to avoid this heavy lifting where we can
    fn check_for_rejection_and_add_to_mempool_internal(
        &mut self,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        // Quick check to avoid transaction validation if it couldn't be added to the mempool anyway
        self.mempool
            .write()
            .check_add_would_be_possible(&unvalidated_transaction.user_payload_hash())?;

        let (record, was_cached) = self.check_for_rejection_with_caching(&unvalidated_transaction);
        let accept_into_mempool = record.should_accept_into_mempool(was_cached);

        if accept_into_mempool.is_ok() {
            // Note - we purposefully don't save a validated transaction in the mempool:
            // * Currently (Nov 2022) static validation isn't sufficiently static, as it includes EG epoch validation
            // * Moreover, the engine expects the validated transaction to be presently valid, else panics
            // * Once epoch validation is moved to the executor, we can persist validated transactions in the mempool
            self.mempool
                .write()
                .add_transaction(unvalidated_transaction.into())?;
        }

        accept_into_mempool.map_err(MempoolAddError::Rejected)
    }

    /// Reads the transaction rejection status from the cache, else calculates it fresh, by
    /// statically validating the transaction and then attempting to run it.
    ///
    /// The result is stored in the cache.
    /// If the transaction is freshly rejected, the caller should perform additional cleanup,
    /// e.g. removing the transaction from the mempool
    ///
    /// Its pending transaction record is returned, along with a boolean about whether the last attempt was cached.
    pub fn check_for_rejection_with_caching(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> (PendingTransactionRecord, bool) {
        let current_time = SystemTime::now();
        let current_epoch = self.store().get_epoch();
        let intent_hash = transaction.intent_hash();
        let payload_hash = transaction.user_payload_hash();
        let invalid_from_epoch = transaction.signed_intent.intent.header.end_epoch_exclusive;

        let record_option = self
            .pending_transaction_result_cache
            .get_pending_transaction_record(&intent_hash, &payload_hash, invalid_from_epoch);

        if let Some(record) = record_option {
            if !record.should_recalculate(current_epoch, current_time) {
                return (record, true);
            }
        }

        // TODO: Remove and use some sort of cache to store size
        let payload_size = scrypto_encode(transaction).unwrap().len();
        let rejection = self
            .check_for_rejection_uncached(transaction, payload_size)
            .err();

        let attempt = TransactionAttempt {
            rejection,
            against_state: AtState::Committed {
                state_version: self.store.max_state_version(),
            },
            timestamp: current_time,
        };
        let invalid_from_epoch = transaction.signed_intent.intent.header.end_epoch_exclusive;
        self.pending_transaction_result_cache
            .track_transaction_result(intent_hash, payload_hash, invalid_from_epoch, attempt);

        // Unwrap allowed as we've just put it in the cache, and unless the cache has size 0 it must be there
        (
            self.pending_transaction_result_cache
                .get_pending_transaction_record(&intent_hash, &payload_hash, invalid_from_epoch)
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
            .get_txn_state_version_by_identifier(&transaction.intent_hash())
            .is_some()
        {
            return Err(RejectionReason::IntentHashCommitted);
        }

        let receipt = self
            .validate_and_test_execute_transaction_up_to_fee_loan(transaction, payload_size)
            .map_err(|reason| match reason {
                StateManagerRejectReason::TransactionValidationError(validation_error) => {
                    RejectionReason::ValidationError(validation_error)
                }
            })?;

        match receipt.result {
            TransactionResult::Reject(reject_result) => Err(RejectionReason::FromExecution(
                Box::new(reject_result.error),
            )),
            TransactionResult::Commit(..) => Ok(()),
            TransactionResult::Abort(abort_result) => {
                // The transaction aborted after the fee loan was repaid - meaning the transaction result would get committed
                match abort_result.reason {
                    AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment => Ok(()),
                }
            }
        }
    }

    pub fn get_relay_transactions(
        &mut self,
        max_num_txns: usize,
        max_payload_size_bytes: usize,
    ) -> Vec<PendingTransaction> {
        let mut mempool_transaction_data: Vec<_> = {
            // Explicit scope to ensure the lock is dropped
            self.mempool
                .read()
                .transactions()
                .values()
                .cloned()
                .collect()
        };
        mempool_transaction_data.shuffle(&mut thread_rng());

        let mut transactions_to_return = Vec::new();
        let mut payload_size_so_far = 0usize;

        // We (partially) cleanup the mempool on the occasion of getting the relay txns
        // TODO: move this to a separate job
        let mut transactions_to_remove = Vec::new();

        for transaction_data in mempool_transaction_data.into_iter() {
            let (record, _) =
                self.check_for_rejection_with_caching(&transaction_data.transaction.payload);
            if record.latest_attempt.rejection.is_some() {
                // Mark the transaction to be removed from the mempool
                // (see the comment above about moving this to a separate job)
                transactions_to_remove.push((
                    transaction_data.transaction.intent_hash,
                    transaction_data.transaction.payload_hash,
                ));
            } else {
                // Check the payload size limit
                payload_size_so_far += transaction_data.transaction.payload_size;
                if payload_size_so_far > max_payload_size_bytes {
                    break;
                }

                // Add the transaction to response
                transactions_to_return.push(transaction_data.transaction);
                if transactions_to_return.len() >= max_num_txns {
                    break;
                }
            }
        }

        // See the comment above about moving this to a separate job
        for transaction_to_remove in transactions_to_remove {
            if self
                .mempool
                .write()
                .remove_transaction(&transaction_to_remove.0, &transaction_to_remove.1)
                .is_some()
            {
                self.metrics
                    .mempool_current_transactions
                    .set(self.mempool.read().get_count() as i64);
            }
        }

        transactions_to_return
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

        let parent_accumulator_hash = AccumulatorHash::pre_genesis();

        let (_, processed) = self.execute_for_staging_with_cache(
            &parent_accumulator_hash,
            &executable,
            &parsed_transaction.get_hash(),
        );
        match &processed.receipt().result {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(..) => PrepareGenesisResult {
                    validator_set: commit
                        .next_epoch
                        .clone()
                        .map(|(validator_set, _)| validator_set),
                    state_hash: *processed.state_hash(),
                },
                TransactionOutcome::Failure(error) => {
                    panic!("Genesis failed. Error: {:?}", error)
                }
            },
            TransactionResult::Reject(reject_result) => {
                panic!("Genesis rejected. Result: {:?}", reject_result)
            }
            TransactionResult::Abort(_) => {
                panic!("Genesis aborted. This should not be possible");
            }
        }
    }

    pub fn prepare(&mut self, prepare_request: PrepareRequest) -> PrepareResult {
        let pending_transaction_base_state_version = self.store.max_state_version();

        // This hashmap is used to check for any proposed intents which have already been commited (or prepared)
        // in order to exclude them. This check will eventually live in the engine/executor.
        let mut already_committed_or_prepared_intent_hashes: HashMap<
            IntentHash,
            AlreadyPreparedTransaction,
        > = HashMap::new();

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
                    self.store
                        .get_txn_state_version_by_identifier(&intent_hash)
                        .map(|_| (intent_hash, AlreadyPreparedTransaction::Committed))
                })
            });

        already_committed_or_prepared_intent_hashes
            .extend(already_committed_proposed_intent_hashes);

        let mut state_tracker = StateTracker::initial(self.store.get_top_accumulator_hash());

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
            .expect("Already prepared tranasctions should be valid");

            let (new_accumulator_hash, processed) = self.execute_for_staging_with_cache(
                state_tracker.accumulator_hash(),
                &executable,
                &parsed_transaction.get_hash(),
            );
            match &processed.receipt().result {
                TransactionResult::Commit(_) => {
                    // TODO: Do we need to check that next epoch request has been prepared?
                    state_tracker.update(new_accumulator_hash, *processed.state_hash());
                }
                TransactionResult::Reject(reject_result) => {
                    panic!(
                        "Already prepared transactions should be committable. Reject result: {:?}",
                        reject_result
                    )
                }
                TransactionResult::Abort(_) => {
                    panic!("Already prepared transactions should be committable.");
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
        let validator_txn = LedgerTransaction::Validator(validator_transaction);
        let (new_accumulator_hash, processed) = self.execute_for_staging_with_cache(
            state_tracker.accumulator_hash(),
            &executable,
            &validator_txn.get_hash(),
        );
        let mut next_epoch = match &processed.receipt().result {
            TransactionResult::Commit(commit_result) => {
                if let TransactionOutcome::Failure(error) = &commit_result.outcome {
                    panic!("Validator txn failed: {:?}", error);
                }

                state_tracker.update(new_accumulator_hash, *processed.state_hash());
                committed.push(scrypto_encode(&validator_txn).unwrap());

                commit_result.next_epoch.clone().map(|e| NextEpoch {
                    validator_set: e.0,
                    epoch: e.1,
                })
            }
            TransactionResult::Reject(reject_result) => {
                panic!("Validator txn failed: {:?}", reject_result)
            }
            TransactionResult::Abort(abort_result) => {
                panic!("Validator txn aborted: {:?}", abort_result);
            }
        };

        let mut rejected_payloads = Vec::new();

        let pending_transaction_timestamp = SystemTime::now();
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
                let invalid_at_epoch = parsed.signed_intent.intent.header.end_epoch_exclusive;
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
                        invalid_at_epoch,
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
                            invalid_at_epoch,
                            Some(RejectionReason::ValidationError(error)),
                        ));
                        continue;
                    }
                };

                let (payload, hash) = LedgerTransaction::User(parsed.clone())
                    .create_payload_and_hash()
                    .unwrap();
                let (new_accumulator_hash, processed) = self.execute_for_staging_with_cache(
                    state_tracker.accumulator_hash(),
                    &executable,
                    &hash,
                );

                match &processed.receipt().result {
                    TransactionResult::Commit(result) => {
                        state_tracker.update(new_accumulator_hash, *processed.state_hash());

                        already_committed_or_prepared_intent_hashes
                            .insert(intent_hash, AlreadyPreparedTransaction::Proposed);
                        committed.push(payload);
                        pending_transaction_results.push((
                            intent_hash,
                            user_payload_hash,
                            invalid_at_epoch,
                            None,
                        ));

                        if let Some(e) = &result.next_epoch {
                            next_epoch = Some(NextEpoch {
                                validator_set: e.0.clone(),
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
                            invalid_at_epoch,
                            Some(RejectionReason::FromExecution(Box::new(
                                reject_result.error.clone(),
                            ))),
                        ));
                    }
                    TransactionResult::Abort(_) => {
                        panic!("Should not be aborting prepared transactions.");
                    }
                };
            }
        }

        if self.logging_config.log_on_transaction_rejection {
            for rejection in rejected_payloads.iter() {
                info!("TXN INVALID: {}", rejection.1);
            }
        }

        {
            let mut mempool = self.mempool.write();
            for (intent_hash, user_payload_hash, _, rejection_option) in
                pending_transaction_results.iter()
            {
                if rejection_option.is_some() {
                    // Removing transactions rejected during prepare from the mempool is a bit of overkill:
                    // just because transactions were rejected in this history doesn't mean this history will be committed.
                    //
                    // But it'll do for now as a defensive measure until we can have a more intelligent mempool.
                    mempool.remove_transaction(intent_hash, user_payload_hash);
                }
            }
            self.metrics
                .mempool_current_transactions
                .set(mempool.get_count() as i64);
        }

        for (intent_hash, user_payload_hash, invalid_at_epoch, rejection_option) in
            pending_transaction_results.into_iter()
        {
            let attempt = TransactionAttempt {
                rejection: rejection_option,
                against_state: AtState::PendingPreparingVertices {
                    base_committed_state_version: pending_transaction_base_state_version,
                },
                timestamp: pending_transaction_timestamp,
            };
            self.pending_transaction_result_cache
                .track_transaction_result(
                    intent_hash,
                    user_payload_hash,
                    invalid_at_epoch,
                    attempt,
                );
        }

        PrepareResult {
            committed,
            rejected: rejected_payloads,
            next_epoch,
            state_hash: state_tracker.into_final_state_hash(),
        }
    }
}

struct StateTracker {
    accumulator_hash: AccumulatorHash,
    state_hash: Option<StateHash>,
}

impl StateTracker {
    pub fn initial(accumulator_hash: AccumulatorHash) -> Self {
        Self {
            accumulator_hash,
            state_hash: None,
        }
    }

    pub fn accumulator_hash(&self) -> &AccumulatorHash {
        &self.accumulator_hash
    }

    pub fn update(&mut self, accumulator_hash: AccumulatorHash, state_hash: StateHash) {
        self.accumulator_hash = accumulator_hash;
        self.state_hash = Some(state_hash);
    }

    pub fn into_final_state_hash(self) -> StateHash {
        self.state_hash
            .expect("at least round update transaction must have succeeded")
    }
}

impl<'db, S> StateManager<S>
where
    S: WriteableVertexStore,
{
    pub fn save_vertex_store(&'db mut self, vertex_store: Vec<u8>) {
        self.store.save_vertex_store(vertex_store);
    }
}

impl<'db, S> StateManager<S>
where
    S: CommitStore,
    S: ReadableSubstateStore + ReadableTreeStore,
    S: QueryableProofStore + QueryableTransactionStore,
{
    pub fn commit(&'db mut self, commit_request: CommitRequest) -> Result<(), CommitError> {
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
            .store()
            .get_top_of_ledger_transaction_identifiers()
            .unwrap_or_else(CommittedTransactionIdentifiers::pre_genesis);
        if current_top_of_ledger.state_version != commit_request_start_state_version {
            panic!(
                "Mismatched state versions - the commit request claims {} but the database thinks we're at {}",
                commit_request_start_state_version,
                current_top_of_ledger.state_version
            );
        }

        let mut current_state_version = current_top_of_ledger.state_version;
        let mut parent_accumulator_hash = current_top_of_ledger.accumulator_hash;
        let mut epoch_boundary = None;
        let mut committed_transaction_bundles = Vec::new();
        let mut state_diff = StateDiff::new();
        let mut hash_tree_diff = HashTreeDiff::new();
        let mut intent_hashes = Vec::new();
        let mut final_state_hash = None;

        let parsed_txns_len = parsed_transactions.len();

        for (i, transaction) in parsed_transactions.into_iter().enumerate() {
            if let LedgerTransaction::System(..) = transaction {
                // TODO: Cleanup and use real system transaction logic
                if commit_request.proof_state_version != 1 && i != 0 {
                    panic!("Non Genesis system transaction cannot be committed.");
                }
            }

            let executable = self
                .ledger_transaction_validator
                .validate_and_create_executable(&transaction)
                .unwrap_or_else(|error| {
                    panic!(
                        "Committed transaction is not valid - likely byzantine quorum: {:?}",
                        error
                    );
                });

            let payload_hash = transaction.get_hash();
            let (current_accumulator_hash, processed) = self.execute_for_staging_with_cache(
                &parent_accumulator_hash,
                &executable,
                &payload_hash,
            );

            parent_accumulator_hash = current_accumulator_hash;

            let commit_result = match &processed.receipt().result {
                TransactionResult::Commit(result) => {
                    if let Some((_, next_epoch)) = result.next_epoch {
                        let is_last = i == (parsed_txns_len - 1);
                        if !is_last {
                            return Err(CommitError::MissingEpochProof);
                        }
                        // TODO: Use actual result and verify proof validator set matches transaction receipt validator set
                        epoch_boundary = Some(next_epoch);
                    }
                    result.clone()
                }
                TransactionResult::Reject(error) => {
                    panic!(
                        "Failed to commit a txn at state version {}: {:?}",
                        commit_request.proof_state_version, error
                    )
                }
                TransactionResult::Abort(abort_result) => {
                    panic!(
                        "Failed to commit a txn at state version {}: {:?}",
                        commit_request.proof_state_version, abort_result
                    );
                }
            };
            let ledger_receipt = LedgerTransactionReceipt::from((
                commit_result,
                processed.receipt().execution.fee_summary.clone(),
            ));

            if let LedgerTransaction::User(notarized_transaction) = &transaction {
                let intent_hash = notarized_transaction.intent_hash();
                intent_hashes.push(intent_hash);
            }

            current_state_version += 1;
            final_state_hash = Some(*processed.state_hash());

            let identifiers = CommittedTransactionIdentifiers {
                state_version: current_state_version,
                accumulator_hash: current_accumulator_hash,
            };

            committed_transaction_bundles.push((transaction, ledger_receipt, identifiers));

            // TODO: the StateDiff below should really have its own complete .extend() method (see
            // HashTreeDiff's), and this would come handy once we support substate deletes.
            state_diff
                .up_substates
                .extend(processed.state_diff().up_substates.clone());
            hash_tree_diff.extend(processed.hash_tree_diff().clone());
        }

        if let Some(state_hash) = final_state_hash {
            if state_hash != commit_request.proof_state_hash {
                warn!(
                    "computed state hash at version {} differs from the one in proof ({} != {})",
                    commit_request.proof_state_version, state_hash, commit_request.proof_state_hash
                );
            }
        }

        self.execution_cache.progress_root(&parent_accumulator_hash);

        self.store.commit(CommitBundle {
            transactions: committed_transaction_bundles,
            proof_bytes: commit_request.proof,
            proof_state_version: commit_request.proof_state_version,
            epoch_boundary,
            substates: state_diff.up_substates,
            vertex_store: commit_request.vertex_store,
            hash_tree_nodes: hash_tree_diff.new_hash_tree_nodes,
        });

        self.metrics
            .ledger_state_version
            .set(current_state_version as i64);
        self.metrics
            .ledger_transactions_committed
            .inc_by(parsed_txns_len as u64);
        self.metrics.ledger_last_update_epoch_second.set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );
        {
            let mut mempool = self.mempool.write();
            mempool.handle_committed_transactions(&intent_hashes);
            self.metrics
                .mempool_current_transactions
                .set(mempool.get_count() as i64);
        }

        self.pending_transaction_result_cache
            .track_committed_transactions(
                SystemTime::now(),
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

    pub fn get_validator_info(&self, validator_address: ComponentAddress) -> JavaValidatorInfo {
        let node_id = self
            .store()
            .global_deref(GlobalAddress::Component(validator_address))
            .unwrap();
        let substate_id = SubstateId(
            node_id,
            NodeModuleId::SELF,
            SubstateOffset::Validator(ValidatorOffset::Validator),
        );
        let output = self.store.get_substate(&substate_id).unwrap();
        let validator_substate: ValidatorSubstate = output.substate.to_runtime().into();
        JavaValidatorInfo {
            lp_token_address: validator_substate.liquidity_token,
            unstake_resource: validator_substate.unstake_nft,
        }
    }

    pub fn get_epoch(&self) -> u64 {
        self.store.get_epoch()
    }
}
