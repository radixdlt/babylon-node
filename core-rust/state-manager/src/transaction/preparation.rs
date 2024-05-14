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

use node_common::config::limits::VertexLimitsConfig;
use node_common::locks::*;
use prometheus::Registry;
use std::ops::Deref;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, info};

use crate::engine_prelude::*;
use crate::limits::*;
use crate::*;
use crate::rocks_db::ActualStateManagerDatabase;

use crate::system_commits::*;

use crate::transaction::*;

pub struct Preparator {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    transaction_executor_factory: Arc<TransactionExecutorFactory>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    ledger_transaction_validator: Arc<LedgerTransactionValidator>,
    vertex_prepare_metrics: VertexPrepareMetrics,
    vertex_limits_config: VertexLimitsConfig,
}

impl Preparator {
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        transaction_executor_factory: Arc<TransactionExecutorFactory>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
        ledger_transaction_validator: Arc<LedgerTransactionValidator>,
        vertex_limits_config: VertexLimitsConfig,
        metrics_registry: &Registry,
    ) -> Self {
        Self {
            database,
            transaction_executor_factory,
            pending_transaction_result_cache,
            ledger_transaction_validator,
            vertex_prepare_metrics: VertexPrepareMetrics::new(metrics_registry),
            vertex_limits_config,
        }
    }

    pub fn prepare_genesis(&self, genesis_transaction: GenesisTransaction) -> SystemPrepareResult {
        let raw = LedgerTransaction::Genesis(Box::new(genesis_transaction))
            .to_raw()
            .expect("Could not encode genesis transaction");
        let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw)
            .expect("Could not prepare genesis transaction");
        let validated = self.ledger_transaction_validator.validate_genesis(prepared);

        let database = self.database.lock();
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());

        series_executor
            .execute_and_update_state(&validated, "genesis")
            .expect("genesis not committable")
            .expect_success("genesis");

        SystemPrepareResult::from_committed_series(
            vec![RawAndValidatedTransaction { raw, validated }],
            series_executor,
        )
    }

    pub fn prepare_protocol_update(
        &self,
        flash_transactions: Vec<FlashTransactionV1>,
    ) -> SystemPrepareResult {
        let database = self.database.lock();
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());

        let mut committed_transactions = Vec::new();
        for flash_transaction in flash_transactions {
            let raw = LedgerTransaction::FlashV1(Box::new(flash_transaction))
                .to_raw()
                .unwrap();
            let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw).unwrap();
            let validated = self.ledger_transaction_validator.validate_flash(prepared);

            series_executor
                .execute_and_update_state(&validated, "protocol update")
                .expect("protocol update not committable")
                .expect_success("protocol update");

            committed_transactions.push(RawAndValidatedTransaction { raw, validated });
        }

        SystemPrepareResult::from_committed_series(committed_transactions, series_executor)
    }

    pub fn prepare_scenario(
        &self,
        scenario_name: &str,
        mut scenario: Box<dyn ScenarioInstance>,
    ) -> (SystemPrepareResult, PreparedScenarioMetadata) {
        let database = self.database.lock();
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());

        let mut previous_engine_receipt = None;
        let mut committed_transactions = Vec::new();
        let mut committed_transaction_names = Vec::new();
        loop {
            let next = scenario
                .next(previous_engine_receipt.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let (transaction, engine_receipt) = self.prepare_scenario_transaction(
                        &mut series_executor,
                        scenario_name,
                        &next,
                    );
                    if matches!(engine_receipt.result, TransactionResult::Commit(_)) {
                        committed_transactions.push(transaction);
                        committed_transaction_names
                            .push((series_executor.latest_state_version(), next.logical_name));
                    } else {
                        info!(
                            "Non-committable transaction {} within scenario {}: {:?}",
                            next.logical_name, scenario_name, engine_receipt
                        )
                    }
                    previous_engine_receipt = Some(engine_receipt);
                }
                NextAction::Completed(end_state) => {
                    let prepare_result = SystemPrepareResult::from_committed_series(
                        committed_transactions,
                        series_executor,
                    );
                    let scenario_metadata = PreparedScenarioMetadata {
                        committed_transaction_names,
                        end_state,
                    };
                    return (prepare_result, scenario_metadata);
                }
            }
        }
    }

    fn prepare_scenario_transaction(
        &self,
        series_executor: &mut TransactionSeriesExecutor<ActualStateManagerDatabase>,
        scenario_name: &str,
        next: &NextTransaction,
    ) -> (RawAndValidatedTransaction, TransactionReceipt) {
        let qualified_name = format!(
            "{} scenario - {} transaction",
            scenario_name, &next.logical_name
        );

        let (raw, prepared) = self
            .try_prepare_ledger_transaction_from_user_transaction(&next.raw_transaction)
            .unwrap_or_else(|_| panic!("cannot prepare {}", qualified_name));

        let validated = self
            .ledger_transaction_validator
            .validate_user_or_round_update(prepared)
            .unwrap_or_else(|_| panic!("{} not valid", qualified_name));

        series_executor
            .capture_next_engine_receipt()
            .execute_and_update_state(&validated, qualified_name.as_str())
            .ok(); // we need to consume the `Result<>`, but we actually only care about the receipt
        let engine_receipt = series_executor.retrieve_captured_engine_receipt();

        (
            RawAndValidatedTransaction { raw, validated },
            engine_receipt,
        )
    }

    pub fn prepare(&self, prepare_request: PrepareRequest) -> PrepareResult {
        //========================================================================================
        // NOTE:
        // In this method, "already prepared" transactions that live between the commit point and the current
        // proposal will be referred to as "ancestor" - to distinguish them from "preparation" of the transactions
        // themselves, which is part of the validation process
        //========================================================================================

        let database = self.database.snapshot();
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());

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

        let pending_transaction_base_state =
            AtState::Specific(AtSpecificState::PendingPreparingVertices {
                base_committed_state_version: series_executor.latest_state_version(),
                pending_transactions_root: prepare_request.ancestor_ledger_hashes.transaction_root,
            });

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

    // only public for test purposes
    pub fn try_prepare_ledger_transaction_from_user_transaction(
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
}

pub struct PreparedScenarioMetadata {
    pub committed_transaction_names: Vec<(StateVersion, String)>,
    pub end_state: EndState,
}

struct PendingTransactionResult {
    pub intent_hash: IntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
    pub invalid_at_epoch: Epoch,
    pub rejection_reason: Option<MempoolRejectionReason>,
}
