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

use crate::mempool_manager::MempoolManager;
use crate::staging::epoch_handling::EpochAwareAccuTreeFactory;
use crate::store::traits::*;
use crate::transaction::*;
use crate::types::CommitRequest;
use crate::*;

use crate::engine_prelude::*;

use node_common::locks::{DbLock, RwLock};

use tracing::warn;

use crate::protocol::*;
use crate::system_commits::*;

use crate::accumulator_tree::storage::ReadableAccuTreeStore;

use std::ops::Deref;
use std::sync::Arc;
use std::time::SystemTime;

pub struct Committer {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    ledger_transaction_validator: Arc<LedgerTransactionValidator>,
    transaction_executor_factory: Arc<TransactionExecutorFactory>,
    mempool_manager: Arc<MempoolManager>,
    execution_cache_manager: Arc<ExecutionCacheManager>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    protocol_state_manager: Arc<ProtocolStateManager>,
    ledger_metrics: Arc<LedgerMetrics>,
}

impl Committer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        transaction_executor_factory: Arc<TransactionExecutorFactory>,
        ledger_transaction_validator: Arc<LedgerTransactionValidator>,
        mempool_manager: Arc<MempoolManager>,
        execution_cache_manager: Arc<ExecutionCacheManager>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
        protocol_state_manager: Arc<ProtocolStateManager>,
        ledger_metrics: Arc<LedgerMetrics>,
    ) -> Self {
        Self {
            database,
            ledger_transaction_validator,
            transaction_executor_factory,
            mempool_manager,
            execution_cache_manager,
            pending_transaction_result_cache,
            protocol_state_manager,
            ledger_metrics,
        }
    }
}

impl Committer {
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
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());

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
            .execution_cache_manager
            .find_transaction_root(
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
        let mut commit_bundle_builder = series_executor.start_commit_builder();

        for ((raw, prepared), proposer_timestamp_ms) in commit_request
            .transactions
            .into_iter()
            .zip(prepared_transactions)
            .zip(proposer_timestamps)
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

            if let ValidatedLedgerTransactionInner::UserV1(user_transaction) = &validated.inner {
                committed_user_transactions.push(CommittedUserTransactionIdentifiers {
                    state_version: series_executor.latest_state_version(),
                    intent_hash: user_transaction.intent_hash(),
                    notarized_transaction_hash: user_transaction.notarized_transaction_hash(),
                });
            }
            transactions_metrics_data.push(TransactionMetricsData::new(&raw, &commit));

            commit_bundle_builder.add_executed_transaction(
                series_executor.latest_state_version(),
                proposer_timestamp_ms,
                raw,
                validated,
                commit,
            );
        }

        let epoch_change = series_executor.epoch_change();
        self.protocol_state_manager
            .update_protocol_state_and_metrics(
                series_executor.protocol_state(),
                epoch_change.as_ref(),
            );

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

        self.execution_cache_manager
            .access_exclusively()
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
            transactions_metrics_data,
        );

        Ok(CommitSummary {
            validator_round_counters: round_counters,
            num_user_transactions,
        })
    }

    /// Performs a simplified [`commit()`] flow meant for system transactions.
    /// This method accepts a pre-validated transaction and trusts its contents (i.e. skips some
    /// validations).
    pub fn commit_system(&self, request: SystemCommitRequest) {
        let SystemCommitRequest {
            raw,
            validated,
            proof,
            require_success,
        } = request;
        let database = self.database.lock();
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());
        let mut commit_bundle_builder = series_executor.start_commit_builder();

        let mut commit = series_executor
            .execute_and_update_state(&validated, "system transaction")
            .expect("cannot execute system transaction");

        let transaction_metrics_data = TransactionMetricsData::new(&raw, &commit);

        if require_success {
            commit = commit.expect_success("system transaction not successful");
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

        self.execution_cache_manager
            .access_exclusively()
            .progress_base(&series_executor.latest_ledger_hashes().transaction_root);

        database.commit(commit_bundle_builder.build(proof, None));

        // Protocol updates aren't allowed during system transactions, so no need to handle an
        // update here, just assign the latest protocol state.
        self.protocol_state_manager
            .update_protocol_state_and_metrics(
                series_executor.protocol_state(),
                series_executor.epoch_change().as_ref(),
            );

        drop(database);

        self.ledger_metrics.update(
            1,
            resultant_state_version,
            Vec::new(),
            proposer_timestamp_ms,
            None,
            vec![transaction_metrics_data],
        );
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
}

pub struct CommittedUserTransactionIdentifiers {
    pub state_version: StateVersion,
    pub intent_hash: IntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}
