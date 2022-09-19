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

use crate::mempool::simple::SimpleMempool;
use crate::query::ResourceAccounter;
use crate::store::traits::*;
use crate::types::{
    CommitRequest, PrepareRequest, PrepareResult, PreviewRequest, StoredTransaction,
    TransactionPrepareResult,
};
use crate::{
    CommittedTransactionIdentifiers, HasIntentHash, HasPayloadHash, IntentHash,
    LedgerTransactionReceipt, MempoolAddError, PayloadHash, PendingTransaction,
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
use scrypto::engine::types::RENodeId;
use scrypto::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;
use tracing::info;
use transaction::errors::TransactionValidationError;
use transaction::model::{
    NotarizedTransaction, PreviewFlags, PreviewIntent, TransactionHeader, TransactionIntent,
    ValidatedTransaction,
};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{TestIntentHashManager, TransactionValidator, ValidationConfig};

struct OwnedValidationConfig {
    pub current_epoch: u64,
    pub max_cost_unit_limit: u32,
    pub min_tip_percentage: u32,
}

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

pub struct StateManager<S> {
    pub mempool: SimpleMempool,
    pub network: NetworkDefinition,
    pub store: S,
    wasm_engine: DefaultWasmEngine,
    wasm_instrumenter: WasmInstrumenter,
    validation_config: OwnedValidationConfig,
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
        StateManager {
            network,
            mempool,
            store,
            wasm_engine: DefaultWasmEngine::new(),
            wasm_instrumenter: WasmInstrumenter::new(),
            validation_config: OwnedValidationConfig {
                current_epoch: 1,
                max_cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
                min_tip_percentage: 0,
            },
            execution_config: ExecutionConfig {
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                is_system: false,
                trace: logging_config.engine_trace,
            },
            fee_reserve_config: FeeReserveConfig {
                cost_unit_price: DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
                system_loan: DEFAULT_SYSTEM_LOAN,
            },
            intent_hash_manager: TestIntentHashManager::new(),
            logging_config: logging_config.state_manager_config,
        }
    }

    pub fn parse_and_validate(
        &self,
        transaction_payload: &[u8],
    ) -> Result<(NotarizedTransaction, ValidatedTransaction), TransactionValidationError> {
        let notarized_transaction = Self::parse_from_slice(transaction_payload)?;

        let validated_transaction = self.validate_transaction(notarized_transaction.clone())?;

        Ok((notarized_transaction, validated_transaction))
    }

    pub fn validate_transaction_slice(
        &self,
        transaction_payload: &[u8],
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        let notarized_transaction = Self::parse_from_slice(transaction_payload)?;

        self.validate_transaction(notarized_transaction)
    }

    fn parse_from_slice(
        transaction_payload: &[u8],
    ) -> Result<NotarizedTransaction, TransactionValidationError> {
        if transaction_payload.len() > TransactionValidator::MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: NotarizedTransaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    pub fn validate_transaction(
        &self,
        transaction: NotarizedTransaction,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        let validation_config = ValidationConfig {
            network_id: self.network.id,
            current_epoch: self.validation_config.current_epoch,
            max_cost_unit_limit: self.validation_config.max_cost_unit_limit,
            min_tip_percentage: self.validation_config.min_tip_percentage,
        };
        TransactionValidator::validate(transaction, &self.intent_hash_manager, &validation_config)
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

impl<S> StateManager<S>
where
    S: ReadableSubstateStore,
    S: QueryableTransactionStore,
{
    fn test_transaction(
        &mut self,
        transaction: &PendingTransaction,
    ) -> Result<TransactionReceipt, TransactionValidationError> {
        let notarized_transaction = transaction.payload.clone();
        let validated_transaction = self.validate_transaction(notarized_transaction)?;

        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);
        let mut staged_store = staged_store_manager.get_output_store(staged_node);

        let mut transaction_executor = TransactionExecutor::new(
            &mut staged_store,
            &mut self.wasm_engine,
            &mut self.wasm_instrumenter,
        );
        let receipt = transaction_executor.execute_and_commit(
            &validated_transaction,
            &self.fee_reserve_config,
            &self.execution_config,
        );
        Ok(receipt)
    }

    pub fn add_to_mempool(
        &mut self,
        transaction: PendingTransaction,
    ) -> Result<(), MempoolAddError> {
        if self
            .store
            .get_committed_transaction_by_intent(&transaction.intent_hash)
            .is_some()
        {
            return Err(MempoolAddError::Rejected {
                reason: "Transaction rejected - duplicate intent hash".to_string(),
            });
        }

        let receipt =
            self.test_transaction(&transaction)
                .map_err(|e| MempoolAddError::Rejected {
                    reason: format!("{:?}", e),
                })?;

        match receipt.result {
            TransactionResult::Reject(result) => Err(MempoolAddError::Rejected {
                reason: format!("{:?}", result),
            }),
            TransactionResult::Commit(..) => self.mempool.add_transaction(transaction),
        }
    }

    pub fn get_relay_transactions(&mut self) -> Vec<PendingTransaction> {
        let mut mempool_txns = self.mempool.get_transactions();

        let mut txns_to_remove = Vec::new();
        for (hash, data) in &mempool_txns {
            let result = self.test_transaction(&data.transaction);
            match result {
                Err(..)
                | Ok(TransactionReceipt {
                    result: TransactionResult::Reject(..),
                    ..
                }) => {
                    txns_to_remove.push(hash.clone());
                }
                _ => {}
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
                .validate_transaction_slice(&prepared)
                .expect("Already prepared transactions should be decodeable");

            already_committed_or_prepared_intent_hashes.insert(validated_transaction.intent_hash());
            validated_prepared.push(validated_transaction);
        }

        let mut validated_proposed_transactions = Vec::new();
        for proposed_payload in prepare_request.proposed_payloads {
            let payload_hash: PayloadHash = PayloadHash::for_payload(&proposed_payload);
            let validation_result = self.validate_transaction_slice(&proposed_payload);

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

            validated_proposed_transactions.push((payload_hash, validation_result));
        }

        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);

        let mut staged_store = staged_store_manager.get_output_store(staged_node);
        for prepared in validated_prepared {
            let mut transaction_executor = TransactionExecutor::new(
                &mut staged_store,
                &mut self.wasm_engine,
                &mut self.wasm_instrumenter,
            );
            transaction_executor.execute_and_commit(
                &prepared,
                &self.fee_reserve_config,
                &self.execution_config,
            );
        }

        let mut transaction_results: Vec<(PayloadHash, TransactionPrepareResult)> = Vec::new();

        for (payload_hash, validation_result) in validated_proposed_transactions {
            match validation_result {
                Ok(validated_transaction) => {
                    let mut transaction_executor = TransactionExecutor::new(
                        &mut staged_store,
                        &mut self.wasm_engine,
                        &mut self.wasm_instrumenter,
                    );

                    let intent_hash = validated_transaction.intent_hash();
                    if already_committed_or_prepared_intent_hashes.contains(&intent_hash) {
                        transaction_results.push((
                            payload_hash,
                            TransactionPrepareResult::Reject {
                                reason: "Transaction rejected - duplicate intent hash".to_string(),
                            },
                        ));
                        if self.logging_config.log_on_transaction_rejection {
                            info!("TXN REJECTED: duplicate intent hash");
                        }
                    } else {
                        let receipt = transaction_executor.execute_and_commit(
                            &validated_transaction,
                            &self.fee_reserve_config,
                            &self.execution_config,
                        );
                        match receipt.result {
                            TransactionResult::Commit(..) => {
                                transaction_results
                                    .push((payload_hash, TransactionPrepareResult::CanCommit));
                                already_committed_or_prepared_intent_hashes.insert(intent_hash);
                            }
                            TransactionResult::Reject(reject_result) => {
                                transaction_results.push((
                                    payload_hash,
                                    TransactionPrepareResult::Reject {
                                        reason: format!("{:?}", reject_result),
                                    },
                                ));
                                if self.logging_config.log_on_transaction_rejection {
                                    info!("TXN REJECTED: {:?}", reject_result);
                                }
                            }
                        }
                    }
                }
                Err(validation_error) => {
                    transaction_results.push((
                        payload_hash,
                        TransactionPrepareResult::Reject {
                            reason: format!("{:?}", validation_error),
                        },
                    ));
                    if self.logging_config.log_on_transaction_rejection {
                        info!("TXN INVALID: {:?}", validation_error);
                    }
                }
            }
        }

        PrepareResult {
            transaction_results,
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
                self.parse_and_validate(&t)
                    .expect("Error on Byzantine quorum")
            })
            .collect::<Vec<_>>();

        let mut db_transaction = self.store.create_db_transaction();
        let ids_count: u64 = payload_hashes
            .len()
            .try_into()
            .expect("Can't map usize to u64");
        let mut current_state_version = commit_request.state_version - ids_count;

        for (notarized_txn, validated_txn) in transactions_to_commit {
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
                engine_receipt.try_into().unwrap_or_else(|_| {
                    panic!(
                        "Failed to commit a txn at state version {}",
                        commit_request.state_version
                    )
                });

            let payload_hash = notarized_txn.payload_hash();
            let intent_hash = notarized_txn.intent_hash();

            let identifiers = CommittedTransactionIdentifiers {
                state_version: current_state_version,
            };
            to_store.push((
                StoredTransaction::User(notarized_txn),
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
}
