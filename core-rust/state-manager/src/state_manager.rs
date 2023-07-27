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

use crate::limits::{VertexLimitsAdvanceSuccess, VertexLimitsTracker};
use crate::mempool_manager::MempoolManager;
use crate::query::*;
use crate::staging::epoch_handling::EpochAwareAccuTreeFactory;
use crate::staging::{ExecutionCache, ReadableStore};
use crate::store::traits::*;
use crate::transaction::*;
use crate::types::{CommitRequest, PrepareRequest, PrepareResult};
use crate::*;
use ::transaction::errors::TransactionValidationError;
use ::transaction::model::{IntentHash, NotarizedTransactionHash};
use ::transaction::prelude::*;
use node_common::config::limits::VertexLimitsConfig;

use radix_engine::system::bootstrap::*;
use radix_engine::transaction::{RejectResult, TransactionReceipt};
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, EpochChangeCondition,
};
use transaction_scenarios::scenario::DescribedAddress as ScenarioDescribedAddress;
use transaction_scenarios::scenario::*;
use transaction_scenarios::scenarios::*;

use node_common::locks::{Mutex, RwLock};
use prometheus::Registry;
use tracing::{info, warn};

use crate::store::traits::scenario::{
    DescribedAddress, ExecutedGenesisScenario, ExecutedGenesisScenarioStore,
    ExecutedScenarioTransaction, ScenarioSequenceNumber,
};
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

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
    network: NetworkDefinition,
    store: Arc<RwLock<S>>,
    mempool_manager: Arc<MempoolManager>,
    execution_configurator: Arc<ExecutionConfigurator>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    execution_cache: Mutex<ExecutionCache>,
    ledger_transaction_validator: LedgerTransactionValidator,
    ledger_metrics: LedgerMetrics,
    committed_transactions_metrics: CommittedTransactionsMetrics,
    vertex_prepare_metrics: VertexPrepareMetrics,
    vertex_limits_config: VertexLimitsConfig,
    logging_config: StateManagerLoggingConfig,
}

impl<S: TransactionIdentifierLoader> StateManager<S> {
    // TODO: refactor and maybe make clippy happy too
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network: &NetworkDefinition,
        vertex_limits_config: VertexLimitsConfig,
        store: Arc<RwLock<S>>,
        mempool_manager: Arc<MempoolManager>,
        execution_configurator: Arc<ExecutionConfigurator>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
        logging_config: LoggingConfig,
        metric_registry: &Registry,
    ) -> StateManager<S> {
        let transaction_root = store.read().get_top_ledger_hashes().1.transaction_root;

        let regular_execution_config = execution_configurator
            .execution_configs
            .get(&ConfigType::Regular)
            .unwrap();
        let committed_transactions_metrics =
            CommittedTransactionsMetrics::new(metric_registry, regular_execution_config);

        StateManager {
            network: network.clone(),
            store,
            mempool_manager,
            execution_configurator,
            pending_transaction_result_cache,
            execution_cache: Mutex::new(ExecutionCache::new(transaction_root)),
            ledger_transaction_validator: LedgerTransactionValidator::new(network),
            logging_config: logging_config.state_manager_config,
            vertex_prepare_metrics: VertexPrepareMetrics::new(metric_registry),
            vertex_limits_config,
            ledger_metrics: LedgerMetrics::new(metric_registry),
            committed_transactions_metrics,
        }
    }
}

pub enum StateManagerRejectReason {
    TransactionValidationError(TransactionValidationError),
}

#[derive(Debug)]
pub struct GenesisCommitRequestFactory {
    epoch: Epoch,
    timestamp: i64,
    state_version: StateVersion,
    genesis_opaque_hash: Hash,
}

impl GenesisCommitRequestFactory {
    pub fn create_next(&mut self, result: GenesisPrepareResult) -> GenesisCommitRequest {
        self.state_version = self.state_version.next();
        GenesisCommitRequest {
            raw: result.raw,
            validated: result.validated,
            proof: self.create_proof(result.ledger_hashes, result.next_epoch),
            require_success: true,
        }
    }

    pub fn create_for_scenario(
        &mut self,
        result: ScenarioPrepareResult,
    ) -> Option<GenesisCommitRequest> {
        let Some(ledger_hashes) = result.committable_ledger_hashes else {
            return None;
        };
        self.state_version = self.state_version.next();
        Some(GenesisCommitRequest {
            raw: result.raw,
            validated: result.validated,
            proof: self.create_proof(ledger_hashes, None),
            require_success: false,
        })
    }

    fn create_proof(&self, hashes: LedgerHashes, next_epoch: Option<NextEpoch>) -> LedgerProof {
        LedgerProof {
            opaque: self.genesis_opaque_hash,
            ledger_header: LedgerHeader {
                epoch: self.epoch,
                round: Round::zero(),
                state_version: self.state_version,
                hashes,
                consensus_parent_round_timestamp_ms: self.timestamp,
                proposer_timestamp_ms: self.timestamp,
                next_epoch,
            },
            timestamped_signatures: vec![],
        }
    }
}

pub struct GenesisPrepareResult {
    raw: RawLedgerTransaction,
    validated: ValidatedLedgerTransaction,
    ledger_hashes: LedgerHashes,
    next_epoch: Option<NextEpoch>,
}

pub struct ScenarioPrepareResult {
    raw: RawLedgerTransaction,
    validated: ValidatedLedgerTransaction,
    committable_ledger_hashes: Option<LedgerHashes>,
}

pub struct GenesisCommitRequest {
    raw: RawLedgerTransaction,
    validated: ValidatedLedgerTransaction,
    proof: LedgerProof,
    require_success: bool,
}

impl<S> StateManager<S>
where
    S: ReadableStore,
    S: for<'a> TransactionIndex<&'a IntentHash>,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    pub fn prepare_genesis(&self, genesis_transaction: GenesisTransaction) -> GenesisPrepareResult {
        let raw = LedgerTransaction::Genesis(Box::new(genesis_transaction))
            .to_raw()
            .expect("Could not encode genesis transaction");
        let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw)
            .expect("Could not prepare genesis transaction");
        let validated = self.ledger_transaction_validator.validate_genesis(prepared);

        let read_store = self.store.read();
        let mut series_executor = self.start_series_execution(read_store.deref());

        let commit = series_executor
            .execute_and_update_state(&validated, "genesis")
            .expect("genesis not committable")
            .expect_success("genesis");

        GenesisPrepareResult {
            raw,
            validated,
            ledger_hashes: commit.hash_structures_diff.ledger_hashes,
            next_epoch: commit.next_epoch(),
        }
    }

    pub fn prepare_scenario_transaction(
        &self,
        scenario_name: &str,
        next: &NextTransaction,
    ) -> (ScenarioPrepareResult, TransactionReceipt) {
        let qualified_name = format!(
            "{} scenario - {} transaction",
            scenario_name, &next.logical_name
        );

        let (raw, prepared_ledger_transaction) = self
            .try_prepare_ledger_transaction_from_user_transaction(&next.raw_transaction)
            .unwrap_or_else(|_| panic!("Expected that {} was preparable", qualified_name));

        let validated = self
            .ledger_transaction_validator
            .validate_user_or_round_update(prepared_ledger_transaction)
            .unwrap_or_else(|_| panic!("Expected that {} was valid", qualified_name));

        let read_store = self.store.read();

        // Note - we first create a basic receipt - because we need it for later
        let basic_receipt = self
            .execution_configurator
            .wrap_ledger_transaction(&validated, "scenario transaction")
            .execute_on(read_store.deref());
        let mut series_executor = self.start_series_execution(read_store.deref());

        let commit = series_executor.execute_and_update_state(&validated, "scenario transaction");

        let prepare_result = ScenarioPrepareResult {
            raw,
            validated,
            committable_ledger_hashes: commit
                .ok()
                .map(|commit_result| commit_result.hash_structures_diff.ledger_hashes),
        };

        (prepare_result, basic_receipt)
    }

    pub fn prepare(&self, prepare_request: PrepareRequest) -> PrepareResult {
        //========================================================================================
        // NOTE:
        // In this method, "already prepared" transactions that live between the commit point and the current
        // proposal will be referred to as "ancestor" - to distinguish them from "preparation" of the transactions
        // themselves, which is part of the validation process
        //========================================================================================

        let read_store = self.store.read();
        let mut series_executor = self.start_series_execution(read_store.deref());

        if &prepare_request.committed_ledger_hashes != series_executor.latest_ledger_hashes() {
            panic!(
                "state {:?} from request does not match the current ledger state {:?}",
                prepare_request.committed_ledger_hashes,
                series_executor.latest_ledger_hashes()
            );
        }

        //========================================================================================
        // PART 1:
        // We execute all the ancestor transactions (on a happy path: only making sure they are in
        // our execution cache),
        //========================================================================================

        let pending_transaction_base_state = AtState::PendingPreparingVertices {
            base_committed_state_version: series_executor.latest_state_version(),
        };

        for raw_ancestor in prepare_request.ancestor_transactions {
            // TODO(optimization-only): We could avoid the hashing, decoding, signature verification
            // and executable creation) by accessing the execution cache in a more clever way.
            let validated = self
                .ledger_transaction_validator
                .validate_user_or_round_update_from_raw(&raw_ancestor)
                .expect("Ancestor transactions should be valid");

            series_executor
                .execute_and_update_state(&validated, "ancestor")
                .expect("ancestor transaction rejected");
        }

        if &prepare_request.ancestor_ledger_hashes != series_executor.latest_ledger_hashes() {
            panic!(
                "State {:?} after ancestor transactions does not match the state {:?} from request",
                series_executor.latest_ledger_hashes(),
                prepare_request.ancestor_ledger_hashes,
            );
        }

        //========================================================================================
        // PART 2:
        // We start off the preparation by adding and executing the round change transaction
        //========================================================================================

        let mut committable_transactions = Vec::new();
        let mut vertex_limits_tracker = VertexLimitsTracker::new(&self.vertex_limits_config);

        // TODO: Unify this with the proposed payloads execution
        let round_update = RoundUpdateTransactionV1::new(
            series_executor.epoch_header(),
            &prepare_request.round_history,
        );
        let ledger_round_update = LedgerTransaction::RoundUpdateV1(Box::new(round_update));
        let validated_round_update = self
            .ledger_transaction_validator
            .validate_user_or_round_update_from_model(&ledger_round_update)
            .expect("expected to be able to prepare the round update transaction");

        let raw_ledger_round_update = ledger_round_update
            .to_raw()
            .expect("Expected round update to be encodable");

        let transaction_size = raw_ledger_round_update.as_slice().len();
        vertex_limits_tracker
            .check_pre_execution(transaction_size)
            .expect("round update transaction should fit inside of empty vertex");

        let round_update_result = series_executor
            .execute_and_update_state(&validated_round_update, "round update")
            .expect("round update rejected");

        vertex_limits_tracker
            .try_next_transaction(
                transaction_size,
                &round_update_result
                    .local_receipt
                    .local_execution
                    .execution_metrics,
            )
            .expect("round update transaction should not trigger vertex limits");

        round_update_result.expect_success("round update");

        committable_transactions.push(CommittableTransaction {
            index: None,
            raw: raw_ledger_round_update,
            intent_hash: None,
            notarized_transaction_hash: None,
            ledger_transaction_hash: validated_round_update.ledger_transaction_hash(),
        });

        //========================================================================================
        // PART 3:
        // We continue by attempting to execute the remaining transactions in the proposal
        //========================================================================================

        let mut rejected_transactions = Vec::new();
        let pending_transaction_timestamp = SystemTime::now();
        let mut pending_transaction_results = Vec::new();
        let total_proposal_size: usize = prepare_request
            .proposed_transactions
            .iter()
            .map(|tx| tx.0.len())
            .sum();
        let mut committed_proposal_size = 0;
        let mut stop_reason = VertexPrepareStopReason::ProposalComplete;

        for (index, raw_user_transaction) in prepare_request
            .proposed_transactions
            .into_iter()
            .enumerate()
        {
            // Don't process any additional transactions if next epoch has occurred
            if series_executor.next_epoch().is_some() {
                stop_reason = VertexPrepareStopReason::EpochChange;
                break;
            }

            let transaction_size = raw_user_transaction.as_slice().len();

            // Skip validating and executing this transaction if it doesn't fit it in the vertex.
            if vertex_limits_tracker
                .check_pre_execution(transaction_size)
                .is_err()
            {
                continue;
            }

            let try_prepare_result =
                self.try_prepare_ledger_transaction_from_user_transaction(&raw_user_transaction);

            let (raw_ledger_transaction, prepared_transaction) = match try_prepare_result {
                Ok(results) => results,
                Err(error) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: None,
                        notarized_transaction_hash: None,
                        ledger_transaction_hash: None,
                        error: format!("{error:?}"),
                    });
                    continue;
                }
            };

            let prepared_user_transaction = prepared_transaction
                .as_user()
                .expect("Proposed was created from user");

            let intent_hash = prepared_user_transaction.intent_hash();
            let notarized_transaction_hash = prepared_user_transaction.notarized_transaction_hash();
            let ledger_transaction_hash = prepared_transaction.ledger_transaction_hash();
            let invalid_at_epoch = prepared_user_transaction
                .signed_intent
                .intent
                .header
                .inner
                .end_epoch_exclusive;

            // TODO(optimization-only): We could avoid signature verification by re-using the
            // validated transaction from the mempool.
            let validate_result = self
                .ledger_transaction_validator
                .validate_user_or_round_update(prepared_transaction);

            let validated = match validate_result {
                Ok(validated) => validated,
                Err(error) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: Some(intent_hash),
                        notarized_transaction_hash: Some(notarized_transaction_hash),
                        ledger_transaction_hash: Some(ledger_transaction_hash),
                        error: format!("{:?}", &error),
                    });
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        notarized_transaction_hash,
                        invalid_at_epoch,
                        rejection_reason: Some(RejectionReason::ValidationError(
                            error.into_user_validation_error(),
                        )),
                    });
                    continue;
                }
            };

            // Note that we're using a "_no_state_update" variant here, because
            // we may still reject some *committable* transactions if they exceed
            // the limit, which would otherwise spoil the internal StateTracker.
            // So it's important to manually update the state if the transaction
            // is to be included (that's the `series_executor.update_state(...)` call below).
            let execute_result =
                series_executor.execute_no_state_update(&validated, "newly proposed");
            match execute_result {
                Ok(processed_commit_result) => {
                    match vertex_limits_tracker.try_next_transaction(
                        transaction_size,
                        &processed_commit_result
                            .local_receipt
                            .local_execution
                            .execution_metrics,
                    ) {
                        Ok(success) => {
                            // We're including the transaction, so updating the executor state
                            series_executor.update_state(&processed_commit_result);
                            committed_proposal_size += transaction_size;
                            committable_transactions.push(CommittableTransaction {
                                index: Some(index as u32),
                                raw: raw_ledger_transaction,
                                intent_hash: Some(intent_hash),
                                notarized_transaction_hash: Some(notarized_transaction_hash),
                                ledger_transaction_hash,
                            });
                            pending_transaction_results.push(PendingTransactionResult {
                                intent_hash,
                                notarized_transaction_hash,
                                invalid_at_epoch,
                                rejection_reason: None,
                            });
                            match success {
                                VertexLimitsAdvanceSuccess::VertexFilled(limit_exceeded) => {
                                    stop_reason =
                                        VertexPrepareStopReason::LimitExceeded(limit_exceeded);
                                    break;
                                }
                                VertexLimitsAdvanceSuccess::VertexNotFilled => {}
                            }
                        }
                        Err(error) => {
                            rejected_transactions.push(RejectedTransaction {
                                index: index as u32,
                                intent_hash: Some(intent_hash),
                                notarized_transaction_hash: Some(notarized_transaction_hash),
                                ledger_transaction_hash: Some(ledger_transaction_hash),
                                error: format!("{:?}", &error),
                            });
                            // In order to mitigate the worst-case scenario where the proposal contains lots of small
                            // transactions that take maximum amount of time to execute, we stop right after first
                            // exceeded vertex limit.
                            stop_reason = VertexPrepareStopReason::LimitExceeded(error);
                            break;
                            // Note: we are not adding this transaction to [`pending_transaction_results`] because
                            // we don't want to remove it from mempool yet.
                        }
                    }
                }
                Err(RejectResult { error }) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: Some(intent_hash),
                        notarized_transaction_hash: Some(notarized_transaction_hash),
                        ledger_transaction_hash: Some(ledger_transaction_hash),
                        error: format!("{:?}", &error),
                    });
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        notarized_transaction_hash,
                        invalid_at_epoch,
                        rejection_reason: Some(RejectionReason::FromExecution(Box::new(error))),
                    });
                }
            }
        }

        if self.logging_config.log_on_transaction_rejection {
            for rejection in rejected_transactions.iter() {
                info!("TXN INVALID: {}", &rejection.error);
            }
        }

        let pending_rejected_transactions = pending_transaction_results
            .iter()
            .filter(|pending_result| pending_result.rejection_reason.is_some())
            .map(|pending_result| {
                (
                    &pending_result.intent_hash,
                    &pending_result.notarized_transaction_hash,
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
                pending_transaction_result.notarized_transaction_hash,
                Some(pending_transaction_result.invalid_at_epoch),
                attempt,
            );
        }
        drop(write_pending_transaction_result_cache);

        self.vertex_prepare_metrics.update(
            total_proposal_size,
            committed_proposal_size,
            stop_reason,
        );

        PrepareResult {
            committed: committable_transactions,
            rejected: rejected_transactions,
            next_epoch: series_executor.next_epoch().cloned(),
            ledger_hashes: *series_executor.latest_ledger_hashes(),
        }
    }

    fn try_prepare_ledger_transaction_from_user_transaction(
        &self,
        raw_user_transaction: &RawNotarizedTransaction,
    ) -> Result<(RawLedgerTransaction, PreparedLedgerTransaction), TransactionValidationError> {
        LedgerTransaction::from_raw_user(raw_user_transaction)
            .map_err(|err| TransactionValidationError::PrepareError(PrepareError::DecodeError(err)))
            .and_then(|ledger_transaction| {
                ledger_transaction.to_raw().map_err(|err| {
                    TransactionValidationError::PrepareError(PrepareError::EncodeError(err))
                })
            })
            .and_then(|raw_ledger_transaction| {
                self.ledger_transaction_validator
                    .prepare_from_raw(&raw_ledger_transaction)
                    .map(|prepared_transaction| (raw_ledger_transaction, prepared_transaction))
            })
    }

    fn start_series_execution<'s>(&'s self, store: &'s S) -> TransactionSeriesExecutor<'s, S> {
        TransactionSeriesExecutor::new(
            store,
            &self.execution_cache,
            self.execution_configurator.deref(),
        )
    }
}

impl<S> StateManager<S>
where
    S: CommitStore + ExecutedGenesisScenarioStore,
    S: ReadableStore,
    S: for<'a> TransactionIndex<&'a IntentHash>,
    S: QueryableProofStore + TransactionIdentifierLoader,
{
    /// Performs an [`execute_genesis()`] with a hardcoded genesis data meant for test purposes.
    pub fn execute_genesis_for_unit_tests(&self) -> LedgerProof {
        // Roughly copied from bootstrap_test_default in scrypto
        let genesis_validator: GenesisValidator = Secp256k1PublicKey([0; 33]).into();
        let genesis_chunks = vec![
            GenesisDataChunk::Validators(vec![genesis_validator.clone()]),
            GenesisDataChunk::Stakes {
                accounts: vec![ComponentAddress::virtual_account_from_public_key(
                    &genesis_validator.key,
                )],
                allocations: vec![(
                    genesis_validator.key,
                    vec![GenesisStakeAllocation {
                        account_index: 0,
                        xrd_amount: dec!("100"),
                    }],
                )],
            },
        ];
        let initial_epoch = Epoch::of(1);
        let initial_config = ConsensusManagerConfig {
            max_validators: 10,
            epoch_change_condition: EpochChangeCondition {
                min_round_count: 1,
                max_round_count: 1,
                target_duration_millis: 0,
            },
            num_unstake_epochs: 1,
            total_emission_xrd_per_epoch: Decimal::one(),
            min_validator_reliability: Decimal::one(),
            num_owner_stake_units_unlock_epochs: 2,
            num_fee_increase_delay_epochs: 1,
            validator_creation_usd_cost: Decimal::one(),
        };
        let initial_timestamp_ms = 1;
        self.execute_genesis(
            genesis_chunks,
            initial_epoch,
            initial_config,
            initial_timestamp_ms,
            Hash([0; Hash::LENGTH]),
            *DEFAULT_TESTING_FAUCET_SUPPLY,
            vec![],
        )
    }

    /// Creates and commits a series of genesis transactions (i.e. a boostrap, then potentially many
    /// data ingestion chunks, and then a wrap-up).
    #[allow(clippy::too_many_arguments)]
    pub fn execute_genesis(
        &self,
        genesis_data_chunks: Vec<GenesisDataChunk>,
        initial_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_timestamp_ms: i64,
        genesis_opaque_hash: Hash,
        faucet_supply: Decimal,
        scenarios_to_run: Vec<String>,
    ) -> LedgerProof {
        let start_instant = Instant::now();

        let read_db = self.store.read();
        if read_db.get_post_genesis_epoch_proof().is_some() {
            panic!("Can't execute genesis: database already initialized")
        }
        let maybe_top_txn_identifiers = read_db.get_top_transaction_identifiers();
        drop(read_db);
        if let Some(top_txn_identifiers) = maybe_top_txn_identifiers {
            // No epoch proof, but there are some committed txns
            panic!(
                "The database is in inconsistent state: \
                there are committed transactions (up to state version {}), but there's no epoch proof. \
                This is likely caused by the the genesis data ingestion being interrupted. \
                Consider wiping your database dir and trying again.", top_txn_identifiers.0);
        }

        let mut genesis_commit_request_factory = GenesisCommitRequestFactory {
            epoch: initial_epoch,
            timestamp: initial_timestamp_ms,
            state_version: StateVersion::pre_genesis(),
            genesis_opaque_hash,
        };

        info!("Committing system flash");
        let prepare_result = self.prepare_genesis(GenesisTransaction::Flash);
        let commit_request = genesis_commit_request_factory.create_next(prepare_result);
        self.commit_genesis(commit_request);

        info!("Committing system bootstrap");
        let transaction = create_system_bootstrap_transaction(
            initial_epoch,
            initial_config,
            initial_timestamp_ms,
            // Leader gets set to None, to be fixed at the first proper round change.
            None,
            faucet_supply,
        );
        let prepare_result =
            self.prepare_genesis(GenesisTransaction::Transaction(Box::new(transaction)));
        let commit_request = genesis_commit_request_factory.create_next(prepare_result);
        self.commit_genesis(commit_request);

        let genesis_data_chunks_len = genesis_data_chunks.len();
        for (index, chunk) in genesis_data_chunks.into_iter().enumerate() {
            let chunk_type = match chunk {
                GenesisDataChunk::Validators(_) => "validators",
                GenesisDataChunk::Stakes { .. } => "stakes",
                GenesisDataChunk::Resources(_) => "resources",
                GenesisDataChunk::ResourceBalances { .. } => "resource_balances",
                GenesisDataChunk::XrdBalances(_) => "xrd_balances",
            };
            info!(
                "Committing data ingestion chunk ({}) {} of {}",
                chunk_type,
                index + 1,
                genesis_data_chunks_len
            );
            let transaction =
                create_genesis_data_ingestion_transaction(&GENESIS_HELPER, chunk, index);
            let prepare_result =
                self.prepare_genesis(GenesisTransaction::Transaction(Box::new(transaction)));
            let commit_request = genesis_commit_request_factory.create_next(prepare_result);
            self.commit_genesis(commit_request);
        }

        // These scenarios are committed before we start consensus / rounds after the genesis wrap-up.
        // This is a little weird, but should be fine.
        if !scenarios_to_run.is_empty() {
            info!("Running {} scenarios", scenarios_to_run.len());
            let mut next_nonce: u32 = 0;
            for (sequence_number, scenario_name) in scenarios_to_run.iter().enumerate() {
                next_nonce = self.execute_genesis_scenario(
                    &mut genesis_commit_request_factory,
                    sequence_number.try_into().unwrap(),
                    scenario_name.as_str(),
                    initial_epoch,
                    next_nonce,
                );
            }
            info!("Scenarios finished");
        }

        info!("Committing genesis wrap-up");
        let transaction: SystemTransactionV1 = create_genesis_wrap_up_transaction();
        let prepare_result =
            self.prepare_genesis(GenesisTransaction::Transaction(Box::new(transaction)));
        let commit_request = genesis_commit_request_factory.create_next(prepare_result);
        let final_ledger_proof = commit_request.proof.clone();
        self.commit_genesis(commit_request);

        info!(
            "Genesis transactions successfully executed in {:?}",
            start_instant.elapsed()
        );
        final_ledger_proof
    }

    fn execute_genesis_scenario(
        &self,
        genesis_commit_request_factory: &mut GenesisCommitRequestFactory,
        sequence_number: ScenarioSequenceNumber,
        scenario_name: &str,
        epoch: Epoch,
        nonce: u32,
    ) -> u32 {
        let mut scenario = self
            .find_scenario(epoch, nonce, scenario_name)
            .unwrap_or_else(|| {
                panic!(
                    "Could not find scenario with logical name: {}",
                    scenario_name
                )
            });
        let mut previous = None;
        let mut committed_transactions = Vec::new();
        info!("Running scenario: {}", scenario_name);
        loop {
            let next = scenario
                .next(previous.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let (prepare_result, basic_receipt) =
                        self.prepare_scenario_transaction(scenario_name, &next);
                    let intent_hash = prepare_result.validated.intent_hash_if_user().unwrap();
                    if let Some(commit_request) =
                        genesis_commit_request_factory.create_for_scenario(prepare_result)
                    {
                        self.commit_genesis(commit_request);
                        committed_transactions.push(ExecutedScenarioTransaction {
                            logical_name: next.logical_name.clone(),
                            state_version: genesis_commit_request_factory.state_version,
                            intent_hash,
                        });
                        info!(
                            "Committed {} at state_version {}",
                            &next.logical_name, genesis_commit_request_factory.state_version
                        );
                    }
                    previous = Some(basic_receipt);
                }
                NextAction::Completed(end_state) => {
                    let encoder = AddressBech32Encoder::new(&self.network);
                    let executed_scenario = ExecutedGenesisScenario {
                        logical_name: scenario.metadata().logical_name.to_string(),
                        committed_transactions,
                        addresses: end_state
                            .output
                            .interesting_addresses
                            .0
                            .into_iter()
                            .map(|(descriptor, address)| DescribedAddress {
                                logical_name: descriptor,
                                rendered_address: match address {
                                    ScenarioDescribedAddress::Global(address) => {
                                        address.to_string(&encoder)
                                    }
                                    ScenarioDescribedAddress::Internal(address) => {
                                        address.to_string(&encoder)
                                    }
                                    ScenarioDescribedAddress::NonFungible(nf_global_id) => {
                                        nf_global_id.to_string(&encoder)
                                    }
                                },
                            })
                            .collect(),
                    };
                    info!(
                        "Completed committing {} transactions for scenario {}, with resultant addresses:\n{}",
                        executed_scenario.committed_transactions.len(),
                        executed_scenario.logical_name,
                        executed_scenario.addresses
                            .iter()
                            .map(|address| format!(
                                "  - {}: {}", address.logical_name, address.rendered_address
                            ))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                    let mut write_store = self.store.write();
                    write_store.put_scenario(sequence_number, executed_scenario);
                    return end_state.next_unused_nonce;
                }
            }
        }
    }

    fn find_scenario(
        &self,
        epoch: Epoch,
        next_nonce: u32,
        scenario_name: &str,
    ) -> Option<Box<dyn ScenarioInstance>> {
        for scenario_builder in get_builder_for_every_scenario() {
            let scenario =
                scenario_builder(ScenarioCore::new(self.network.clone(), epoch, next_nonce));
            if scenario.metadata().logical_name == scenario_name {
                return Some(scenario);
            }
        }
        None
    }

    /// Validates and commits the transactions from the given request (or returns an error in case
    /// of invalid request).
    /// Persistently stores the transaction payloads and execution results, together with the
    /// associated proof and vertex store state.
    pub fn commit(&self, commit_request: CommitRequest) -> Result<(), InvalidCommitRequestError> {
        let commit_transactions_len = commit_request.transactions.len();
        if commit_transactions_len == 0 {
            panic!("broken invariant: no transactions in request {commit_request:?}");
        }

        let commit_ledger_header = &commit_request.proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;
        let commit_request_start_state_version =
            commit_state_version.relative(-(commit_transactions_len as i128));

        // Step 1.: Parse the transactions (and collect specific metrics from them, as a drive-by)
        let mut prepared_transactions = Vec::new();
        let mut leader_round_counters_builder = LeaderRoundCountersBuilder::default();
        for (index, raw_transaction) in commit_request.transactions.iter().enumerate() {
            let result = self
                .ledger_transaction_validator
                .prepare_from_raw(raw_transaction);
            let prepared_transaction = match result {
                Ok(prepared_transaction) => prepared_transaction,
                Err(error) => {
                    warn!(
                        "invalid commit request: cannot parse transaction at index {}: {:?}",
                        index, error
                    );
                    return Err(InvalidCommitRequestError::TransactionParsingFailed);
                }
            };

            if let PreparedLedgerTransactionInner::RoundUpdateV1(_) = &prepared_transaction.inner {
                let round_update = LedgerTransaction::from_raw(raw_transaction)
                    .expect("the same transaction was parsed fine above");
                if let LedgerTransaction::RoundUpdateV1(round_update) = round_update {
                    leader_round_counters_builder.update(&round_update.leader_proposal_history);
                }
            }

            prepared_transactions.push(prepared_transaction);
        }

        // Step 2.: Start the write DB transaction, check invariants, set-up DB update structures
        let mut write_store = self.store.write();
        let mut series_executor = self.start_series_execution(write_store.deref());

        if commit_request_start_state_version != series_executor.latest_state_version() {
            panic!(
                "broken invariant: commit request assumed state version ({} - {} = {}) while ledger is at {}",
                commit_state_version,
                commit_transactions_len,
                commit_request_start_state_version,
                series_executor.latest_state_version(),
            );
        }

        let resultant_transaction_root = self
            .find_transaction_root_in_execution_cache(
                &series_executor.latest_ledger_hashes().transaction_root,
                &prepared_transactions,
            )
            .unwrap_or_else(|| {
                Self::calculate_transaction_root(
                    write_store.deref(),
                    series_executor.epoch_identifiers(),
                    series_executor.latest_state_version(),
                    &prepared_transactions,
                )
            });

        if resultant_transaction_root != commit_ledger_header.hashes.transaction_root {
            warn!(
                "invalid commit request: resultant transaction root at version {} differs from the proof ({} != {})",
                commit_state_version,
                resultant_transaction_root,
                commit_ledger_header.hashes.transaction_root
            );
            return Err(InvalidCommitRequestError::TransactionRootMismatch);
        }

        let mut committed_transaction_bundles = Vec::new();
        let mut transactions_metrics_data = Vec::new();
        let mut substate_store_update = SubstateStoreUpdate::new();
        let mut state_tree_update = HashTreeUpdate::new();
        let epoch_accu_trees = EpochAwareAccuTreeFactory::new(
            series_executor.epoch_identifiers().state_version,
            series_executor.latest_state_version(),
        );
        let mut transaction_tree_slice_merger = epoch_accu_trees.create_merger();
        let mut receipt_tree_slice_merger = epoch_accu_trees.create_merger();
        let mut intent_hashes = Vec::new();

        // Step 3.: Actually execute the transactions, collect their results into DB structures
        for (raw, prepared) in commit_request
            .transactions
            .into_iter()
            .zip(prepared_transactions)
        {
            let validated = self
                .ledger_transaction_validator
                .validate_user_or_round_update(prepared)
                .unwrap_or_else(|error| {
                    panic!("cannot validate transaction to be committed: {error:?}");
                });

            let commit = series_executor
                .execute_and_update_state(&validated, "prepared")
                .expect("cannot execute transaction to be committed");

            if let Some(intent_hash) = validated.intent_hash_if_user() {
                intent_hashes.push((series_executor.latest_state_version(), intent_hash));
            }

            substate_store_update.apply(commit.database_updates);
            let hash_structures_diff = commit.hash_structures_diff;
            state_tree_update.add(
                series_executor.latest_state_version(),
                hash_structures_diff.state_hash_tree_diff,
            );
            transaction_tree_slice_merger.append(hash_structures_diff.transaction_tree_diff.slice);
            receipt_tree_slice_merger.append(hash_structures_diff.receipt_tree_diff.slice);

            transactions_metrics_data.push(TransactionMetricsData::new(
                raw.0.len(),
                commit
                    .local_receipt
                    .local_execution
                    .execution_metrics
                    .clone(),
            ));
            committed_transaction_bundles.push(CommittedTransactionBundle {
                state_version: series_executor.latest_state_version(),
                raw,
                receipt: commit.local_receipt,
                identifiers: CommittedTransactionIdentifiers {
                    payload: validated.create_identifiers(),
                    resultant_ledger_hashes: *series_executor.latest_ledger_hashes(),
                },
            });
        }

        // Step 4.: Check final invariants, perform the DB commit
        if series_executor.next_epoch() != commit_ledger_header.next_epoch.as_ref() {
            panic!(
                "resultant next epoch at version {} differs from the proof ({:?} != {:?})",
                commit_state_version,
                series_executor.next_epoch(),
                commit_ledger_header.next_epoch
            );
        }

        let final_ledger_hashes = series_executor.latest_ledger_hashes();
        if final_ledger_hashes != &commit_ledger_header.hashes {
            panic!(
                "resultant ledger hashes at version {} differ from the proof ({:?} != {:?})",
                commit_state_version, final_ledger_hashes, commit_ledger_header.hashes
            );
        }

        self.execution_cache
            .lock()
            .progress_base(&final_ledger_hashes.transaction_root);

        let round_counters = leader_round_counters_builder.build(series_executor.epoch_header());

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

        self.mempool_manager
            .remove_committed(intent_hashes.iter().map(|entry| &entry.1));
        self.pending_transaction_result_cache
            .write()
            .track_committed_transactions(SystemTime::now(), intent_hashes);

        self.ledger_metrics.update(
            commit_transactions_len,
            commit_state_version,
            round_counters,
        );
        self.committed_transactions_metrics
            .update(transactions_metrics_data);
        Ok(())
    }

    /// Performs a simplified [`commit()`] flow meant for (internal) genesis transactions.
    /// This method accepts a pre-validated transaction and trusts its contents (i.e. skips some
    /// validations).
    fn commit_genesis(&self, request: GenesisCommitRequest) {
        let mut write_store = self.store.write();
        let mut series_executor = self.start_series_execution(write_store.deref());

        let mut commit = series_executor
            .execute_and_update_state(&request.validated, "genesis")
            .expect("cannot execute genesis");

        if request.require_success {
            commit = commit.expect_success("genesis not successful");
        }

        let resultant_state_version = series_executor.latest_state_version();
        let resultant_ledger_hashes = *series_executor.latest_ledger_hashes();

        self.execution_cache
            .lock()
            .progress_base(&resultant_ledger_hashes.transaction_root);

        let committed_transaction_bundle = CommittedTransactionBundle {
            state_version: resultant_state_version,
            raw: request.raw,
            receipt: commit.local_receipt,
            identifiers: CommittedTransactionIdentifiers {
                payload: request.validated.create_identifiers(),
                resultant_ledger_hashes,
            },
        };

        let hash_structures_diff = commit.hash_structures_diff;
        write_store.commit(CommitBundle {
            transactions: vec![committed_transaction_bundle],
            proof: request.proof,
            substate_store_update: SubstateStoreUpdate::from_single(commit.database_updates),
            vertex_store: None,
            state_tree_update: HashTreeUpdate::from_single(
                resultant_state_version,
                hash_structures_diff.state_hash_tree_diff,
            ),
            transaction_tree_slice: hash_structures_diff.transaction_tree_diff.slice,
            receipt_tree_slice: hash_structures_diff.receipt_tree_diff.slice,
        });
        drop(write_store);

        self.ledger_metrics
            .update(1, resultant_state_version, Vec::new());
    }

    fn find_transaction_root_in_execution_cache(
        &self,
        parent_transaction_root: &TransactionTreeHash,
        transactions: &[PreparedLedgerTransaction],
    ) -> Option<TransactionTreeHash> {
        let execution_cache = self.execution_cache.lock();
        let mut transaction_root = parent_transaction_root;
        for transaction in transactions {
            transaction_root = match execution_cache.get_cached_transaction_root(
                transaction_root,
                &transaction.ledger_transaction_hash(),
            ) {
                Some(cached) => cached,
                None => return None,
            }
        }
        Some(*transaction_root)
    }

    fn calculate_transaction_root(
        store: &S,
        epoch_identifiers: &EpochTransactionIdentifiers,
        parent_version: StateVersion,
        transactions: &[PreparedLedgerTransaction],
    ) -> TransactionTreeHash {
        let epoch_accu_trees =
            EpochAwareAccuTreeFactory::new(epoch_identifiers.state_version, parent_version);
        let transaction_tree_diff = epoch_accu_trees.compute_tree_diff(
            epoch_identifiers.transaction_hash,
            store,
            transactions
                .iter()
                .map(|transaction| transaction.ledger_transaction_hash())
                .map(TransactionTreeHash::from)
                .collect(),
        );
        *transaction_tree_diff.slice.root()
    }
}

struct PendingTransactionResult {
    pub intent_hash: IntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
    pub invalid_at_epoch: Epoch,
    pub rejection_reason: Option<RejectionReason>,
}
