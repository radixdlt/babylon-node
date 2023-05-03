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

use crate::mempool::*;
use crate::types::*;
use crate::{MempoolMetrics, TakesMetricLabels};
use prometheus::Registry;

use std::collections::HashSet;
use std::sync::Arc;

use crate::mempool_relay_dispatcher::MempoolRelayDispatcher;
use crate::simple_mempool::SimpleMempool;
use crate::store::StateManagerDatabase;
use crate::transaction::CachedCommitabilityValidator;
use parking_lot::RwLock;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tracing::warn;
use transaction::model::NotarizedTransaction;

/// A high-level API giving a thread-safe access to the `SimpleMempool`.
pub struct MempoolManager {
    mempool: Arc<RwLock<SimpleMempool>>,
    relay_dispatcher: Option<MempoolRelayDispatcher>,
    cached_commitability_validator: CachedCommitabilityValidator<StateManagerDatabase>,
    metrics: MempoolMetrics,
}

impl MempoolManager {
    /// Creates a manager and registers its metrics.
    pub fn new(
        mempool: Arc<RwLock<SimpleMempool>>,
        relay_dispatcher: Option<MempoolRelayDispatcher>,
        cached_commitability_validator: CachedCommitabilityValidator<StateManagerDatabase>,
        metric_registry: &Registry,
    ) -> Self {
        Self {
            mempool,
            relay_dispatcher,
            cached_commitability_validator,
            metrics: MempoolMetrics::new(metric_registry),
        }
    }
}

impl MempoolManager {
    /// Picks an arbitrary subset of transactions to form the proposal from.
    /// Obeys the given count/size limits and explicit exclusions.
    pub fn get_proposal_transactions(
        &self,
        max_count: u64,
        max_payload_size_bytes: u64,
        user_payload_hashes_to_exclude: &HashSet<UserPayloadHash>,
    ) -> Vec<PendingTransaction> {
        let read_mempool = self.mempool.read();
        let candidate_pending_transactions = read_mempool.get_all_transactions();
        drop(read_mempool);
        let mut payload_size_so_far = 0u64;
        candidate_pending_transactions
            .into_iter()
            .filter(|candidate_transaction| {
                !user_payload_hashes_to_exclude.contains(&candidate_transaction.payload_hash)
            })
            .filter(|transaction| {
                let increased_payload_size = payload_size_so_far + transaction.payload_size as u64;
                let fits = increased_payload_size <= max_payload_size_bytes;
                if fits {
                    payload_size_so_far = increased_payload_size;
                }
                fits
            })
            .take(max_count as usize)
            .collect()
    }

    /// Picks a random subset of transactions to be relayed via a mempool sync.
    /// Obeys the given count/size limits.
    /// Checks the commitability of each transaction considered for relay (using
    /// `CachedCommitabilityValidator`) - in case of rejection, the transaction will not be
    /// returned, but removed from the mempool instead.
    pub fn get_relay_transactions(
        &self,
        max_num_txns: u32,
        max_payload_size_bytes: u32,
    ) -> Vec<PendingTransaction> {
        let read_mempool = self.mempool.read();
        let candidate_transactions = read_mempool.get_all_transactions();
        drop(read_mempool);

        let (transactions_to_relay, transactions_to_remove) = self.check_transactions_to_relay(
            candidate_transactions,
            max_num_txns.try_into().unwrap(),
            max_payload_size_bytes.try_into().unwrap(),
        );

        if !transactions_to_remove.is_empty() {
            let mut write_mempool = self.mempool.write();
            let removed_count = transactions_to_remove
                .iter()
                .filter_map(|transaction_to_remove| {
                    write_mempool.remove_transaction(
                        &transaction_to_remove.intent_hash,
                        &transaction_to_remove.payload_hash,
                    )
                })
                .count();
            drop(write_mempool);
            self.metrics.current_transactions.sub(removed_count as i64);
        }

        transactions_to_relay
    }

    /// Adds the given transaction to the mempool (applying all the commitability checks, see
    /// `add_if_commitable()`), and then triggers an unscheduled mempool sync (propagating only this
    /// transaction to other nodes).
    /// The triggering only takes place if the mempool did not already contain this transaction (to
    /// prevent flooding). Any error encountered during the triggering will only be logged (as
    /// `warn!`) and then ignored.
    /// Although an arbitrary `MempoolAddSource` can be passed, this method is primarily meant for
    /// relaying new transactions submitted via Core API.
    pub fn add_and_trigger_relay(
        &self,
        source: MempoolAddSource,
        transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        self.add_if_commitable(source, transaction.clone())?;

        if let Some(relay_dispatcher) = &self.relay_dispatcher {
            if let Err(error) = relay_dispatcher.trigger_relay(transaction) {
                warn!("Could not trigger a mempool relay: {:?}; ignoring", error);
            }
        }
        Ok(())
    }

    /// Checks the commitability of the given transaction (see `CachedCommitabilityValidator`) and
    /// either adds it to the mempool, or returns the encountered error.
    pub fn add_if_commitable(
        &self,
        source: MempoolAddSource,
        unvalidated_transaction: NotarizedTransaction,
    ) -> Result<(), MempoolAddError> {
        // Quick check to avoid transaction validation if it couldn't be added to the mempool anyway
        let mut write_mempool = self.mempool.write();
        write_mempool.check_add_would_be_possible(&unvalidated_transaction.user_payload_hash())?;
        drop(write_mempool);

        let (record, was_cached) = self
            .cached_commitability_validator
            .check_for_rejection_cached(&unvalidated_transaction);
        let result = record
            .should_accept_into_mempool(was_cached)
            .map_err(MempoolAddError::Rejected);

        match &result {
            Ok(_) => {
                // Note - we purposefully don't save a validated transaction in the mempool:
                // * Currently (Nov 2022) static validation isn't sufficiently static, as it includes EG epoch validation
                // * Moreover, the engine expects the validated transaction to be presently valid, else panics
                // * Once epoch validation is moved to the executor, we can persist validated transactions in the mempool
                let mut write_mempool = self.mempool.write();
                write_mempool.add_transaction(unvalidated_transaction.into(), source)?;
                drop(write_mempool);
                self.metrics.submission_added.with_label(source).inc();
                self.metrics.current_transactions.inc();
            }
            Err(error) => {
                self.metrics
                    .submission_rejected
                    .with_two_labels(source, error)
                    .inc();
            }
        };

        result
    }

    /// Removes all the transactions that have the given intent hashes.
    /// This method is meant to be called for transactions that were successfully committed - and
    /// this assumption is important for metric correctness.
    pub fn remove_committed(&self, intent_hashes: &[IntentHash]) {
        let mut write_mempool = self.mempool.write();
        let removed = intent_hashes
            .iter()
            .flat_map(|intent_hash| write_mempool.remove_transactions(intent_hash))
            .collect::<Vec<_>>();
        drop(write_mempool);
        removed
            .iter()
            .filter(|data| data.source == MempoolAddSource::CoreApi)
            .map(|data| data.added_at.elapsed().as_secs_f64())
            .for_each(|wait| self.metrics.from_local_api_to_commit_wait.observe(wait));
        self.metrics.current_transactions.sub(removed.len() as i64);
    }

    /// Removes the transactions specified by the given user payload hashes (while checking
    /// consistency of their intent hashes).
    /// This method is meant to be called for transactions that were rejected - and
    /// this assumption is important for metric correctness.
    ///
    /// Note:
    /// Removing transactions rejected during prepare from the mempool is a bit of overkill:
    /// just because transactions were rejected in this history doesn't mean this history will be
    /// committed.
    /// But it'll do for now as a defensive measure until we can have a more intelligent mempool.
    pub fn remove_rejected(&self, rejected_transactions: &[(&IntentHash, &UserPayloadHash)]) {
        let mut write_mempool = self.mempool.write();
        let removed_count = rejected_transactions
            .iter()
            .filter_map(|(intent_hash, user_payload_hash)| {
                write_mempool.remove_transaction(intent_hash, user_payload_hash)
            })
            .count();
        drop(write_mempool);
        self.metrics.current_transactions.sub(removed_count as i64);
    }

    /// Checks the given candidate transactions for rejection and decides which ones should be
    /// relayed via mempool sync, and which ones should be removed from mempool.
    fn check_transactions_to_relay(
        &self,
        mut candidate_transactions: Vec<PendingTransaction>,
        max_num_txns: usize,
        max_payload_size_bytes: usize,
    ) -> (Vec<PendingTransaction>, Vec<PendingTransaction>) {
        let mut to_relay = Vec::new();
        let mut payload_size_so_far = 0usize;

        // We (partially) cleanup the mempool on the occasion of getting the relay txns
        // TODO: move this to a separate job
        let mut to_remove = Vec::new();

        candidate_transactions.shuffle(&mut thread_rng());
        for candidate_transaction in candidate_transactions.into_iter() {
            let (record, _) = self
                .cached_commitability_validator
                .check_for_rejection_cached(&candidate_transaction.payload);
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

        (to_relay, to_remove)
    }
}
