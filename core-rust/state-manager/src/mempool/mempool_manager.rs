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

use std::time::Instant;

use tracing::warn;

/// A high-level API giving thread-safe access to all aspects of pending transaction state:
/// * [`PriorityMempool`]
/// * [`PendingTransactionResultCache`] and [`CommittabilityValidator`] through the
///   [`CachedCommittabilityValidator`]
///
/// It is intended to encapsulate the logic, invariants/atomicity, and lock safety of
/// this system.
///
/// This means that the API of the mempool manager is relatively wide, as it covers
/// interactions with the mempool and results cache during submit, mempool sync,
/// prepare and commit.
///
/// This encapsulation will also allow us to safely refactor the internals of this
/// logic, for example, by combining the [`PriorityMempool`] and [`PendingTransactionResultCache`]
/// in future.
///
/// In particular, the [`MempoolManager`] is responsible for preventing deadlocks,
/// by ensuring that locks, if taken out simultaneously, are taken out in a very
/// particular order:
/// * First, the [`PendingTransactionResultCache`] lock.
/// * Then, the [`PriorityMempool`] lock.
pub struct MempoolManager {
    mempool: RwLock<PriorityMempool>,
    relay_dispatcher: Option<MempoolRelayDispatcher>,
    cached_committability_validator: CachedCommittabilityValidator,
    metrics: MempoolManagerMetrics,
}

impl MempoolManager {
    /// Creates a manager and registers its metrics.
    pub fn new(
        mempool: RwLock<PriorityMempool>,
        relay_dispatcher: MempoolRelayDispatcher,
        cached_committability_validator: CachedCommittabilityValidator,
        metric_registry: &MetricRegistry,
    ) -> Self {
        Self {
            mempool,
            relay_dispatcher: Some(relay_dispatcher),
            cached_committability_validator,
            metrics: MempoolManagerMetrics::new(metric_registry),
        }
    }

    /// Creates a testing manager (without the JNI-based relay dispatcher) and registers its metrics.
    pub fn new_for_testing(
        mempool: RwLock<PriorityMempool>,
        cached_committability_validator: CachedCommittabilityValidator,
        metric_registry: &MetricRegistry,
    ) -> Self {
        Self {
            mempool,
            relay_dispatcher: None,
            cached_committability_validator,
            metrics: MempoolManagerMetrics::new(metric_registry),
        }
    }

    /// This is *INTERNAL ONLY* as it must be used carefully to avoid deadlocks,
    /// as described in the RustDoc of [`MempoolManager`].
    fn cache(&self) -> &RwLock<PendingTransactionResultCache> {
        self.cached_committability_validator.get_cache()
    }

    pub fn get_proposal_transactions(
        &self,
        max_count: usize,
        max_payload_size_bytes: u64,
        user_payload_hashes_to_exclude: &HashSet<NotarizedTransactionHash>,
    ) -> Vec<Arc<MempoolTransaction>> {
        self.mempool.read().get_proposal_transactions(
            max_count,
            max_payload_size_bytes,
            user_payload_hashes_to_exclude,
        )
    }

    pub fn get_mempool_count(&self) -> usize {
        self.mempool.read().get_count()
    }

    /// Picks a random subset of transactions to be relayed via a mempool sync.
    /// Obeys the given count/size limits.
    pub fn get_relay_transactions(
        &self,
        max_count: usize,
        max_payload_size_bytes: u64,
    ) -> Vec<Arc<MempoolTransaction>> {
        // TODO: Definitely a better algorithm could be used here, especially with extra information like:
        // which peer/peers are we sending this to? or what do we know about said peer to have in it's mempool?
        // However (NOTE/WARN): changing transactions selection without careful consideration of the peer selection,
        // can lead to a scenario where we keep sending same transactions to same peer.
        let candidate_transactions = {
            // We use a block to scope the lock guard
            self.mempool.read().get_k_random_transactions(max_count * 2)
        };

        let mut payload_size_so_far = 0;
        candidate_transactions
            .into_iter()
            .filter(|transaction| {
                let increased_payload_size = payload_size_so_far + transaction.raw.len() as u64;
                let fits = increased_payload_size <= max_payload_size_bytes;
                if fits {
                    payload_size_so_far = increased_payload_size;
                }
                fits
            })
            .take(max_count)
            .collect()
    }

    pub fn all_known_pending_payloads_for_intent(
        &self,
        intent_hash: &TransactionIntentHash,
    ) -> HashMap<NotarizedTransactionHash, PendingTransactionRecord> {
        self.cache()
            .read()
            .peek_all_known_payloads_for_intent(intent_hash)
    }

    pub fn get_mempool_payload_hashes_for_intent(
        &self,
        intent_hash: &TransactionIntentHash,
    ) -> Vec<NotarizedTransactionHash> {
        self.mempool
            .read()
            .get_notarized_transaction_hashes_for_intent(intent_hash)
    }

    pub fn get_mempool_payload(
        &self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<Arc<MempoolTransaction>> {
        self.mempool.read().get_payload(notarized_transaction_hash)
    }

    pub fn get_mempool_all_hashes(&self) -> Vec<(TransactionIntentHash, NotarizedTransactionHash)> {
        self.mempool
            .read()
            .all_hashes_iter()
            .map(|(intent_hash, payload_hash)| (*intent_hash, *payload_hash))
            .collect()
    }

    /// Checks the committability of up to `max_reevaluated_count` of transactions executed against
    /// earliest state versions and removes the newly rejected ones from the mempool.
    pub fn reevaluate_transaction_committability(&self, max_reevaluated_count: u32) {
        let candidate_transactions = {
            // We use a block to scope the lock guard
            self.mempool
                .read()
                .iter_by_state_version()
                .take(max_reevaluated_count as usize)
                .collect::<Vec<_>>() // collect, to allow releasing the lock early
        };

        for candidate_transaction in candidate_transactions {
            // invoking the check automatically removes the transaction when rejected
            self.cached_committability_validator
                .check_for_rejection_validated(
                    &candidate_transaction.transaction.executable,
                    &candidate_transaction.transaction.hashes,
                );
        }
    }

    /// Adds the given transaction to the mempool (applying all the committability checks, see
    /// `add_if_committable()`), and then triggers an unscheduled mempool sync (propagating only this
    /// transaction to other nodes).
    /// The triggering only takes place if the mempool did not already contain this transaction (to
    /// prevent flooding). Any error encountered during the triggering will only be logged (as
    /// `warn!`) and then ignored.
    /// Although an arbitrary `MempoolAddSource` can be passed, this method is primarily meant for
    /// relaying new transactions submitted via Core API.
    pub fn add_and_trigger_relay(
        &self,
        source: MempoolAddSource,
        transaction: RawNotarizedTransaction,
        force_recalculate: bool,
    ) -> Result<(), MempoolAddError> {
        let added_transaction = self.add_if_committable(source, transaction, force_recalculate)?;

        if let Some(relay_dispatcher) = &self.relay_dispatcher {
            if let Err(error) = relay_dispatcher.trigger_relay(&added_transaction.raw) {
                warn!("Could not trigger a mempool relay: {:?}; ignoring", error);
            }
        }
        Ok(())
    }

    /// A wrapper around [`self.add_if_committable_internal`] that catches all submission rejections and reports them to Prometheus.
    pub fn add_if_committable(
        &self,
        source: MempoolAddSource,
        raw_transaction: RawNotarizedTransaction,
        force_recalculate: bool,
    ) -> Result<Arc<MempoolTransaction>, MempoolAddError> {
        let start = Instant::now();
        let result = self.add_if_committable_internal(source, raw_transaction, force_recalculate);
        self.metrics
            .submission_attempt
            .with_two_labels(source, MempoolAddResult::new(&result))
            .observe(start.elapsed().as_secs_f64());
        result
    }

    /// Checks the committability of the given transaction (see `CachedCommittabilityValidator`) and
    /// either adds it to the mempool, or returns the encountered error.
    fn add_if_committable_internal(
        &self,
        source: MempoolAddSource,
        raw_transaction: RawNotarizedTransaction,
        force_recalculate: bool,
    ) -> Result<Arc<MempoolTransaction>, MempoolAddError> {
        // STEP 1 - We prepare the transaction to check it's in the right structure and so we have hashes to work with
        let prepared = match self
            .cached_committability_validator
            .prepare_from_raw(&raw_transaction)
        {
            Ok(prepared) => prepared,
            Err(prepare_error) => {
                // If the transaction fails to prepare at this point then we don't even have a hash to assign against it,
                // so we can't cache anything - just return an error
                return Err(MempoolAddError::Rejected(
                    MempoolAddRejection::for_static_rejection(prepare_error.into()),
                    None,
                ));
            }
        };

        let notarized_transaction_hash = prepared.notarized_transaction_hash();

        // STEP 2 - Check if transaction is already in the mempool to avoid transaction execution.
        let is_in_mempool = {
            // We use a block to scope the lock guard
            self.mempool
                .read()
                .contains_transaction(&notarized_transaction_hash)
        };

        if is_in_mempool {
            return Err(MempoolAddError::Duplicate(notarized_transaction_hash));
        }

        // STEP 3 - We validate + run the transaction through
        let force_recalculation = if force_recalculate {
            ForceRecalculation::Yes
        } else {
            // Note - if we've got to this point then there's room in the mempool for this transaction.
            // We need to get a validated transaction to add into the mempool, so need to recalculate it.
            // IMPORTANT: This is also a precondition for calling `should_accept_into_mempool`
            ForceRecalculation::IfCachedAsValid
        };
        let (record, check_result) = self
            .cached_committability_validator
            .check_for_rejection_cached(prepared, force_recalculation);

        // STEP 4 - We check if the result should mean we add the transaction to our mempool
        let PendingExecutedTransaction {
            executable,
            user_hashes,
            latest_attempt_against_state,
        } = record
            .should_accept_into_mempool(check_result)
            .map_err(|rejection| {
                MempoolAddError::Rejected(rejection, Some(notarized_transaction_hash))
            })?;

        let mempool_transaction = Arc::new(MempoolTransaction {
            executable,
            hashes: user_hashes,
            raw: raw_transaction,
        });
        match self.mempool.write().add_transaction_if_not_present(
            mempool_transaction.clone(),
            source,
            StdInstant::now(),
            latest_attempt_against_state.committed_version(),
        ) {
            Ok(_evicted) => Ok(mempool_transaction),
            Err(error) => Err(error),
        }
    }

    pub fn observe_pending_execution_result(
        &self,
        user_transaction_hashes: UserTransactionHashes,
        invalid_at_epoch: Option<Epoch>,
        attempt: TransactionAttempt,
    ) {
        // Taking out both locks enforces atomicity of the update across the mempool and cache.
        // SAFETY: We use the correct order as described in the `MempoolManager` RustDoc.
        let mut pending_cache = self.cache().write();
        let mut mempool = self.mempool.write();

        mempool.observe_pending_execution_result(
            &user_transaction_hashes.notarized_transaction_hash,
            &attempt,
        );
        pending_cache.track_transaction_result(user_transaction_hashes, invalid_at_epoch, attempt);
    }

    /// Removes all the transactions that have the given intent hashes.
    /// This method is meant to be called for transactions that were successfully committed - and
    /// this assumption is important for metric correctness.
    pub fn handle_committed_transactions(
        &self,
        commit_time: SystemTime,
        epoch_change: Option<EpochChangeEvent>,
        committed_transactions: Vec<(CommittedUserTransactionIdentifiers, Vec<Nullification>)>,
    ) {
        // TODO:CUTTLEFISH - We should review the atomicity of these operations between the mempool
        // and the cache. They weren't atomic before, but maybe they should be.

        if let Some(epoch_change) = epoch_change {
            self.mempool
                .write()
                .remove_txns_where_end_epoch_expired(epoch_change.epoch);
        }

        for (committed, nullifications) in committed_transactions.iter() {
            for nullification in nullifications.iter() {
                let Nullification::Intent { intent_hash, .. } = nullification;
                match intent_hash {
                    IntentHash::Transaction(transaction_intent_hash) => {
                        let removed_payloads = {
                            // SAFETY: Take out both locks at once for atomicity.
                            // This take order is as per that of locks in `MempoolManager` RustDoc.
                            // At some point, we should consider merging the cache and the mempool into a single structure.
                            let mut cache = self.cache().write();
                            let mut mempool = self.mempool.write();

                            cache.handle_nullified_transaction_intent(
                                commit_time,
                                committed,
                                *transaction_intent_hash,
                            );
                            mempool.remove_by_intent_hash(transaction_intent_hash)
                        };

                        for removed_data in removed_payloads {
                            let is_same_transaction =
                                removed_data.transaction.hashes.transaction_intent_hash
                                    == *transaction_intent_hash;
                            if is_same_transaction
                                && removed_data.source == MempoolAddSource::CoreApi
                            {
                                self.metrics
                                    .from_local_api_to_commit_wait
                                    .observe(removed_data.added_at.elapsed().as_secs_f64());
                            }
                        }
                    }
                    IntentHash::Subintent(subintent_hash) => {
                        // SAFETY: Take out both locks at once for atomicity.
                        // This take order is as per that of locks in `MempoolManager` RustDoc.
                        let mut cache = self.cache().write();
                        let mut mempool = self.mempool.write();

                        mempool.remove_by_subintent_hash(subintent_hash);
                        cache.handle_nullified_subintent(commit_time, committed, *subintent_hash);
                    }
                }
            }
        }
    }
}
