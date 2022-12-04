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
use crate::mempool::transaction_rejection_cache::{RejectionCache, RejectionReason};
use crate::query::*;
use crate::store::traits::*;
use crate::types::{CommitRequest, PrepareRequest, PrepareResult, PreviewRequest};
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, HasUserPayloadHash, IntentHash,
    LedgerTransactionReceipt, MempoolAddError, PendingTransaction,
};
use prometheus::core::Collector;
use prometheus::{IntCounter, IntCounterVec, IntGauge, Registry};
use radix_engine::engine::ScryptoInterpreter;
use radix_engine::fee::SystemLoanFeeReserve;
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    execute_and_commit_transaction, execute_preview, execute_transaction,
    execute_transaction_with_fee_reserve, ExecutionConfig, FeeReserveConfig, PreviewError,
    PreviewResult, TransactionOutcome, TransactionReceipt, TransactionResult,
};
use radix_engine::types::{
    scrypto_encode, ComponentAddress, Decimal, Decode, Encode, GlobalAddress, PublicKey, RENodeId,
    ResourceAddress, TypeId,
};
use radix_engine::wasm::{
    DefaultWasmEngine, InstructionCostRules, WasmInstrumenter, WasmMeteringConfig,
};
use radix_engine_constants::DEFAULT_MAX_CALL_DEPTH;
use radix_engine_interface::core::NetworkDefinition;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::info;

use transaction::errors::TransactionValidationError;
use transaction::model::{
    NotarizedTransaction, PreviewFlags, PreviewIntent, TransactionHeader, TransactionIntent,
};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{TestIntentHashManager, ValidationConfig};

use crate::transaction::{
    LedgerTransaction, LedgerTransactionValidator, UserTransactionValidator, ValidatorTransaction,
};

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct LoggingConfig {
    pub engine_trace: bool,
    pub state_manager_config: StateManagerLoggingConfig,
}

// TODO: Replace this with better loglevel integration
#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct StateManagerLoggingConfig {
    pub log_on_transaction_rejection: bool,
}

pub struct Counters {
    pub ledger_state_version: IntGauge,
    pub ledger_transactions_committed: IntCounter,
    pub ledger_last_update_timestamp_ms: IntGauge,
    pub mempool_current_transactions_total: IntGauge,
    pub mempool_submission_added_count: IntCounterVec,
    pub mempool_submission_rejected_count: IntCounterVec,
}

impl Counters {
    pub fn register_with(&self, registry: &Registry) {
        let metrics: Vec<Box<dyn Collector>> = vec![
            Box::new(self.ledger_state_version.clone()),
            Box::new(self.ledger_transactions_committed.clone()),
            Box::new(self.ledger_last_update_timestamp_ms.clone()),
            Box::new(self.mempool_current_transactions_total.clone()),
            Box::new(self.mempool_submission_added_count.clone()),
            Box::new(self.mempool_submission_rejected_count.clone()),
        ];

        for metric in metrics.into_iter() {
            registry.register(metric).unwrap();
        }
    }
}

impl Default for Counters {
    fn default() -> Self {
        use prometheus::opts;

        Self {
            ledger_transactions_committed: IntCounter::with_opts(opts!(
                "ledger_transactions_committed",
                "Number of transactions committed to the ledger."
            ))
            .unwrap(),
            ledger_last_update_timestamp_ms: IntGauge::with_opts(opts!(
                "ledger_last_update_timestamp_ms",
                "Last time the ledger was updated."
            ))
            .unwrap(),
            ledger_state_version: IntGauge::with_opts(opts!(
                "ledger_state_version",
                "Version of the ledger state."
            ))
            .unwrap(),
            mempool_current_transactions_total: IntGauge::with_opts(opts!(
                "mempool_current_transactions_total",
                "Number of the transactions in progress in the mempool."
            ))
            .unwrap(),
            mempool_submission_added_count: IntCounterVec::new(
                opts!(
                    "mempool_submission_added_count",
                    "Number of submissions added to the mempool."
                ),
                &["Source"],
            )
            .unwrap(),
            mempool_submission_rejected_count: IntCounterVec::new(
                opts!(
                    "mempool_submission_rejected_count",
                    "Number of the submissions rejected by the mempool."
                ),
                &["Source", "RejectionReason"],
            )
            .unwrap(),
        }
    }
}

pub struct StateManager<S> {
    pub mempool: SimpleMempool,
    pub network: NetworkDefinition,
    pub store: S,
    pub user_transaction_validator: UserTransactionValidator,
    pub ledger_transaction_validator: LedgerTransactionValidator,
    pub rejection_cache: RejectionCache,
    rounds_per_epoch: u64,
    pub counters: Counters,
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
        rounds_per_epoch: u64,
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

        let counters = Counters::default();
        let prometheus_registry = Registry::new();
        counters.register_with(&prometheus_registry);

        StateManager {
            network,
            mempool,
            store,
            user_transaction_validator,
            ledger_transaction_validator: committed_transaction_validator,
            rounds_per_epoch,
            execution_config: ExecutionConfig {
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                trace: logging_config.engine_trace,
                max_sys_call_trace_depth: 1,
            },
            scrypto_interpreter: ScryptoInterpreter {
                wasm_engine: DefaultWasmEngine::default(),
                wasm_instrumenter: WasmInstrumenter::default(),
                wasm_metering_config: WasmMeteringConfig::new(
                    InstructionCostRules::tiered(1, 5, 10, 5000),
                    512,
                ),
            },
            fee_reserve_config: FeeReserveConfig::standard(),
            intent_hash_manager: TestIntentHashManager::new(),
            logging_config: logging_config.state_manager_config,
            rejection_cache: RejectionCache::new(10000, 10000, Duration::from_secs(10)),
            counters,
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

impl<S> StateManager<S>
where
    S: ReadableSubstateStore,
    S: for<'a> TransactionIndex<&'a IntentHash> + TransactionIndex<u64> + QueryableTransactionStore,
    S: ReadableSubstateStore + QueryableSubstateStore, // Temporary - can remove when epoch validation moves to executor
{
    /// Performs static-validation, and then executes the transaction.
    /// By checking the TransactionReceipt, you can see if the transaction is presently commitable.
    fn validate_and_test_execute_transaction(
        &self,
        transaction: &NotarizedTransaction,
    ) -> Result<TransactionReceipt, TransactionValidationError> {
        let executable = self
            .user_transaction_validator
            .validate_and_create_executable(transaction)?;

        let receipt = execute_transaction(
            &self.store,
            &self.scrypto_interpreter,
            &self.fee_reserve_config,
            &self.execution_config,
            &executable,
        );

        Ok(receipt)
    }

    /// Checking if the transaction should be rejected requires full validation, ie:
    /// * Static Validation
    /// * Executing the transaction (up to loan replatment)
    ///
    /// We look for cached rejections first, to avoid this heavy lifting where we can
    fn check_for_rejection_and_add_to_mempool(
        &mut self,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        // Quick check to avoid transaction validation if it couldn't be added to the mempool anyway
        self.mempool
            .check_add_would_be_possible(&unvalidated_transaction.user_payload_hash())?;

        let rejection_check = self.check_for_rejection_with_caching(&unvalidated_transaction);

        match rejection_check {
            // Note - we purposefully don't save a validated transaction in the mempool:
            // * Currently (Nov 2022) static validation isn't sufficiently static, as it includes EG epoch validation
            // * Moreover, the engine expects the validated transaction to be presently valid, else panics
            // * Once epoch validation is moved to the executor, we can persist validated transactions in the mempool
            Ok(_) => self
                .mempool
                .add_transaction(unvalidated_transaction.into())
                .map(|_| {
                    self.counters
                        .mempool_current_transactions_total
                        .set(self.mempool.get_count() as i64)
                }),
            Err(reason) => Err(MempoolAddError::Rejected(reason)),
        }
    }

    pub fn check_for_rejection_and_add_to_mempool_from_mempool_sync(
        &mut self,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        self.check_for_rejection_and_add_to_mempool(unvalidated_transaction)
            .map(|_| {
                self.counters
                    .mempool_submission_added_count
                    .with_label_values(&["MempoolSync"])
                    .inc();
            })
            .map_err(|err| {
                let prometheus_rejection_dimension = match err {
                    MempoolAddError::Rejected(RejectionReason::FromExecution(_)) => {
                        "ExecutionError"
                    }
                    MempoolAddError::Rejected(RejectionReason::ValidationError(_)) => {
                        "ValidationError"
                    }
                    MempoolAddError::Rejected(RejectionReason::IntentHashCommitted) => {
                        "IntentHashCommitted"
                    }
                    MempoolAddError::Full { .. } => "MempoolFull",
                    MempoolAddError::Duplicate => "Duplicate",
                };
                self.counters
                    .mempool_submission_rejected_count
                    .with_label_values(&["MempoolSync", prometheus_rejection_dimension])
                    .inc();

                err
            })
    }

    pub fn check_for_rejection_and_add_to_mempool_from_core_api(
        &mut self,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        self.check_for_rejection_and_add_to_mempool(unvalidated_transaction)
            .map(|_| {
                self.counters
                    .mempool_submission_added_count
                    .with_label_values(&["CoreApi"])
                    .inc();
            })
            .map_err(|err| {
                let prometheus_rejection_dimension = match err {
                    MempoolAddError::Rejected(RejectionReason::FromExecution(_)) => {
                        "ExecutionError"
                    }
                    MempoolAddError::Rejected(RejectionReason::ValidationError(_)) => {
                        "ValidationError"
                    }
                    MempoolAddError::Rejected(RejectionReason::IntentHashCommitted) => {
                        "IntentHashCommitted"
                    }
                    MempoolAddError::Full { .. } => "MempoolFull",
                    MempoolAddError::Duplicate => "Duplicate",
                };
                self.counters
                    .mempool_submission_rejected_count
                    .with_label_values(&["CoreApi", prometheus_rejection_dimension])
                    .inc();

                err
            })
    }

    /// Reads the transaction rejection status from the cache, else calculates it fresh, by
    /// statically validating the transaction and then attempting to run it.
    ///
    /// If the transaction is freshly rejected, it is removed from the mempool and added
    /// to the rejection cache.
    pub fn check_for_rejection_with_caching(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> Result<(), RejectionReason> {
        let cached_status = self
            .rejection_cache
            .get_rejection_status(&transaction.intent_hash(), &transaction.user_payload_hash());

        if let Some(rejection_reason) = cached_status {
            return Err(rejection_reason.clone());
        }

        let new_status = self.check_for_rejection_uncached(transaction);

        if let Err(rejection_reason) = new_status {
            let payload_hash = transaction.user_payload_hash();
            // Let's also remove it from the mempool, if it's present
            if self.mempool.remove_transaction(&payload_hash).is_some() {
                self.counters
                    .mempool_current_transactions_total
                    .set(self.mempool.get_count() as i64);
            }
            self.rejection_cache.track_rejection(
                transaction.intent_hash(),
                transaction.user_payload_hash(),
                rejection_reason.clone(),
            );
            return Err(rejection_reason);
        }

        Ok(())
    }

    pub fn check_for_rejection_uncached(
        &self,
        transaction: &NotarizedTransaction,
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
            .validate_and_test_execute_transaction(transaction)
            .map_err(RejectionReason::ValidationError)?;

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
        for (hash, data) in &mempool_txns {
            let result = self.check_for_rejection_with_caching(&data.transaction.payload);
            if result.is_err() {
                txns_to_remove.push(*hash);
            }
        }

        for txn_to_remove in txns_to_remove {
            mempool_txns.remove(&txn_to_remove);
            if self.mempool.remove_transaction(&txn_to_remove).is_some() {
                self.counters
                    .mempool_current_transactions_total
                    .set(self.mempool.get_count() as i64);
            }
        }

        mempool_txns
            .into_values()
            .map(|data| data.transaction)
            .collect()
    }

    pub fn prepare(&mut self, prepare_request: PrepareRequest) -> PrepareResult {
        // This intent hash check, and current epoch should eventually live in the executor
        let mut already_committed_or_prepared_intent_hashes: HashSet<IntentHash> = HashSet::new();

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
                        .map(|_| intent_hash)
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

            if let LedgerTransaction::User(notarized_transaction) = &parsed_transaction {
                already_committed_or_prepared_intent_hashes
                    .insert(notarized_transaction.intent_hash());
            }

            let prepared_parsed_transaction = parsed_transaction.prepare();
            let executable = &self
                .ledger_transaction_validator
                .validate_and_create_executable(&prepared_parsed_transaction)
                .expect("Already prepared tranasctions should be valid");

            let receipt = execute_and_commit_transaction(
                &mut staged_store,
                &self.scrypto_interpreter,
                &self.fee_reserve_config,
                &self.execution_config,
                executable,
            );
            match receipt.result {
                TransactionResult::Commit(_) => {}
                TransactionResult::Reject(reject_result) => {
                    panic!(
                        "Already prepared transactions should be committable. Reject result: {:?}",
                        reject_result
                    )
                }
            }
        }

        let mut committed = Vec::new();

        if prepare_request.round_number % self.rounds_per_epoch == 0 {
            let new_epoch = (prepare_request.round_number / self.rounds_per_epoch) + 1;
            let epoch_update_txn = ValidatorTransaction::EpochUpdate(new_epoch);
            let prepared_epoch_update_txn = epoch_update_txn.prepare();
            let executable = prepared_epoch_update_txn.get_executable();

            let receipt = execute_transaction_with_fee_reserve(
                &staged_store,
                &self.scrypto_interpreter,
                SystemLoanFeeReserve::no_fee(),
                &self.execution_config,
                &executable,
            );
            match receipt.result {
                TransactionResult::Commit(commit_result) => {
                    if let TransactionOutcome::Failure(failure) = commit_result.outcome {
                        panic!("Epoch Update failed: {:?}", failure);
                    }

                    commit_result.state_updates.commit(&mut staged_store);

                    committed.push(
                        scrypto_encode(&LedgerTransaction::Validator(epoch_update_txn)).unwrap(),
                    );
                }
                TransactionResult::Reject(reject_result) => {
                    panic!("Epoch Update rejected: {:?}", reject_result);
                }
            }
        }

        let mut rejected = Vec::new();

        for proposed_payload in prepare_request.proposed_payloads {
            let parsed =
                match UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
                    &proposed_payload,
                ) {
                    Ok(parsed) => parsed,
                    Err(error) => {
                        rejected.push((proposed_payload, format!("{:?}", error)));
                        continue;
                    }
                };

            let intent_hash = parsed.intent_hash();
            if already_committed_or_prepared_intent_hashes.contains(&intent_hash) {
                rejected.push((
                    proposed_payload,
                    format!("Duplicate intent hash: {:?}", &intent_hash),
                ));
                continue;
            }

            let validate_result = self
                .user_transaction_validator
                .validate_and_create_executable(&parsed);

            let executable = match validate_result {
                Ok(executable) => executable,
                Err(error) => {
                    rejected.push((proposed_payload, format!("{:?}", error)));
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
                TransactionResult::Commit(..) => {
                    already_committed_or_prepared_intent_hashes.insert(intent_hash);
                    committed.push(LedgerTransaction::User(parsed).create_payload().unwrap());
                }
                TransactionResult::Reject(reject_result) => {
                    rejected.push((proposed_payload, format!("{:?}", reject_result)));
                }
            };
        }

        if self.logging_config.log_on_transaction_rejection {
            for rejection in rejected.iter() {
                info!("TXN INVALID: {}", rejection.1);
            }
        }

        PrepareResult {
            rejected,
            committed,
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

    pub fn commit(&'db mut self, commit_request: CommitRequest) {
        let mut to_store = Vec::new();
        let mut payload_hashes = Vec::new();
        let mut intent_hashes = Vec::new();
        let commit_request_start_state_version =
            commit_request.state_version - (commit_request.transaction_payloads.len() as u64);

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

        let prepared_parsed_transactions = parsed_transactions
            .iter()
            .map(|parsed| parsed.prepare())
            .collect::<Vec<_>>();

        let current_top_of_ledger = self
            .store
            .get_top_of_ledger_transaction_identifiers_unwrap(); // Genesis must have happened to be here
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

        let receipts = prepared_parsed_transactions
            .iter()
            .map(|prepared| {
                let executable = self
                    .ledger_transaction_validator
                    .validate_and_create_executable(prepared)
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

                let ledger_receipt: LedgerTransactionReceipt =
                    engine_receipt.try_into().unwrap_or_else(|error| {
                        panic!(
                            "Failed to commit a txn at state version {}: {}",
                            commit_request.state_version, error
                        )
                    });
                ledger_receipt
            })
            .collect::<Vec<_>>();

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
            commit_request.state_version,
            payload_hashes,
            commit_request.proof,
        );
        if let Some(vertex_store) = commit_request.vertex_store {
            db_transaction.save_vertex_store(vertex_store);
        }

        db_transaction.commit();
        self.counters
            .ledger_state_version
            .set(current_state_version as i64);
        self.counters
            .ledger_transactions_committed
            .inc_by(committed_transactions_count as u64);
        self.counters.ledger_last_update_timestamp_ms.set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        );
        self.mempool.handle_committed_transactions(&intent_hashes);
        self.counters
            .mempool_current_transactions_total
            .set(self.mempool.get_count() as i64);

        self.rejection_cache
            .track_committed_transactions(intent_hashes);
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
