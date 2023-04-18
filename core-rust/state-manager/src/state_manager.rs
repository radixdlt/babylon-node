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

use crate::mempool::simple_mempool::SimpleMempool;
use crate::query::*;
use crate::staging::{ExecutionCache, HashStructuresDiff, ReadableStore, TransactionLogic};
use crate::store::traits::*;
use crate::transaction::{
    LedgerTransaction, LedgerTransactionValidator, UserTransactionValidator, ValidatorTransaction,
};
use crate::types::{CommitRequest, PrepareRequest, PrepareResult, PreviewRequest};
use crate::*;
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, IntentHash, MempoolAddError, PendingTransaction,
};
use ::transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use ::transaction::errors::TransactionValidationError;
use ::transaction::model::{
    Executable, NotarizedTransaction, PreviewFlags, PreviewIntent, TransactionHeader,
    TransactionIntent,
};
use ::transaction::validation::{TestIntentHashManager, ValidationConfig};
use parking_lot::RwLock;
use prometheus::Registry;
use radix_engine::transaction::{
    execute_preview, execute_transaction, AbortReason, ExecutionConfig, FeeReserveConfig,
    PreviewError, PreviewResult, TransactionReceipt, TransactionResult,
};
use radix_engine::types::{Categorize, ComponentAddress, Decode, Encode, PublicKey};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};

use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::Arc;

use crate::staging::epoch_handling::AccuTreeEpochHandler;

use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine_interface::data::manifest::manifest_encode;
use radix_engine_interface::network::NetworkDefinition;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

use crate::mempool_relay_dispatcher::MempoolRelayDispatcher;

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

const UP_TO_FEE_LOAN_TRANSACTION_WARN_TIME_LIMIT: Duration = Duration::from_millis(100);
const FULL_TRANSACTION_WARN_TIME_LIMIT: Duration = Duration::from_millis(500);

pub struct StateManager<S> {
    mempool: Arc<RwLock<SimpleMempool>>,
    mempool_relay_dispatcher: MempoolRelayDispatcher,
    network: NetworkDefinition,
    store: Arc<RwLock<S>>,
    execution_cache: ExecutionCache,
    pub user_transaction_validator: UserTransactionValidator,
    ledger_transaction_validator: LedgerTransactionValidator,
    pub pending_transaction_result_cache: PendingTransactionResultCache,
    ledger_metrics: LedgerMetrics,
    execution_config: ExecutionConfig,
    execution_config_for_pending_transactions: ExecutionConfig,
    execution_config_for_genesis: ExecutionConfig,
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: FeeReserveConfig,
    intent_hash_manager: TestIntentHashManager,
    logging_config: StateManagerLoggingConfig,
}

impl<S> StateManager<S>
where
    S: ReadableSubstateStore + TransactionIdentifierLoader,
{
    pub fn new(
        network: NetworkDefinition,
        store: Arc<RwLock<S>>,
        mempool: Arc<RwLock<SimpleMempool>>,
        mempool_relay_dispatcher: MempoolRelayDispatcher,
        logging_config: LoggingConfig,
        metric_registry: &Registry,
    ) -> StateManager<S> {
        let user_transaction_validator = UserTransactionValidator {
            validation_config: ValidationConfig::default(network.id),
            intent_hash_manager: TestIntentHashManager::new(),
        };

        let committed_transaction_validator = LedgerTransactionValidator {
            validation_config: ValidationConfig::default(network.id),
            intent_hash_manager: TestIntentHashManager::new(),
        };

        let accumulator_hash = store
            .read()
            .get_top_transaction_identifiers()
            .accumulator_hash;

        StateManager {
            network,
            mempool,
            mempool_relay_dispatcher,
            store,
            execution_cache: ExecutionCache::new(accumulator_hash),
            user_transaction_validator,
            ledger_transaction_validator: committed_transaction_validator,
            execution_config: ExecutionConfig::standard().with_trace(logging_config.engine_trace),
            execution_config_for_pending_transactions: ExecutionConfig::up_to_loan_repayment()
                .with_trace(logging_config.engine_trace),
            execution_config_for_genesis: ExecutionConfig::genesis()
                .with_trace(logging_config.engine_trace),
            scrypto_interpreter: ScryptoInterpreter {
                wasm_engine: DefaultWasmEngine::default(),
                wasm_instrumenter: WasmInstrumenter::default(),
                wasm_metering_config: WasmMeteringConfig::default(),
            },
            fee_reserve_config: FeeReserveConfig::standard(),
            intent_hash_manager: TestIntentHashManager::new(),
            logging_config: logging_config.state_manager_config,
            pending_transaction_result_cache: PendingTransactionResultCache::new(10000, 10000),
            ledger_metrics: LedgerMetrics::new(metric_registry),
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

        let start = Instant::now();

        let result = execute_preview(
            self.store.read().deref(),
            &self.scrypto_interpreter,
            &self.intent_hash_manager,
            &self.network,
            preview_intent,
        );

        let elapsed = start.elapsed();

        if elapsed > FULL_TRANSACTION_WARN_TIME_LIMIT {
            warn!(
                "Preview execution took {}ms, above warning threshold of {}ms",
                elapsed.as_millis(),
                FULL_TRANSACTION_WARN_TIME_LIMIT.as_millis()
            );
        }

        result
    }
}

pub enum StateManagerRejectReason {
    TransactionValidationError(TransactionValidationError),
}

/// A result of `StateManager::check_transactions_to_relay()`.
pub struct PendingTransactionBatchCheckResult {
    /// A subset of candidate transactions that is appropriate to relaying via a mempool sync.
    pub to_relay: Vec<PendingTransaction>,
    /// A subset of candidate transactions that was discovered to be invalid (and should be removed
    /// from the mempool).
    /// Note: this is not the same as "all candidate transactions except the ones selected to relay"
    /// since some (valid or invalid) candidates might have not been considered at all (due to
    /// mempool sync relay limits).
    pub to_remove: Vec<PendingTransaction>,
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
    /// Performs static-validation, and then executes the transaction.
    /// By checking the TransactionReceipt, you can see if the transaction is presently commitable.
    fn validate_and_test_execute_transaction_up_to_fee_loan(
        &self,
        root_store: &S,
        transaction: &NotarizedTransaction,
        payload_size: usize,
    ) -> Result<TransactionReceipt, StateManagerRejectReason> {
        let executable = self
            .user_transaction_validator
            .validate_and_create_executable(transaction, payload_size)
            .map_err(StateManagerRejectReason::TransactionValidationError)?;
        let transaction_logic = TimeWarningTransactionLogic::wrap(
            ConfiguredExecutable::new(
                &executable,
                &self.scrypto_interpreter,
                &self.fee_reserve_config,
                &self.execution_config_for_pending_transactions,
            ),
            UP_TO_FEE_LOAN_TRANSACTION_WARN_TIME_LIMIT,
            format!(
                "pending intent hash {}, up to fee loan",
                transaction.intent_hash()
            ),
        );
        Ok(transaction_logic.execute_on(root_store))
    }

    /// Adds the given transaction to the mempool (applying all the checks and caching, see
    /// `check_for_rejection_and_add_to_mempool()`), and then triggers an unscheduled mempool sync
    /// (propagating only this transaction to other nodes).
    /// The triggering only takes place if the mempool did not already contain this transaction (to
    /// prevent flooding). Any error encountered during the triggering will only be logged (as
    /// `warn!`) and then ignored.
    /// Although an arbitrary `MempoolAddSource` can be passed, this method is primarily meant for
    /// relaying new transactions submitted via Core API.
    pub fn add_to_mempool_and_trigger_relay(
        &mut self,
        source: MempoolAddSource,
        transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        self.check_for_rejection_and_add_to_mempool(source, transaction.clone())?;
        if let Err(error) = self.mempool_relay_dispatcher.trigger_relay(transaction) {
            warn!("Could not trigger a mempool relay: {:?}; ignoring", error);
        }
        Ok(())
    }

    /// Checking if the transaction should be rejected requires full validation, ie:
    /// * Static Validation
    /// * Executing the transaction (up to loan repayment)
    ///
    /// We look for cached rejections first, to avoid this heavy lifting where we can
    pub fn check_for_rejection_and_add_to_mempool(
        &mut self,
        source: MempoolAddSource,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        // Quick check to avoid transaction validation if it couldn't be added to the mempool anyway
        let mut write_mempool = self.mempool.write();
        write_mempool.check_add_would_be_possible(&unvalidated_transaction.user_payload_hash())?;
        drop(write_mempool);

        let (record, was_cached) = self.check_for_rejection_with_caching(&unvalidated_transaction);
        let result = record
            .should_accept_into_mempool(was_cached)
            .map_err(MempoolAddError::Rejected);

        let mut write_mempool = self.mempool.write();
        match &result {
            Ok(_) => {
                // Note - we purposefully don't save a validated transaction in the mempool:
                // * Currently (Nov 2022) static validation isn't sufficiently static, as it includes EG epoch validation
                // * Moreover, the engine expects the validated transaction to be presently valid, else panics
                // * Once epoch validation is moved to the executor, we can persist validated transactions in the mempool
                write_mempool.add_transaction(unvalidated_transaction.into(), source)?;
            }
            Err(error) => {
                write_mempool.record_rejection(source, error);
            }
        };

        result
    }

    /// Reads the transaction rejection status from the cache, else calculates it fresh, by
    /// statically validating the transaction and then attempting to run it.
    ///
    /// The result is stored in the cache.
    /// If the transaction is freshly rejected, the caller should perform additional cleanup,
    /// e.g. removing the transaction from the mempool
    ///
    /// Its pending transaction record is returned, along with a boolean about whether the last attempt was cached.
    fn check_for_rejection_with_caching(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> (PendingTransactionRecord, bool) {
        let read_store = self.store.read();
        let current_epoch = read_store.get_epoch();
        let max_state_version = read_store.max_state_version();

        let current_time = SystemTime::now();
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
        let payload_size = manifest_encode(transaction).unwrap().len();
        let rejection = self
            .check_for_rejection_uncached(transaction, payload_size)
            .err();

        let attempt = TransactionAttempt {
            rejection,
            against_state: AtState::Committed {
                state_version: max_state_version,
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
        let read_store = self.store.read();
        let existing = read_store.get_txn_state_version_by_identifier(&transaction.intent_hash());
        if existing.is_some() {
            return Err(RejectionReason::IntentHashCommitted);
        }

        let receipt = self
            .validate_and_test_execute_transaction_up_to_fee_loan(
                read_store.deref(),
                transaction,
                payload_size,
            )
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

    /// Checks the given candidate transactions for rejection and decides which ones should be
    /// relayed via mempool sync, and which ones should be removed from mempool.
    pub fn check_transactions_to_relay(
        &mut self,
        mut candidate_transactions: Vec<PendingTransaction>,
        max_num_txns: usize,
        max_payload_size_bytes: usize,
    ) -> PendingTransactionBatchCheckResult {
        candidate_transactions.shuffle(&mut thread_rng());

        let mut to_relay = Vec::new();
        let mut payload_size_so_far = 0usize;

        // We (partially) cleanup the mempool on the occasion of getting the relay txns
        // TODO: move this to a separate job
        let mut to_remove = Vec::new();

        for candidate_transaction in candidate_transactions.into_iter() {
            let (record, _) = self.check_for_rejection_with_caching(&candidate_transaction.payload);
            if record.latest_attempt.rejection.is_some() {
                // Mark the transaction to be removed from the mempool
                // (see the comment above about moving this to a separate job)
                to_remove.push(candidate_transaction);
            } else {
                // Check the payload size limit
                payload_size_so_far += candidate_transaction.payload_size;
                if payload_size_so_far > max_payload_size_bytes {
                    break;
                }

                // Add the transaction to response
                to_relay.push(candidate_transaction);
                if to_relay.len() >= max_num_txns {
                    break;
                }
            }
        }

        PendingTransactionBatchCheckResult {
            to_relay,
            to_remove,
        }
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

        let processed = self.execution_cache.execute_transaction(
            self.store.read().deref(),
            &EpochTransactionIdentifiers::pre_genesis(),
            &CommittedTransactionIdentifiers::pre_genesis(),
            &parsed_transaction.get_hash(),
            &wrap_transaction_logic(
                &executable,
                "genesis".to_string(),
                &self.scrypto_interpreter,
                &self.fee_reserve_config,
                &self.execution_config_for_genesis,
            ),
        );

        let commit = processed.expect_commit("genesis");
        commit.check_success("genesis");
        PrepareGenesisResult {
            validator_set: commit
                .next_epoch()
                .map(|next_epoch| next_epoch.validator_set),
            ledger_hashes: commit.hash_structures_diff.ledger_hashes,
        }
    }

    pub fn prepare(&mut self, prepare_request: PrepareRequest) -> PrepareResult {
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
            let processed = self.execution_cache.execute_transaction(
                read_store.deref(),
                &epoch_identifiers,
                state_tracker.latest_transaction_identifiers(),
                &transaction_hash,
                &wrap_transaction_logic(
                    &executable,
                    format!("already prepared {}", transaction_hash),
                    &self.scrypto_interpreter,
                    &self.fee_reserve_config,
                    &self.execution_config,
                ),
            );

            let commit = processed.expect_commit("already prepared");
            // TODO: Do we need to check that next epoch request has been prepared?
            state_tracker.update(&commit.hash_structures_diff);
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

        let processed_round_update = self.execution_cache.execute_transaction(
            read_store.deref(),
            &epoch_identifiers,
            state_tracker.latest_transaction_identifiers(),
            &ledger_round_update.get_hash(),
            &wrap_transaction_logic(
                &round_update.prepare().to_executable(),
                format!("round update {}", prepare_request.round_number),
                &self.scrypto_interpreter,
                &self.fee_reserve_config,
                &self.execution_config,
            ),
        );

        let round_update_commit = processed_round_update.expect_commit("round update");
        round_update_commit.check_success("round update");
        state_tracker.update(&round_update_commit.hash_structures_diff);
        committed.push(manifest_encode(&ledger_round_update).unwrap());

        let mut next_epoch = round_update_commit.next_epoch();
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

                let processed = self.execution_cache.execute_transaction(
                    read_store.deref(),
                    &epoch_identifiers,
                    state_tracker.latest_transaction_identifiers(),
                    &hash,
                    &wrap_transaction_logic(
                        &executable,
                        format!("newly proposed {}", hash),
                        &self.scrypto_interpreter,
                        &self.fee_reserve_config,
                        &self.execution_config,
                    ),
                );

                match processed.expect_commit_or_reject("prepared") {
                    Ok(commit) => {
                        state_tracker.update(&commit.hash_structures_diff);

                        already_committed_or_prepared_intent_hashes
                            .insert(intent_hash, AlreadyPreparedTransaction::Proposed);
                        committed.push(payload);
                        pending_transaction_results.push((
                            intent_hash,
                            user_payload_hash,
                            invalid_at_epoch,
                            None,
                        ));
                        next_epoch = commit.next_epoch();
                    }
                    Err(reject) => {
                        rejected_payloads.push((proposed_payload, format!("{:?}", reject)));
                        pending_transaction_results.push((
                            intent_hash,
                            user_payload_hash,
                            invalid_at_epoch,
                            Some(RejectionReason::FromExecution(Box::new(
                                reject.error.clone(),
                            ))),
                        ));
                    }
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
            .filter(|(_, _, _, rejection)| rejection.is_some())
            .map(|(intent_hash, user_payload_hash, _, _)| (intent_hash, user_payload_hash))
            .collect::<Vec<_>>();
        if !pending_rejected_transactions.is_empty() {
            let mut write_mempool = self.mempool.write();
            for (intent_hash, user_payload_hash) in pending_rejected_transactions {
                // Removing transactions rejected during prepare from the mempool is a bit of overkill:
                // just because transactions were rejected in this history doesn't mean this history will be committed.
                //
                // But it'll do for now as a defensive measure until we can have a more intelligent mempool.
                write_mempool.remove_transaction(intent_hash, user_payload_hash);
            }
        }

        for (intent_hash, user_payload_hash, invalid_at_epoch, rejection_option) in
            pending_transaction_results.into_iter()
        {
            let attempt = TransactionAttempt {
                rejection: rejection_option,
                against_state: pending_transaction_base_state.clone(),
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

impl<'db, S> StateManager<S>
where
    S: CommitStore,
    S: ReadableStore,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    pub fn commit(&'db mut self, commit_request: CommitRequest) -> Result<(), CommitError> {
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

        for (i, transaction) in parsed_transactions.into_iter().enumerate() {
            if let LedgerTransaction::System(..) = transaction {
                // TODO: Cleanup and use real system transaction logic
                if commit_state_version != 1 && i != 0 {
                    panic!("Non Genesis system transaction cannot be committed.");
                }
            }

            let executable = self
                .ledger_transaction_validator
                .validate_and_create_executable(&transaction)
                .unwrap_or_else(|error| {
                    panic!(
                        "Committed transaction is not valid - likely byzantine quorum: {error:?}"
                    );
                });

            let transaction_hash = transaction.get_hash();
            let processed = self.execution_cache.execute_transaction(
                write_store.deref(),
                &epoch_identifiers,
                state_tracker.latest_transaction_identifiers(),
                &transaction_hash,
                &wrap_transaction_logic(
                    &executable,
                    format!("committing {}", transaction_hash),
                    &self.scrypto_interpreter,
                    &self.fee_reserve_config,
                    &self.execution_config,
                ),
            );

            let commit =
                processed.expect_commit(&format!("at state version {}", commit_state_version));

            let is_last_transaction_in_request = i == (commit_transactions_len - 1);
            if !is_last_transaction_in_request && commit.next_epoch().is_some() {
                return Err(CommitError::MissingEpochProof);
            }

            // TODO: verify that `result.next_epoch == commit_ledger_header.next_epoch`
            // (currently it would fail for some of our tests which create genesis proof
            // directly, without caring about validator addresses)

            if let LedgerTransaction::User(notarized_transaction) = &transaction {
                let intent_hash = notarized_transaction.intent_hash();
                intent_hashes.push(intent_hash);
            }

            let hash_structures_diff = &commit.hash_structures_diff;
            state_tracker.update(hash_structures_diff);

            committed_transaction_bundles.push((
                transaction,
                commit.local_receipt.clone(),
                state_tracker.latest_transaction_identifiers().clone(),
            ));

            substate_store_update.apply(&commit.local_receipt.on_ledger.substate_changes);
            state_tree_update.add(
                state_tracker.latest_transaction_identifiers().state_version,
                &hash_structures_diff.state_hash_tree_diff,
            );
            transaction_tree_slice_merger
                .append(hash_structures_diff.transaction_tree_diff.slice.clone());
            receipt_tree_slice_merger.append(hash_structures_diff.receipt_tree_diff.slice.clone());
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

        self.execution_cache
            .progress_root(&final_transaction_identifiers.accumulator_hash);

        write_store.commit(CommitBundle {
            transactions: committed_transaction_bundles,
            proof: commit_request.proof,
            substate_store_update,
            vertex_store: commit_request.vertex_store,
            state_tree_update,
            transaction_tree_slice: transaction_tree_slice_merger.into_slice(),
            receipt_tree_slice: receipt_tree_slice_merger.into_slice(),
        });

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
        let mut write_mempool = self.mempool.write();
        write_mempool.handle_committed_transactions(&intent_hashes);
        drop(write_mempool);

        self.pending_transaction_result_cache
            .track_committed_transactions(
                SystemTime::now(),
                commit_request_start_state_version,
                intent_hashes,
            );

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

fn wrap_transaction_logic<'a>(
    executable: &'a Executable<'a>,
    logged_description: String,
    scrypto_interpreter: &'a ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: &'a FeeReserveConfig,
    execution_config: &'a ExecutionConfig,
) -> TimeWarningTransactionLogic<ConfiguredExecutable<'a>> {
    TimeWarningTransactionLogic::wrap(
        ConfiguredExecutable::new(
            executable,
            scrypto_interpreter,
            fee_reserve_config,
            execution_config,
        ),
        FULL_TRANSACTION_WARN_TIME_LIMIT,
        logged_description,
    )
}

struct ConfiguredExecutable<'a> {
    executable: &'a Executable<'a>,
    scrypto_interpreter: &'a ScryptoInterpreter<DefaultWasmEngine>,
    fee_reserve_config: &'a FeeReserveConfig,
    execution_config: &'a ExecutionConfig,
}

impl<'a> ConfiguredExecutable<'a> {
    pub fn new(
        executable: &'a Executable<'a>,
        scrypto_interpreter: &'a ScryptoInterpreter<DefaultWasmEngine>,
        fee_reserve_config: &'a FeeReserveConfig,
        execution_config: &'a ExecutionConfig,
    ) -> Self {
        Self {
            executable,
            scrypto_interpreter,
            fee_reserve_config,
            execution_config,
        }
    }
}

impl<'a, S: ReadableSubstateStore> TransactionLogic<S> for ConfiguredExecutable<'a> {
    fn execute_on(&self, store: &S) -> TransactionReceipt {
        execute_transaction(
            store,
            self.scrypto_interpreter,
            self.fee_reserve_config,
            self.execution_config,
            self.executable,
        )
    }
}

struct TimeWarningTransactionLogic<U> {
    underlying: U,
    time_limit: Duration,
    description: String, // for error-surfacing only
}

impl<U> TimeWarningTransactionLogic<U> {
    pub fn wrap(underlying: U, time_limit: Duration, description: String) -> Self {
        Self {
            underlying,
            time_limit,
            description,
        }
    }
}

impl<U, S> TransactionLogic<S> for TimeWarningTransactionLogic<U>
where
    S: ReadableSubstateStore,
    U: TransactionLogic<S>,
{
    fn execute_on(&self, store: &S) -> TransactionReceipt {
        let start = Instant::now();
        let result = self.underlying.execute_on(store);
        let elapsed = start.elapsed();
        if elapsed > self.time_limit {
            warn!(
                "Transaction execution took {}ms, above warning threshold of {}ms ({})",
                elapsed.as_millis(),
                self.time_limit.as_millis(),
                self.description
            );
        }
        result
    }
}
