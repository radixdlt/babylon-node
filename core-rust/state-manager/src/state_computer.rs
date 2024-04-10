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
use node_common::config::limits::VertexLimitsConfig;

use crate::engine_prelude::*;
use ::radix_transactions::model::PrepareError; // disambiguation needed because of a wide prelude

use node_common::locks::{DbLock, LockFactory, Mutex, RwLock};
use prometheus::Registry;
use tracing::{debug, info, warn};

use crate::protocol::*;
use crate::store::traits::scenario::{
    DescribedAddressRendering, ExecutedGenesisScenario, ExecutedGenesisScenarioStore,
    ExecutedScenarioTransaction, ScenarioSequenceNumber,
};

use crate::accumulator_tree::storage::ReadableAccuTreeStore;
use crate::commit_bundle::CommitBundleBuilder;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

pub struct StateComputer {
    network: NetworkDefinition,
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    mempool_manager: Arc<MempoolManager>,
    execution_configurator: Arc<RwLock<ExecutionConfigurator>>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    execution_cache: Mutex<ExecutionCache>,
    ledger_transaction_validator: RwLock<LedgerTransactionValidator>,
    ledger_metrics: LedgerMetrics,
    committed_transactions_metrics: CommittedTransactionsMetrics,
    protocol_metrics: ProtocolMetrics,
    vertex_prepare_metrics: VertexPrepareMetrics,
    vertex_limits_config: VertexLimitsConfig,
    protocol_state: RwLock<ProtocolState>,
}

impl StateComputer {
    // TODO: refactor and maybe make clippy happy too
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network: &NetworkDefinition,
        vertex_limits_config: VertexLimitsConfig,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        mempool_manager: Arc<MempoolManager>,
        execution_configurator: Arc<RwLock<ExecutionConfigurator>>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
        metrics_registry: &Registry,
        lock_factory: LockFactory,
        initial_updatable_config: &ProtocolStateComputerConfig,
        initial_protocol_state: ProtocolState,
    ) -> Self {
        let (current_transaction_root, current_ledger_proposer_timestamp_ms) = database
            .lock()
            .get_latest_proof()
            .map(|proof| proof.ledger_header)
            .map(|header| (header.hashes.transaction_root, header.proposer_timestamp_ms))
            .unwrap_or_else(|| (LedgerHashes::pre_genesis().transaction_root, 0));

        let committed_transactions_metrics =
            CommittedTransactionsMetrics::new(metrics_registry, &execution_configurator.read());

        let protocol_metrics = ProtocolMetrics::new(metrics_registry, &initial_protocol_state);

        StateComputer {
            network: network.clone(),
            database,
            mempool_manager,
            execution_configurator,
            pending_transaction_result_cache,
            execution_cache: lock_factory
                .named("execution_cache")
                .new_mutex(ExecutionCache::new(current_transaction_root)),
            ledger_transaction_validator: lock_factory
                .named("ledger_transaction_validator")
                .new_rwlock(initial_updatable_config.ledger_transaction_validator()),
            vertex_prepare_metrics: VertexPrepareMetrics::new(metrics_registry),
            vertex_limits_config,
            ledger_metrics: LedgerMetrics::new(
                network,
                // we deliberately opt-out of measuring the "technical" locks used inside metrics:
                lock_factory.named("ledger_metrics").not_measured(),
                metrics_registry,
                current_ledger_proposer_timestamp_ms,
            ),
            committed_transactions_metrics,
            protocol_metrics,
            protocol_state: lock_factory
                .named("protocol_state")
                .new_rwlock(initial_protocol_state),
        }
    }

    /// Exposes the [`LedgerMetrics::get_ledger_status()`].
    /// This abstraction leak is needed to transfer the "overall ledger health" information from a
    /// Rust-side (derived) metric, via JNI, to the Java-based "system health" endpoint.
    pub fn get_ledger_status_from_metrics(&self) -> LedgerStatus {
        self.ledger_metrics.get_ledger_status()
    }

    /// Exposes the [`LedgerMetrics::get_recent_self_proposal_miss_statistic()`].
    /// This abstraction leak is needed to transfer this information from a Rust-side (derived)
    /// metric, via JNI, to the Java-based "system health" endpoint.
    pub fn get_recent_self_proposal_miss_statistic(&self) -> RecentSelfProposalMissStatistic {
        self.ledger_metrics
            .get_recent_self_proposal_miss_statistic()
    }
}

pub enum StateComputerRejectReason {
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
        self.state_version = self
            .state_version
            .next()
            .expect("Invalid next state version!");
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
        let ledger_hashes = result.committable_ledger_hashes?;
        self.state_version = self
            .state_version
            .next()
            .expect("Invalid next state version!");
        Some(GenesisCommitRequest {
            raw: result.raw,
            validated: result.validated,
            proof: self.create_proof(ledger_hashes, None),
            require_success: false,
        })
    }

    fn create_proof(&self, hashes: LedgerHashes, next_epoch: Option<NextEpoch>) -> LedgerProof {
        LedgerProof {
            ledger_header: LedgerHeader {
                epoch: self.epoch,
                round: Round::zero(),
                state_version: self.state_version,
                hashes,
                consensus_parent_round_timestamp_ms: self.timestamp,
                proposer_timestamp_ms: self.timestamp,
                next_epoch,
                next_protocol_version: None,
            },
            origin: LedgerProofOrigin::Genesis {
                genesis_opaque_hash: self.genesis_opaque_hash,
            },
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

impl StateComputer {
    pub fn prepare_genesis(&self, genesis_transaction: GenesisTransaction) -> GenesisPrepareResult {
        let raw = LedgerTransaction::Genesis(Box::new(genesis_transaction))
            .to_raw()
            .expect("Could not encode genesis transaction");
        let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw)
            .expect("Could not prepare genesis transaction");
        let validated = self
            .ledger_transaction_validator
            .read()
            .validate_genesis(prepared);

        let database = self.database.lock();
        let mut series_executor = self.start_series_execution(database.deref());

        let commit = series_executor
            .execute_and_update_state(&validated, "genesis")
            .expect("genesis not committable")
            .expect_success("genesis");

        GenesisPrepareResult {
            raw,
            validated,
            ledger_hashes: commit.hash_structures_diff.ledger_hashes,
            next_epoch: commit.epoch_change().map(|ev| ev.into()),
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
            .read()
            .validate_user_or_round_update(prepared_ledger_transaction)
            .unwrap_or_else(|_| panic!("Expected that {} was valid", qualified_name));

        let database = self.database.lock();

        // Note - we first create a basic receipt - because we need it for later
        let basic_receipt = self
            .execution_configurator
            .read()
            .wrap_ledger_transaction(&validated, "scenario transaction")
            .execute_on(database.deref());
        let mut series_executor = self.start_series_execution(database.deref());

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

        let database = self.database.snapshot();
        let mut series_executor = self.start_series_execution(database.deref());

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
                .read()
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
            .read()
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
                    .fee_summary,
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
            // Don't process any additional transactions if protocol update has been enacted.
            // Note that if a protocol update happens at the end of epoch
            // then a ProtocolUpdate stop reason is returned.
            if series_executor.next_protocol_version().is_some() {
                stop_reason = VertexPrepareStopReason::ProtocolUpdate;
                break;
            }

            // Don't process any additional transactions if epoch change has occurred
            if series_executor.epoch_change().is_some() {
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
                .read()
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
                        rejection_reason: Some(MempoolRejectionReason::ValidationError(
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
                            .fee_summary,
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
                Err(ProcessedRejectResult {
                    result,
                    fee_summary,
                }) => {
                    rejected_transactions.push(RejectedTransaction {
                        index: index as u32,
                        intent_hash: Some(intent_hash),
                        notarized_transaction_hash: Some(notarized_transaction_hash),
                        ledger_transaction_hash: Some(ledger_transaction_hash),
                        error: format!("{:?}", &result.reason),
                    });
                    pending_transaction_results.push(PendingTransactionResult {
                        intent_hash,
                        notarized_transaction_hash,
                        invalid_at_epoch,
                        rejection_reason: Some(MempoolRejectionReason::FromExecution(Box::new(
                            result.reason,
                        ))),
                    });

                    // We want to account for rejected execution costs too and stop accordingly since
                    // executing the maximum number of (rejected) transactions in a proposal for the
                    // maximum amount of execution units per transaction is considerably higher than
                    // the vertex execution limit.
                    if let Err(error) =
                        vertex_limits_tracker.count_rejected_transaction(&fee_summary)
                    {
                        stop_reason = VertexPrepareStopReason::LimitExceeded(error);
                        break;
                    }
                }
            }
        }

        for rejection in rejected_transactions.iter() {
            debug!("TXN INVALID: {}", &rejection.error);
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
            next_epoch: series_executor.epoch_change().map(|ev| ev.into()),
            next_protocol_version: series_executor.next_protocol_version(),
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
                    .read()
                    .prepare_from_raw(&raw_ledger_transaction)
                    .map(|prepared_transaction| (raw_ledger_transaction, prepared_transaction))
            })
    }

    fn start_series_execution<'s, S>(&'s self, store: &'s S) -> TransactionSeriesExecutor<'s, S>
    where
        S: ReadableStore + QueryableProofStore + TransactionIdentifierLoader,
    {
        TransactionSeriesExecutor::new(
            store,
            &self.execution_cache,
            self.execution_configurator.deref(),
            self.protocol_state.read().deref().clone(),
        )
    }
}

impl StateComputer {
    pub fn execute_genesis_for_unit_tests_with_config(
        &self,
        consensus_manager_config: ConsensusManagerConfig,
    ) -> LedgerProof {
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
        let initial_timestamp_ms = 1;
        self.execute_genesis(
            genesis_chunks,
            initial_epoch,
            consensus_manager_config,
            initial_timestamp_ms,
            Hash([0; Hash::LENGTH]),
            *DEFAULT_TESTING_FAUCET_SUPPLY,
            vec![],
        )
    }

    /// Performs an [`execute_genesis()`] with a hardcoded genesis data meant for test purposes.
    pub fn execute_genesis_for_unit_tests_with_default_config(&self) -> LedgerProof {
        let default_config = ConsensusManagerConfig {
            max_validators: 10,
            epoch_change_condition: EpochChangeCondition {
                min_round_count: 3,
                max_round_count: 3,
                target_duration_millis: 0,
            },
            num_unstake_epochs: 1,
            total_emission_xrd_per_epoch: Decimal::one(),
            min_validator_reliability: Decimal::one(),
            num_owner_stake_units_unlock_epochs: 2,
            num_fee_increase_delay_epochs: 1,
            validator_creation_usd_cost: Decimal::one(),
        };
        self.execute_genesis_for_unit_tests_with_config(default_config)
    }

    /// Creates and commits a series of genesis transactions (i.e. a bootstrap, then potentially many
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

        let database = self.database.lock();
        if database.get_post_genesis_epoch_proof().is_some() {
            panic!("Can't execute genesis: database already initialized")
        }
        let maybe_top_txn_identifiers = database.get_top_transaction_identifiers();
        drop(database);

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
                            .map(|(descriptor, address)| DescribedAddressRendering {
                                logical_name: descriptor,
                                rendered_address: match address {
                                    DescribedAddress::Global(address) => {
                                        address.to_string(&encoder)
                                    }
                                    DescribedAddress::Internal(address) => {
                                        address.to_string(&encoder)
                                    }
                                    DescribedAddress::NonFungible(nf_global_id) => {
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
                    let database = self.database.lock();
                    database.put_scenario(sequence_number, executed_scenario);
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
    for (requirements, scenario_builders) in scenario_builders() {
            for scenario_builder in scenario_builders {
                let scenario =
                    scenario_builder(ScenarioCore::new(self.network.clone(), epoch, next_nonce));
                if scenario.metadata().logical_name == scenario_name {
                    return Some(scenario);
                }
            }
        }
        None
    }

    /// Validates and commits the transactions from the given request (or returns an error in case
    /// of invalid request).
    /// Persistently stores the transaction payloads and execution results, together with the
    /// associated proof and vertex store state.
    pub fn commit(
        &self,
        commit_request: CommitRequest,
    ) -> Result<CommitSummary, InvalidCommitRequestError> {
        let commit_transactions_len = commit_request.transactions.len();
        if commit_transactions_len == 0 {
            panic!("broken invariant: no transactions in request {commit_request:?}");
        }

        let commit_ledger_header = &commit_request.proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;
        let commit_request_start_state_version =
            commit_state_version.relative(-(commit_transactions_len as i128))
                .expect("`commit_request_start_state_version` should be computable from `commit_state_version - commit_transactions_len` and valid.");

        // We advance the top-of-ledger, hence lock:
        let database = self.database.lock();

        // Step 1.: Parse the transactions (and collect specific metrics from them, as a drive-by)
        let mut prepared_transactions = Vec::new();
        let mut leader_round_counters_builder = LeaderRoundCountersBuilder::default();
        let mut proposer_timestamps = Vec::new();
        let mut proposer_timestamp_ms = database
            .get_latest_proof()
            .unwrap()
            .ledger_header
            .proposer_timestamp_ms;
        for (index, raw_transaction) in commit_request.transactions.iter().enumerate() {
            let result = self
                .ledger_transaction_validator
                .read()
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
                    proposer_timestamp_ms = round_update.proposer_timestamp_ms;
                }
            }

            prepared_transactions.push(prepared_transaction);
            proposer_timestamps.push(proposer_timestamp_ms);
        }

        // Step 2.: Check invariants, set-up DB update structures
        let mut series_executor = self.start_series_execution(database.deref());

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
                    database.deref(),
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

        let mut transactions_metrics_data = Vec::new();
        let mut committed_user_transactions = Vec::new();

        // Step 3.: Actually execute the transactions, collect their results into DB structures
        let mut commit_bundle_builder = CommitBundleBuilder::new(
            series_executor.epoch_identifiers().state_version,
            series_executor.latest_state_version(),
        );
        for ((raw, prepared), proposer_timestamp_ms) in commit_request
            .transactions
            .into_iter()
            .zip(prepared_transactions)
            .zip(proposer_timestamps)
        {
            let validated = self
                .ledger_transaction_validator
                .read()
                .validate_user_or_round_update(prepared)
                .unwrap_or_else(|error| {
                    panic!("cannot validate transaction to be committed: {error:?}");
                });

            let commit = series_executor
                .execute_and_update_state(&validated, "prepared")
                .expect("cannot execute transaction to be committed");

            if let ValidatedLedgerTransactionInner::UserV1(user_transaction) = &validated.inner {
                committed_user_transactions.push(CommittedUserTransactionIdentifiers {
                    state_version: series_executor.latest_state_version(),
                    intent_hash: user_transaction.intent_hash(),
                    notarized_transaction_hash: user_transaction.notarized_transaction_hash(),
                });
            }
            transactions_metrics_data.push(TransactionMetricsData::new(
                raw.0.len(),
                commit.local_receipt.local_execution.fee_summary.clone(),
            ));

            commit_bundle_builder.add_executed_transaction(
                series_executor.latest_state_version(),
                proposer_timestamp_ms,
                raw,
                validated,
                commit,
            );
        }

        let new_protocol_state = series_executor.protocol_state();

        // Update the protocol metrics
        let epoch_change = series_executor.epoch_change();
        if let Some(epoch_change) = &epoch_change {
            self.protocol_metrics
                .update(&new_protocol_state, epoch_change)
        }

        // Step 4.: Check final invariants, perform the DB commit
        let next_epoch: Option<NextEpoch> = epoch_change.map(|ev| ev.into());
        if next_epoch != commit_ledger_header.next_epoch {
            panic!(
                "resultant next epoch at version {} differs from the proof ({:?} != {:?})",
                commit_state_version, next_epoch, commit_ledger_header.next_epoch
            );
        }

        self.verify_post_commit_protocol_version(
            commit_state_version,
            &series_executor.next_protocol_version(),
            &commit_ledger_header.next_protocol_version,
        );

        let final_ledger_hashes = series_executor.latest_ledger_hashes();
        if final_ledger_hashes != &commit_ledger_header.hashes {
            panic!(
                "resultant ledger hashes at version {} differ from the proof ({:?} != {:?})",
                commit_state_version, final_ledger_hashes, commit_ledger_header.hashes
            );
        }

        // capture these before we lose the appropriate borrows:
        let proposer_timestamp_ms = commit_ledger_header.proposer_timestamp_ms;
        let round_counters = leader_round_counters_builder.build(series_executor.epoch_header());
        let final_transaction_root = final_ledger_hashes.transaction_root;

        database
            .commit(commit_bundle_builder.build(commit_request.proof, commit_request.vertex_store));
        drop(database);

        self.execution_cache
            .lock()
            .progress_base(&final_transaction_root);

        self.mempool_manager.remove_committed(
            committed_user_transactions
                .iter()
                .map(|txn| &txn.intent_hash),
        );

        if let Some(next_epoch) = next_epoch {
            self.mempool_manager
                .remove_txns_where_end_epoch_expired(next_epoch.epoch);
        }

        let num_user_transactions = committed_user_transactions.len() as u32;
        self.pending_transaction_result_cache
            .write()
            .track_committed_transactions(SystemTime::now(), committed_user_transactions);

        self.ledger_metrics.update(
            commit_transactions_len,
            commit_state_version,
            round_counters.clone(),
            proposer_timestamp_ms,
            commit_request.self_validator_id,
        );

        self.committed_transactions_metrics
            .update(transactions_metrics_data);

        *self.protocol_state.write() = new_protocol_state;

        Ok(CommitSummary {
            validator_round_counters: round_counters,
            num_user_transactions,
        })
    }

    fn verify_post_commit_protocol_version(
        &self,
        commit_state_version: StateVersion,
        next_version_from_local_execution: &Option<ProtocolVersionName>,
        next_version_from_proof: &Option<ProtocolVersionName>,
    ) {
        match (next_version_from_local_execution, next_version_from_proof) {
            (Some(local_version), Some(proof_version)) if local_version == proof_version => {
                // All good, local execution enacts the same protocol version as proof. No-op.
            }
            (Some(local_version), Some(proof_version)) => {
                // We're enacting a different version than the proof
                panic!(
                    "FATAL: At state version {} this node wants to enact {}, but this doesn't match the proof, which enacts {} (i.e. the validator set have enacted a different version). Make sure your node is running the latest software version.",
                    commit_state_version,
                    local_version,
                    proof_version,
                );
            }
            (Some(local_version), None) => {
                // We're enacting locally, but the proof doesn't enact anything
                panic!(
                    "FATAL: At state version {} this node wants to enact {}, but this doesn't match the proof, which doesn't enact anything (i.e. the validator set haven't enacted it). Make sure your node is running the latest software version.",
                    commit_state_version,
                    local_version,
                );
            }
            (None, Some(proof_version)) => {
                // The proof enacts, but we locally don't
                panic!(
                    "FATAL: At state version {} the validator set have enacted {}, but this node didn't expect it to happen (expected enactment at a different time, or protocol version unknown). Make sure your node is running the latest software version.",
                    commit_state_version,
                    proof_version,
                );
            }
            (None, None) => {
                // All good, nothing gets enacted. No-op.
            }
        }
    }

    pub fn handle_protocol_update(
        &self,
        protocol_version_name: &ProtocolVersionName,
        new_ledger_transaction_validator: LedgerTransactionValidator,
    ) {
        *self.ledger_transaction_validator.write() = new_ledger_transaction_validator;

        self.protocol_state.write().current_protocol_version = protocol_version_name.clone();

        let current_header = self
            .database
            .lock()
            .get_latest_proof()
            .map(|proof| proof.ledger_header)
            .expect("Can't apply a protocol update pre-genesis");

        // Protocol update might change transaction execution rules, so we need to clean the cache
        *self.execution_cache.lock() = ExecutionCache::new(current_header.hashes.transaction_root);
    }

    /// Performs a simplified [`commit()`] flow meant for (internal) genesis transactions.
    /// This method accepts a pre-validated transaction and trusts its contents (i.e. skips some
    /// validations).
    fn commit_genesis(&self, request: GenesisCommitRequest) {
        let GenesisCommitRequest {
            raw,
            validated,
            proof,
            require_success,
        } = request;
        let database = self.database.lock();
        let mut series_executor = self.start_series_execution(database.deref());
        let mut commit_bundle_builder = CommitBundleBuilder::new(
            series_executor.epoch_identifiers().state_version,
            series_executor.latest_state_version(),
        );

        let mut commit = series_executor
            .execute_and_update_state(&validated, "genesis")
            .expect("cannot execute genesis");

        if require_success {
            commit = commit.expect_success("genesis not successful");
        }

        let proposer_timestamp_ms = proof.ledger_header.proposer_timestamp_ms;
        let resultant_state_version = series_executor.latest_state_version();

        commit_bundle_builder.add_executed_transaction(
            resultant_state_version,
            proposer_timestamp_ms,
            raw,
            validated,
            commit,
        );

        self.execution_cache
            .lock()
            .progress_base(&series_executor.latest_ledger_hashes().transaction_root);

        database.commit(commit_bundle_builder.build(proof, None));

        // Protocol updates aren't allowed during genesis, so no need to handle an update here, just
        // assign the latest protocol state.
        let mut locked_protocol_state = self.protocol_state.write();
        *locked_protocol_state = series_executor.protocol_state();
        drop(locked_protocol_state);

        drop(database);

        self.ledger_metrics.update(
            1,
            resultant_state_version,
            Vec::new(),
            proposer_timestamp_ms,
            None,
        );
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

    fn calculate_transaction_root<S: ReadableAccuTreeStore<StateVersion, TransactionTreeHash>>(
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

    pub fn current_protocol_version(&self) -> ProtocolVersionName {
        self.protocol_state.read().current_protocol_version.clone()
    }

    pub fn protocol_state(&self) -> ProtocolState {
        self.protocol_state.read().deref().clone()
    }
}

pub struct CommittedUserTransactionIdentifiers {
    pub state_version: StateVersion,
    pub intent_hash: IntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}

struct PendingTransactionResult {
    pub intent_hash: IntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
    pub invalid_at_epoch: Epoch,
    pub rejection_reason: Option<MempoolRejectionReason>,
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::engine_prelude::*;
    use crate::transaction::{LedgerTransaction, RoundUpdateTransactionV1};
    use crate::{
        LedgerProof, PrepareRequest, PrepareResult, RoundHistory, StateManager, StateManagerConfig,
    };
    use node_common::config::limits::VertexLimitsConfig;
    use node_common::locks::LockFactory;
    use node_common::scheduler::Scheduler;
    use prometheus::Registry;
    use tempfile::TempDir;

    // TODO: maybe move/refactor testing infra as we add more Rust tests
    fn build_unit_test_round_history(proof: &LedgerProof) -> RoundHistory {
        RoundHistory {
            is_fallback: false,
            epoch: proof.ledger_header.epoch,
            round: Round::of(proof.ledger_header.round.number() + 1),
            gap_round_leader_addresses: Vec::new(),
            proposer_address: proof
                .ledger_header
                .next_epoch
                .clone()
                .unwrap()
                .validator_set[0]
                .address,
            proposer_timestamp_ms: proof.ledger_header.proposer_timestamp_ms,
        }
    }

    fn build_unit_test_prepare_request(
        proof: &LedgerProof,
        proposed_transactions: Vec<RawNotarizedTransaction>,
    ) -> PrepareRequest {
        PrepareRequest {
            committed_ledger_hashes: proof.ledger_header.hashes,
            ancestor_transactions: Vec::new(),
            ancestor_ledger_hashes: proof.ledger_header.hashes,
            proposed_transactions,
            round_history: build_unit_test_round_history(proof),
        }
    }

    fn build_committable_transaction(epoch: Epoch, nonce: u32) -> RawNotarizedTransaction {
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let notary_private_key = Secp256k1PrivateKey::from_u64(2).unwrap();

        TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: epoch,
                end_epoch_exclusive: epoch.after(100).unwrap(),
                nonce,
                notary_public_key: notary_private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .get_free_xrd_from_faucet()
                    .try_deposit_entire_worktop_or_abort(
                        ComponentAddress::virtual_account_from_public_key(
                            &sig_1_private_key.public_key(),
                        ),
                        None,
                    )
                    .build(),
            )
            .sign(&sig_1_private_key)
            .notarize(&notary_private_key)
            .build()
            .to_raw()
            .unwrap()
    }

    fn build_rejected_transaction(epoch: Epoch, nonce: u32) -> RawNotarizedTransaction {
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let notary_private_key = Secp256k1PrivateKey::from_u64(2).unwrap();

        TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: epoch,
                end_epoch_exclusive: epoch.after(100).unwrap(),
                nonce,
                notary_public_key: notary_private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new().get_free_xrd_from_faucet().build())
            .sign(&sig_1_private_key)
            .notarize(&notary_private_key)
            .build()
            .to_raw()
            .unwrap()
    }

    fn setup_state_manager(
        tmp: &TempDir,
        vertex_limits_config: VertexLimitsConfig,
    ) -> (LedgerProof, StateManager) {
        let lock_factory = LockFactory::new("testing");
        let metrics_registry = Registry::new();

        let config = StateManagerConfig {
            vertex_limits_config: Some(vertex_limits_config),
            ..StateManagerConfig::new_for_testing(tmp.path().to_str().unwrap())
        };
        let state_manager = StateManager::new(
            config,
            None,
            &lock_factory,
            &metrics_registry,
            &Scheduler::new("testing"),
        );

        let proof = state_manager
            .state_computer
            .execute_genesis_for_unit_tests_with_default_config();

        (proof, state_manager)
    }

    fn prepare_with_vertex_limits(
        tmp: &TempDir,
        vertex_limits_config: VertexLimitsConfig,
        proposed_transactions: Vec<RawNotarizedTransaction>,
    ) -> PrepareResult {
        let (proof, state_manager) = setup_state_manager(tmp, vertex_limits_config);
        state_manager
            .state_computer
            .prepare(build_unit_test_prepare_request(
                &proof,
                proposed_transactions,
            ))
    }

    fn compute_consumed_execution_units(
        state_manager: &StateManager,
        prepare_request: PrepareRequest,
    ) -> u32 {
        let database = state_manager.state_computer.database.snapshot();
        let mut series_executor = state_manager
            .state_computer
            .start_series_execution(database.deref());

        let round_update = RoundUpdateTransactionV1::new(
            series_executor.epoch_header(),
            &prepare_request.round_history,
        );
        let ledger_round_update = LedgerTransaction::RoundUpdateV1(Box::new(round_update));
        let validated_round_update = state_manager
            .state_computer
            .ledger_transaction_validator
            .read()
            .validate_user_or_round_update_from_model(&ledger_round_update)
            .expect("expected to be able to prepare the round update transaction");

        let round_update_result = series_executor
            .execute_and_update_state(&validated_round_update, "cost computation - round update")
            .expect("round update rejected");

        prepare_request
            .proposed_transactions
            .iter()
            .map(|raw_user_transaction| {
                let (_, prepared_transaction) = state_manager
                    .state_computer
                    .try_prepare_ledger_transaction_from_user_transaction(raw_user_transaction)
                    .unwrap();

                let validated = state_manager
                    .state_computer
                    .ledger_transaction_validator
                    .read()
                    .validate_user_or_round_update(prepared_transaction)
                    .unwrap();

                let execute_result =
                    series_executor.execute_and_update_state(&validated, "cost computation");

                match execute_result {
                    Ok(commit) => {
                        commit
                            .local_receipt
                            .local_execution
                            .fee_summary
                            .total_execution_cost_units_consumed
                    }
                    Err(reject) => reject.fee_summary.total_execution_cost_units_consumed,
                }
            })
            .sum::<u32>()
            + round_update_result
                .local_receipt
                .local_execution
                .fee_summary
                .total_execution_cost_units_consumed
    }

    #[test]
    fn test_prepare_vertex_limits() {
        let tmp = tempfile::tempdir().unwrap();
        let (proof, state_manager) = setup_state_manager(&tmp, VertexLimitsConfig::max());

        let mut proposed_transactions = Vec::new();
        let epoch = proof.ledger_header.epoch;
        proposed_transactions.push(build_committable_transaction(epoch, 1));
        proposed_transactions.push(build_committable_transaction(epoch, 2));
        proposed_transactions.push(build_rejected_transaction(epoch, 1));
        proposed_transactions.push(build_committable_transaction(epoch, 3));
        proposed_transactions.push(build_committable_transaction(epoch, 4));
        proposed_transactions.push(build_rejected_transaction(epoch, 2));
        proposed_transactions.push(build_committable_transaction(epoch, 5));
        proposed_transactions.push(build_rejected_transaction(epoch, 3));
        proposed_transactions.push(build_committable_transaction(epoch, 6));
        proposed_transactions.push(build_committable_transaction(epoch, 7));
        proposed_transactions.push(build_rejected_transaction(epoch, 4));
        proposed_transactions.push(build_committable_transaction(epoch, 8));
        proposed_transactions.push(build_committable_transaction(epoch, 9));
        proposed_transactions.push(build_rejected_transaction(epoch, 5));

        let prepare_result = state_manager
            .state_computer
            .prepare(build_unit_test_prepare_request(
                &proof,
                proposed_transactions.clone(),
            ));

        assert_eq!(prepare_result.committed.len(), 10); // 9 committable transactions + 1 round update transaction
        assert_eq!(prepare_result.rejected.len(), 5); // 5 rejected transactions

        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig {
                max_transaction_count: 6,
                ..VertexLimitsConfig::max()
            },
            proposed_transactions.clone(),
        );

        assert_eq!(prepare_result.committed.len(), 6); // same as the limit
                                                       // only first 7 (5 committable) transactions are executed before the limit is hit, at which point we have encountered only 2 rejected transactions
        assert_eq!(prepare_result.rejected.len(), 2);

        let limited_proposal_ledger_hashes = prepare_result.ledger_hashes;

        // We now compute PrepareResult only for the first 7 transactions in order to test that indeed resultant states are the same.
        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig::max(),
            proposed_transactions.clone()[0..7].to_vec(),
        );
        assert_eq!(prepare_result.committed.len(), 6);
        assert_eq!(prepare_result.rejected.len(), 2);
        assert_eq!(prepare_result.ledger_hashes, limited_proposal_ledger_hashes);

        // Transaction size/count only tests `check_pre_execution`. We also need to test `try_next_transaction`.
        let tmp = tempfile::tempdir().unwrap();
        let cost_for_first_9_user_transactions = compute_consumed_execution_units(
            &setup_state_manager(&tmp, VertexLimitsConfig::max()).1,
            build_unit_test_prepare_request(&proof, proposed_transactions.clone()[0..9].to_vec()),
        );
        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig {
                // We add an extra cost unit in order to not trigger the LimitExceeded right at 9th transaction.
                max_total_execution_cost_units_consumed: cost_for_first_9_user_transactions + 1,
                ..VertexLimitsConfig::max()
            },
            proposed_transactions.clone(),
        );
        assert_eq!(prepare_result.committed.len(), 7); // in the first 9 proposed transactions we have 6 that gets committed + 1 round update transaction
        assert_eq!(prepare_result.rejected.len(), 4); // 3 rejected transactions + last one that is committable but gets discarded due to limits

        let limited_proposal_ledger_hashes = prepare_result.ledger_hashes;
        let tmp = tempfile::tempdir().unwrap();
        let prepare_result = prepare_with_vertex_limits(
            &tmp,
            VertexLimitsConfig::max(),
            proposed_transactions.clone()[0..9].to_vec(),
        );

        // Should be identical to previous prepare run (cost limited)
        assert_eq!(prepare_result.committed.len(), 7);
        assert_eq!(prepare_result.rejected.len(), 3);
        assert_eq!(prepare_result.ledger_hashes, limited_proposal_ledger_hashes);
    }
}
