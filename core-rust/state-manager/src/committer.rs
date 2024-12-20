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

use crate::prelude::*;

use crate::system_commits::*;

pub struct Committer {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    validator: Arc<RwLock<TransactionValidator>>,
    transaction_executor_factory: Arc<TransactionExecutorFactory>,
    mempool_manager: Arc<MempoolManager>,
    execution_cache_manager: Arc<ExecutionCacheManager>,
    protocol_manager: Arc<ProtocolManager>,
    ledger_metrics: Arc<LedgerMetrics>,
    formatter: Arc<Formatter>,
}

impl Committer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        transaction_executor_factory: Arc<TransactionExecutorFactory>,
        ledger_transaction_validator: Arc<RwLock<TransactionValidator>>,
        mempool_manager: Arc<MempoolManager>,
        execution_cache_manager: Arc<ExecutionCacheManager>,
        protocol_manager: Arc<ProtocolManager>,
        ledger_metrics: Arc<LedgerMetrics>,
        formatter: Arc<Formatter>,
    ) -> Self {
        Self {
            database,
            validator: ledger_transaction_validator,
            transaction_executor_factory,
            mempool_manager,
            execution_cache_manager,
            protocol_manager,
            ledger_metrics,
            formatter,
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
        let CommitRequest {
            transactions,
            proof,
            vertex_store,
            self_validator_id,
        } = commit_request;

        let commit_ledger_header = &proof.ledger_header;
        let commit_state_version = commit_ledger_header.state_version;
        let commit_proposer_timestamp_ms = commit_ledger_header.proposer_timestamp_ms;

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
        for (index, raw_transaction) in transactions.iter().enumerate() {
            let result = raw_transaction.prepare(self.validator.read().preparation_settings());
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

            if let PreparedLedgerTransactionInner::Validator(_) = &prepared_transaction.inner {
                let ledger_transaction = LedgerTransaction::from_raw(raw_transaction)
                    .expect("the same transaction was parsed fine above");
                match ledger_transaction {
                    LedgerTransaction::RoundUpdateV1(round_update) => {
                        leader_round_counters_builder.update(&round_update.leader_proposal_history);
                        proposer_timestamp_ms = round_update.proposer_timestamp_ms;
                    }
                    LedgerTransaction::Genesis(_)
                    | LedgerTransaction::UserV1(_)
                    | LedgerTransaction::FlashV1(_)
                    | LedgerTransaction::UserV2(_) => {
                        panic!("A validator transaction can only be a RoundUpdate")
                    }
                }
            }

            prepared_transactions.push(prepared_transaction);
            proposer_timestamps.push(proposer_timestamp_ms);
        }

        // Step 2.: Check invariants, set-up DB update structures
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());
        self.verify_pre_commit_invariants(&series_executor, transactions.len(), &proof);

        debug!(
            "Starting commit of normal transaction batch on top of existing state version {} until state version {}",
            series_executor.latest_state_version(),
            commit_state_version,
        );

        // Naively, the below could be a part of pre-commit invariants (see above); however, we do
        // not want to panic, but rather return an `Err` for the consensus layer, since a
        // transaction root mismatch may mean a malicious peer (and not our inconsistent state).
        let resultant_transaction_root = self
            .execution_cache_manager
            .find_transaction_root(
                &series_executor.latest_ledger_hashes().transaction_root,
                &prepared_transactions,
            )
            .unwrap_or_else(|| {
                Self::calculate_transaction_root(
                    database.deref(),
                    &series_executor.epoch_identifiers(),
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

        for ((raw, prepared), proposer_timestamp_ms) in transactions
            .into_iter()
            .zip(prepared_transactions)
            .zip(proposer_timestamps)
        {
            let validated = prepared
                .validate(
                    self.validator.read().deref(),
                    AcceptedLedgerTransactionKind::UserOrValidator,
                )
                .unwrap_or_else(|error| {
                    panic!("cannot validate transaction to be committed: {error:?}");
                });

            let hashes = validated.create_hashes();
            let executable = validated.create_ledger_executable();

            if let Some(user_hashes) = hashes.as_user() {
                debug!(
                    "Starting commit execution of {} for {:?}",
                    user_hashes
                        .transaction_intent_hash
                        .display(&*self.formatter),
                    series_executor.latest_state_version().next().unwrap(),
                );
            }

            let commit = series_executor
                .execute_and_update_state(&executable, &hashes, "prepared")
                .expect("cannot execute transaction to be committed");

            if let Some(user_hashes) = hashes.as_user() {
                let identifiers = CommittedUserTransactionIdentifiers {
                    state_version: series_executor.latest_state_version(),
                    transaction_intent_hash: user_hashes.transaction_intent_hash,
                    notarized_transaction_hash: user_hashes.notarized_transaction_hash,
                };
                let nullifications = commit.local_receipt.local_execution.nullifications.clone();
                committed_user_transactions.push((identifiers, nullifications));
            }
            transactions_metrics_data.push(TransactionMetricsData::new(&raw, &commit));

            commit_bundle_builder.add_executed_transaction(
                series_executor.latest_state_version(),
                proposer_timestamp_ms,
                raw,
                hashes,
                commit,
            );
        }

        let round_counters = leader_round_counters_builder.build(series_executor.epoch_header());
        let end_state = series_executor.finalize_series(BatchSituation::NonSystem);

        self.protocol_manager
            .update_protocol_state_and_metrics(&end_state);

        // Step 4.: Check final invariants, perform the DB commit
        self.verify_post_commit_invariants(&end_state, &proof);

        database.commit(commit_bundle_builder.build(proof, vertex_store));

        drop(database);

        self.execution_cache_manager
            .access_exclusively()
            .progress_base(&end_state.ledger_hashes.transaction_root);

        let num_user_transactions = committed_user_transactions.len() as u32;

        self.mempool_manager.handle_committed_transactions(
            SystemTime::now(),
            end_state.epoch_change,
            committed_user_transactions,
        );

        self.ledger_metrics.update(
            commit_state_version,
            round_counters.clone(),
            commit_proposer_timestamp_ms,
            self_validator_id,
            transactions_metrics_data,
        );

        Ok(CommitSummary {
            validator_round_counters: round_counters,
            num_user_transactions,
        })
    }

    /// Performs a simplified [`commit()`] flow meant for system transactions.
    /// This method accepts a pre-validated transaction and trusts its contents (i.e. skips some
    /// validations). The pre/post-commit invariants are still checked, for sanity only.
    /// All system transactions are expected to be committable, and the commit request may
    /// additionally require that they are all successful.
    pub fn commit_system(&self, request: SystemCommitRequest) {
        let SystemCommitRequest {
            transactions,
            proof,
            require_committed_successes,
            batch_situation,
        } = request;
        let commit_proposer_timestamp_ms = proof.ledger_header.proposer_timestamp_ms;

        let database = self.database.lock();
        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(database.deref());
        self.verify_pre_commit_invariants(&series_executor, transactions.len(), &proof);

        debug!(
            "Starting commit of system transaction batch on top of existing state version {} until state version {}",
            series_executor.latest_state_version(),
            proof.ledger_header.state_version,
        );

        let mut commit_bundle_builder = series_executor.start_commit_builder();
        let mut transactions_metrics_data = Vec::new();
        for ProcessedLedgerTransaction {
            raw,
            executable,
            hashes,
        } in transactions
        {
            if let Some(user_hashes) = hashes.as_user() {
                debug!(
                    "Starting commit execution of {} for {:?}",
                    user_hashes
                        .transaction_intent_hash
                        .display(&*self.formatter),
                    series_executor.latest_state_version().next().unwrap(),
                );
            }
            let mut commit = series_executor
                .execute_and_update_state(&executable, &hashes, "system transaction")
                .expect("cannot execute system transaction");
            if require_committed_successes {
                commit = commit.expect_success("system transaction not successful");
            }

            transactions_metrics_data.push(TransactionMetricsData::new(&raw, &commit));
            commit_bundle_builder.add_executed_transaction(
                series_executor.latest_state_version(),
                commit_proposer_timestamp_ms,
                raw,
                hashes,
                commit,
            );
        }

        let end_state = series_executor.finalize_series(batch_situation);
        self.verify_post_commit_invariants(&end_state, &proof);

        self.execution_cache_manager
            .access_exclusively()
            .progress_base(&end_state.ledger_hashes.transaction_root);

        database.commit(commit_bundle_builder.build(proof, None));

        self.protocol_manager
            .update_protocol_state_and_metrics(&end_state);

        self.ledger_metrics.update(
            end_state.state_version,
            Vec::new(),
            commit_proposer_timestamp_ms,
            None,
            transactions_metrics_data,
        );
    }

    fn verify_pre_commit_invariants(
        &self,
        pristine_series_executor: &TransactionSeriesExecutor<ActualStateManagerDatabase>,
        commit_transactions_len: usize,
        proof: &LedgerProof,
    ) {
        if commit_transactions_len == 0 {
            panic!("broken invariant: no transactions in a commit request");
        }
        let commit_state_version = proof.ledger_header.state_version;
        let commit_request_start_state_version = commit_state_version
            .relative(-(commit_transactions_len as i128))
            .expect("cannot compute `commit_request_start_state_version`");
        if commit_request_start_state_version != pristine_series_executor.latest_state_version() {
            panic!(
                "broken invariant: commit request assumed state version ({} - {} = {}) while ledger is at {}",
                commit_state_version,
                commit_transactions_len,
                commit_request_start_state_version,
                pristine_series_executor.latest_state_version(),
            );
        }
    }

    fn verify_post_commit_invariants(&self, end_state: &StateTrackerEndState, proof: &LedgerProof) {
        let commit_state_version = proof.ledger_header.state_version;
        if end_state.state_version != commit_state_version {
            panic!(
                "resultant state version differs from the proof ({:?} != {:?})",
                end_state.state_version, commit_state_version,
            );
        }

        if end_state.ledger_hashes != proof.ledger_header.hashes {
            panic!(
                "resultant ledger hashes at version {} differ from the proof ({:?} != {:?})",
                commit_state_version, end_state.ledger_hashes, proof.ledger_header.hashes,
            );
        }

        let next_epoch = end_state.epoch_change.clone().map(NextEpoch::from);
        if next_epoch != proof.ledger_header.next_epoch {
            panic!(
                "resultant next epoch at version {} differs from the proof ({:?} != {:?})",
                commit_state_version, next_epoch, proof.ledger_header.next_epoch
            );
        }

        self.verify_post_commit_protocol_version(
            commit_state_version,
            &end_state.next_protocol_version,
            &proof.ledger_header.next_protocol_version,
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
            epoch_identifiers.transaction_root,
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
    pub transaction_intent_hash: TransactionIntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}
