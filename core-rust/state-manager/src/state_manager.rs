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

use crate::mempool::Mempool;
use crate::query::ResourceAccounter;
use crate::types::{
    CommitRequest, PrepareRequest, PrepareResult, PreviewRequest, TId, Transaction,
};
use crate::LedgerTransactionReceipt;
use radix_engine::constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_MAX_CALL_DEPTH, DEFAULT_SYSTEM_LOAN,
};
use radix_engine::ledger::{QueryableSubstateStore, ReadableSubstateStore, WriteableSubstateStore};
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    ExecutionConfig, PreviewError, PreviewExecutor, PreviewResult, TransactionExecutor,
    TransactionResult,
};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter};
use scrypto::engine::types::RENodeId;
use scrypto::prelude::*;
use std::collections::HashMap;
use transaction::errors::TransactionValidationError;
use transaction::model::{
    PreviewFlags, PreviewIntent, TransactionHeader, TransactionIntent, ValidatedTransaction,
};
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::{TestIntentHashManager, TransactionValidator, ValidationConfig};

struct OwnedValidationConfig {
    pub current_epoch: u64,
    pub max_cost_unit_limit: u32,
    pub min_tip_percentage: u32,
}

pub struct StateManager<M: Mempool, S> {
    pub mempool: M,
    pub network: NetworkDefinition,
    pub store: S,
    wasm_engine: DefaultWasmEngine,
    wasm_instrumenter: WasmInstrumenter,
    validation_config: OwnedValidationConfig,
    execution_config: ExecutionConfig,
    intent_hash_manager: TestIntentHashManager,
}

impl<M: Mempool, S> StateManager<M, S> {
    pub fn new(network: NetworkDefinition, mempool: M, store: S) -> StateManager<M, S> {
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
                cost_unit_price: DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
                max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                system_loan: DEFAULT_SYSTEM_LOAN,
                is_system: false,
                trace: false,
            },
            intent_hash_manager: TestIntentHashManager::new(),
        }
    }

    pub fn decode_transaction(
        &self,
        txn: &Transaction,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        let validation_config = ValidationConfig {
            network: &self.network,
            current_epoch: self.validation_config.current_epoch,
            max_cost_unit_limit: self.validation_config.max_cost_unit_limit,
            min_tip_percentage: self.validation_config.min_tip_percentage,
        };
        TransactionValidator::validate_from_slice(
            &txn.payload,
            &self.intent_hash_manager,
            &validation_config,
        )
    }
}

impl<M, S> StateManager<M, S>
where
    M: Mempool,
    S: ReadableSubstateStore,
{
    pub fn preview(
        &mut self,
        preview_request: PreviewRequest,
    ) -> Result<PreviewResult, PreviewError> {
        // not really used for preview
        let notary_private_key = EcdsaPrivateKey::from_u64(2).unwrap();

        let preview_intent = PreviewIntent {
            intent: TransactionIntent {
                header: TransactionHeader {
                    version: 1,
                    network_id: self.network.id,
                    start_epoch_inclusive: 0,
                    end_epoch_exclusive: 100,
                    nonce: preview_request.nonce,
                    notary_public_key: notary_private_key.public_key(),
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

impl<M, S> StateManager<M, S>
where
    M: Mempool,
    S: ReadableSubstateStore,
{
    pub fn prepare(&mut self, prepare_request: PrepareRequest) -> PrepareResult {
        let mut validated_prepared = Vec::new();
        for prepared in prepare_request.prepared {
            let validated_transaction = self
                .decode_transaction(&prepared)
                .expect("Prepared should be decodeable");
            validated_prepared.push(validated_transaction);
        }

        let mut non_rejected_txns = Vec::new();
        let mut rejected_txns = HashMap::new();
        let mut staged_store_manager = StagedSubstateStoreManager::new(&mut self.store);
        let staged_node = staged_store_manager.new_child_node(0);

        let mut staged_store = staged_store_manager.get_output_store(staged_node);
        for prepared in validated_prepared {
            let mut transaction_executor = TransactionExecutor::new(
                &mut staged_store,
                &mut self.wasm_engine,
                &mut self.wasm_instrumenter,
            );
            transaction_executor.execute_and_commit(&prepared, &self.execution_config);
        }

        for proposed in prepare_request.proposed {
            let maybe_validated_transaction = TransactionValidator::validate_from_slice(
                &proposed.payload,
                &self.intent_hash_manager,
                &ValidationConfig {
                    network: &self.network,
                    current_epoch: self.validation_config.current_epoch,
                    max_cost_unit_limit: self.validation_config.max_cost_unit_limit,
                    min_tip_percentage: self.validation_config.min_tip_percentage,
                },
            );

            match maybe_validated_transaction {
                Ok(validated_transaction) => {
                    let mut transaction_executor = TransactionExecutor::new(
                        &mut staged_store,
                        &mut self.wasm_engine,
                        &mut self.wasm_instrumenter,
                    );
                    let receipt = transaction_executor
                        .execute_and_commit(&validated_transaction, &self.execution_config);
                    match receipt.result {
                        TransactionResult::Commit(..) => non_rejected_txns.push(proposed),
                        TransactionResult::Reject(reject_result) => {
                            rejected_txns.insert(proposed, format!("{:?}", reject_result));
                        }
                    }
                }
                Err(validation_error) => {
                    rejected_txns.insert(proposed, format!("{:?}", validation_error));
                }
            }
        }

        PrepareResult {
            non_rejected_txns,
            rejected_txns,
        }
    }
}

pub trait CommitStoreTransaction<'db>:
    WriteableTransactionStore
    + WriteableProofStore
    + WriteableVertexStore
    + WriteableSubstateStore
    + ReadableSubstateStore
{
    fn commit(self);
}

pub trait CommitStore<'db> {
    type DBTransaction: CommitStoreTransaction<'db>;

    fn create_db_transaction(&'db mut self) -> Self::DBTransaction;
}

pub trait WriteableTransactionStore {
    fn insert_transactions(&mut self, transactions: Vec<(&Transaction, LedgerTransactionReceipt)>);
}

pub trait WriteableProofStore {
    fn insert_tids_and_proof(&mut self, state_version: u64, ids: Vec<TId>, proof_bytes: Vec<u8>);
}

pub trait WriteableVertexStore {
    fn save_vertex_store(&mut self, vertex_store_bytes: Vec<u8>);
}

impl<'db, M, S> StateManager<M, S>
where
    M: Mempool,
    S: CommitStore<'db>,
{
    pub fn commit(&'db mut self, commit_request: CommitRequest) {
        let mut to_store = Vec::new();
        let mut ids = Vec::new();

        let mut db_transaction = self.store.create_db_transaction();

        for transaction in &commit_request.transactions {
            let validation_config = ValidationConfig {
                network: &self.network,
                current_epoch: self.validation_config.current_epoch,
                max_cost_unit_limit: self.validation_config.max_cost_unit_limit,
                min_tip_percentage: self.validation_config.min_tip_percentage,
            };
            let validated_txn = TransactionValidator::validate_from_slice(
                &transaction.payload,
                &self.intent_hash_manager,
                &validation_config,
            )
            .expect("Error on Byzantine quorum");

            let mut transaction_executor = TransactionExecutor::new(
                &mut db_transaction,
                &mut self.wasm_engine,
                &mut self.wasm_instrumenter,
            );

            let engine_receipt =
                transaction_executor.execute_and_commit(&validated_txn, &self.execution_config);

            let ledger_receipt: LedgerTransactionReceipt =
                engine_receipt.try_into().unwrap_or_else(|_| {
                    panic!(
                        "Failed to commit a txn at state version {}",
                        commit_request.state_version
                    )
                });

            to_store.push((transaction, ledger_receipt));
            ids.push(transaction.id.clone());
        }

        db_transaction.insert_transactions(to_store);
        db_transaction.insert_tids_and_proof(
            commit_request.state_version,
            ids,
            commit_request.proof,
        );
        if let Some(vertex_store) = commit_request.vertex_store {
            db_transaction.save_vertex_store(vertex_store);
        }

        db_transaction.commit();

        self.mempool
            .handle_committed_transactions(&commit_request.transactions);
    }
}

impl<M: Mempool, S: ReadableSubstateStore + QueryableSubstateStore> StateManager<M, S> {
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
