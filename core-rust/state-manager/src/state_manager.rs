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
use crate::query::ResourceAccounter;
use crate::store::traits::*;
use crate::types::{
    CommitRequest, PrepareRequest, PrepareResult, PreviewRequest, StoredTransaction,
};
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, HasPayloadHash, IntentHash,
    LedgerTransactionReceipt, MempoolAddError, PendingTransaction,
};
use radix_engine::constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_MAX_CALL_DEPTH, DEFAULT_SYSTEM_LOAN,
};
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    ExecutionConfig, FeeReserveConfig, PreviewError, PreviewExecutor, PreviewResult,
    TransactionExecutor, TransactionReceipt, TransactionResult,
};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter};
use scrypto::engine::types::{RENodeId, SubstateId};
use scrypto::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;
use radix_engine::fee::SystemLoanFeeReserve;
use tracing::info;

use transaction::errors::TransactionValidationError;
use transaction::model::{NotarizedTransaction, PreviewFlags, PreviewIntent, Transaction, TransactionHeader, TransactionIntent, Validated};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{NetworkTransactionValidator, TestIntentHashManager, TransactionValidator, ValidationConfig};
use transaction::validation::MAX_PAYLOAD_SIZE;

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

pub struct TransactionValidation {
    validator: NetworkTransactionValidator,
    intent_hash_manager: TestIntentHashManager,
}

impl TransactionValidation {
    /// Checks the Payload max size, and SBOR decodes to a NotarizedTransaction if the size is okay
    pub fn parse_unvalidated_user_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<NotarizedTransaction, TransactionValidationError> {
        if transaction_payload.len() > MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: NotarizedTransaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    /// Performs static validation only
    pub fn parse_and_validate_user_transaction_slice(
        &self,
        transaction_payload: &[u8],
    ) -> Result<Validated<NotarizedTransaction>, TransactionValidationError> {
        let notarized_transaction = Self::parse_unvalidated_user_transaction_from_slice(transaction_payload)?;
        self.validate_user_transaction(notarized_transaction)
    }

    /// Performs static validation only
    fn validate_user_transaction(
        &self,
        transaction: NotarizedTransaction,
    ) -> Result<Validated<NotarizedTransaction>, TransactionValidationError> {
        self.validator.validate(transaction, &self.intent_hash_manager)
    }

    pub fn parse_unvalidated_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<Transaction, TransactionValidationError> {
        let transaction: Transaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    pub fn parse_and_validate_transaction_slice(
        &self,
        transaction_payload: &[u8],
    ) -> Result<Validated<Transaction>, TransactionValidationError> {
        let transaction = Self::parse_unvalidated_transaction_from_slice(transaction_payload)?;
        self.validate_transaction(transaction)
    }

    fn validate_transaction(&self, transaction: Transaction) -> Result<Validated<Transaction>, TransactionValidationError> {
        self.validator.validate(transaction, &self.intent_hash_manager)
    }
}

pub struct StateManager<S> {
    pub mempool: SimpleMempool,
    pub network: NetworkDefinition,
    pub store: S,
    pub validation: TransactionValidation,
    pub rejection_cache: RejectionCache,
    wasm_engine: DefaultWasmEngine,
    wasm_instrumenter: WasmInstrumenter,
    execution_config: ExecutionConfig,
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
        let validation = TransactionValidation {
            validator: NetworkTransactionValidator::new(ValidationConfig {
                network_id: network.id,
                current_epoch: 1,
                max_cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
                min_tip_percentage: 0,
            }),
            intent_hash_manager: TestIntentHashManager::new(),
        };

        StateManager {
            network,
            mempool,
            store,
            wasm_engine: DefaultWasmEngine::new(),
            wasm_instrumenter: WasmInstrumenter::new(),
            validation,
            execution_config: ExecutionConfig {
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                trace: logging_config.engine_trace,
            },
            fee_reserve_config: FeeReserveConfig {
                cost_unit_price: DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
                system_loan: DEFAULT_SYSTEM_LOAN,
            },
            intent_hash_manager: TestIntentHashManager::new(),
            logging_config: logging_config.state_manager_config,
            rejection_cache: RejectionCache::new(10000, 10000, Duration::from_secs(10)),
        }
    }

}


impl<S> StateManager<S>
where
    S: ReadableSubstateStore,
{
    pub fn preview(
        &mut self,
        preview_request: PreviewRequest,
    ) -> Result<PreviewResult, PreviewError> {
        // not really used for preview
        let notary_private_key = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();

        let preview_intent = PreviewIntent {
            intent: TransactionIntent {
                header: TransactionHeader {
                    version: 1,
                    network_id: self.network.id,
                    start_epoch_inclusive: 0,
                    end_epoch_exclusive: 100,
                    nonce: preview_request.nonce,
                    notary_public_key: PublicKey::EcdsaSecp256k1(notary_private_key.public_key()),
                    notary_as_signatory: false,
                    cost_unit_limit: preview_request.cost_unit_limit,
                    tip_percentage: preview_request.tip_percentage,
                },
                manifest: preview_request.manifest,
            },
            signer_public_keys: preview_request.signer_public_keys,
            flags: PreviewFlags {
                unlimited_loan: preview_request.flags.unlimited_loan,
            },
        };

        PreviewExecutor::new(
            &mut self.store,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
            &self.intent_hash_manager,
            &self.network,
        )
        .execute(preview_intent)
    }
}

enum ValidationResult {
    Ok {
        validated: Validated<NotarizedTransaction>,
        payload: Vec<u8>,
    },
    Err {
        error: TransactionValidationError,
        payload: Vec<u8>
    }
}

impl<S> StateManager<S>
where
    S: ReadableSubstateStore,
    S: QueryableTransactionStore,
{
    /// Performs static-validation, and then executes the transaction.
    /// By checking the TransactionReceipt, you can see if the transaction is presently commitable.
    fn validate_and_test_execute_transaction(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> Result<TransactionReceipt, TransactionValidationError> {
        let validated_transaction = self.validation.validate_user_transaction(transaction.clone())?;

        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);
        let mut staged_store = staged_store_manager.get_output_store(staged_node);

        let mut transaction_executor = TransactionExecutor::new(
            &mut staged_store,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
        );
        let receipt = transaction_executor.execute(
            &validated_transaction,
            &self.fee_reserve_config,
            &self.execution_config,
        );
        Ok(receipt)
    }

    /// Checking if the transaction should be rejected requires full validation, ie:
    /// * Static Validation
    /// * Executing the transaction (up to loan replatment)
    ///
    /// We look for cached rejections first, to avoid this heavy lifting where we can
    pub fn check_for_rejection_and_add_to_mempool(
        &mut self,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        // Quick check to avoid transaction validation if it couldn't be added to the mempool anyway
        self.mempool
            .check_add_would_be_possible(&unvalidated_transaction.payload_hash())?;

        let rejection_check = self.check_for_rejection_with_caching(&unvalidated_transaction);

        match rejection_check {
            // Note - we purposefully don't save a validated transaction in the mempool:
            // * Currently (Nov 2022) static validation isn't sufficiently static, as it includes EG epoch validation
            // * Moreover, the engine expects the validated transaction to be presently valid, else panics
            // * Once epoch validation is moved to the executor, we can persist validated transactions in the mempool
            Ok(_) => self.mempool.add_transaction(unvalidated_transaction.into()),
            Err(reason) => Err(MempoolAddError::Rejected(reason)),
        }
    }

    /// Reads the transaction rejection status from the cache, else calculates it fresh, by
    /// statically validating the transaction and then attempting to run it.
    ///
    /// If the transaction is freshly rejected, it is removed from the mempool and added
    /// to the rejection cache.
    fn check_for_rejection_with_caching(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> Result<(), RejectionReason> {
        let cached_status = self
            .rejection_cache
            .get_rejection_status(&transaction.intent_hash(), &transaction.payload_hash());

        if let Some(rejection_reason) = cached_status {
            return Err(rejection_reason.clone());
        }

        let new_status = self.check_for_rejection_uncached(transaction);

        if let Err(rejection_reason) = new_status {
            let payload_hash = transaction.payload_hash();
            // Let's also remove it from the mempool, if it's present
            self.mempool.remove_transaction(&payload_hash);
            self.rejection_cache.track_rejection(
                transaction.intent_hash(),
                transaction.payload_hash(),
                rejection_reason.clone(),
            );
            return Err(rejection_reason);
        }

        Ok(())
    }

    fn check_for_rejection_uncached(
        &mut self,
        transaction: &NotarizedTransaction,
    ) -> Result<(), RejectionReason> {
        if self
            .store
            .get_committed_transaction_by_intent(&transaction.intent_hash())
            .is_some()
        {
            return Err(RejectionReason::IntentHashCommitted);
        }

        // TODO: Only run transaction up to the loan
        let receipt = self
            .validate_and_test_execute_transaction(transaction)
            .map_err(RejectionReason::ValidationError)?;

        match receipt.result {
            TransactionResult::Reject(result) => Err(RejectionReason::FromExecution(result.error)),
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
            self.mempool.remove_transaction(&txn_to_remove);
        }

        mempool_txns
            .into_values()
            .map(|data| data.transaction)
            .collect()
    }

    pub fn prepare(&mut self, prepare_request: PrepareRequest) -> PrepareResult {
        let mut validated_prepared = Vec::new();

        // This intent hash check should eventually live in the executor
        let mut already_committed_or_prepared_intent_hashes: HashSet<IntentHash> = HashSet::new();

        for prepared in prepare_request.already_prepared_payloads {
            let validated_transaction = self
                .validation
                .parse_and_validate_transaction_slice(&prepared)
                .expect("Already prepared transactions should be decodeable");

            if let Transaction::User(notarized_transaction) = validated_transaction.transaction() {
                already_committed_or_prepared_intent_hashes.insert(notarized_transaction.intent_hash());
            }
            validated_prepared.push(validated_transaction);
        }

        let mut validated_proposed_transactions : Vec<ValidationResult> = Vec::new();

        for proposed_payload in prepare_request.proposed_payloads {
            let validation_result = self
                .validation
                .parse_and_validate_user_transaction_slice(&proposed_payload);

            if let Ok(validated_transaction) = &validation_result {
                let intent_hash = validated_transaction.intent_hash();
                if self
                    .store
                    .get_committed_transaction_by_intent(&intent_hash)
                    .is_some()
                {
                    already_committed_or_prepared_intent_hashes.insert(intent_hash);
                }
            }

            let validation_result = validation_result.map(|validated| {
                ValidationResult::Ok { validated, payload: proposed_payload.clone() }
            }).unwrap_or_else(|error| ValidationResult::Err { error, payload: proposed_payload.clone() });

            validated_proposed_transactions.push(validation_result);
        }

        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);
        let mut staged_store = staged_store_manager.get_output_store(staged_node);
        let mut transaction_executor = TransactionExecutor::new(
            &mut staged_store,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
        );

        for prepared in validated_prepared {
            transaction_executor.execute_and_commit(
                &prepared,
                &self.fee_reserve_config,
                &self.execution_config,
            );
        }

        let mut committed = Vec::new();
        let epoch_update_txn = Transaction::EpochUpdate(prepare_request.round_number);
        let validated_epoch_update_txn = self.validation.validate_transaction(epoch_update_txn).unwrap();
        let mut fee_reserve = SystemLoanFeeReserve::default();
        // TODO: Clean up fee reserve
        fee_reserve.credit(10_000_000);
        let receipt = transaction_executor.execute_with_fee_reserve(&validated_epoch_update_txn, &self.execution_config, fee_reserve);
        match receipt.result {
            TransactionResult::Commit(..) => {
                let serialized_validated_txn = scrypto_encode(validated_epoch_update_txn.transaction());
                committed.push(serialized_validated_txn);
            }
            TransactionResult::Reject(reject_result) => {
                panic!("Epoch Update rejected: {:?}", reject_result);
            }
        }

        let mut rejected = Vec::new();

        for validation_result in validated_proposed_transactions {
            match validation_result {
                ValidationResult::Ok { validated, payload } => {
                    let intent_hash = validated.intent_hash();
                    if already_committed_or_prepared_intent_hashes.contains(&intent_hash) {
                        rejected.push((
                            payload,
                            "Transaction rejected - duplicate intent hash".to_string(),
                        ));
                        continue;
                    }

                    let receipt = transaction_executor.execute_and_commit(
                        &validated,
                        &self.fee_reserve_config,
                        &self.execution_config,
                    );

                    match receipt.result {
                        TransactionResult::Commit(..) => {
                            already_committed_or_prepared_intent_hashes.insert(validated.intent_hash());
                            let serialized_validated_txn = scrypto_encode(&Transaction::User(validated.into_transaction()));
                            committed.push(serialized_validated_txn);
                        }
                        TransactionResult::Reject(reject_result) => {
                            rejected.push((payload, format!("{:?}", reject_result)));

                            if self.logging_config.log_on_transaction_rejection {
                                info!("TXN REJECTED: {:?}", reject_result);
                            }
                        }
                    }
                }
                ValidationResult::Err { error, payload } => {
                    rejected.push((payload, format!("{:?}", error)));
                    if self.logging_config.log_on_transaction_rejection {
                        info!("TXN INVALID: {:?}", error);
                    }
                }
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

        let transactions_to_commit = commit_request
            .transaction_payloads
            .into_iter()
            .map(|t| {
                self.validation
                    .parse_and_validate_transaction_slice(&t)
                    .expect("Error on Byzantine quorum")
            })
            .collect::<Vec<_>>();

        let mut db_transaction = self.store.create_db_transaction();
        let ids_count: u64 = payload_hashes
            .len()
            .try_into()
            .expect("Can't map usize to u64");
        let mut current_state_version = commit_request.state_version - ids_count;

        for validated_txn in transactions_to_commit {
            let mut transaction_executor = TransactionExecutor::new(
                &mut db_transaction,
                &mut self.wasm_engine,
                &mut self.wasm_instrumenter,
            );

            let engine_receipt = transaction_executor.execute_and_commit(
                &validated_txn,
                &self.fee_reserve_config,
                &self.execution_config,
            );

            let ledger_receipt: LedgerTransactionReceipt =
                engine_receipt.try_into()
                    .unwrap_or_else(|error| {
                    panic!(
                        "Failed to commit a txn at state version {}: {}",
                        commit_request.state_version, error
                    )
                });

            let transaction = validated_txn.into_transaction();
            let payload_hash = transaction.payload_hash();
            let intent_hash = transaction.intent_hash();

            // TODO: Remove
            let stored_transaction = match transaction {
                Transaction::User(notarized_transaction) => StoredTransaction::User(notarized_transaction),
                Transaction::EpochUpdate(epoch) => StoredTransaction::System(scrypto_encode(&epoch)),
            };

            let identifiers = CommittedTransactionIdentifiers {
                state_version: current_state_version,
            };
            to_store.push((
                stored_transaction,
                ledger_receipt,
                identifiers,
            ));
            payload_hashes.push(payload_hash);
            intent_hashes.push(intent_hash);
            current_state_version += 1;
        }

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
        self.mempool.handle_committed_transactions(&intent_hashes);
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
            .add_resources(RENodeId::Component(component_address))
            .map_or(Option::None, |()| Some(resource_accounter.into_map()))
    }

    pub fn get_epoch(&self) -> u64 {
        self.store
            .get_substate(&SubstateId::System)
            .unwrap()
            .substate
            .system()
            .epoch
    }
}
