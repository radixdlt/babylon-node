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

use node_common::config::MempoolConfig;
use node_common::metrics::TakesMetricLabels;
use prometheus::Registry;
use rand::seq::index::sample;
use tracing::warn;

use crate::{mempool::*, StateVersion};

use std::cmp::{max, min, Ordering};
use std::collections::{HashMap, HashSet};

use node_common::indexing::SecondaryIndex;
use std::ops::Index;
use std::sync::Arc;
use std::time::Instant;

use super::metrics::MempoolMetrics;

// Memory overhead of transactions living in the mempool. This does not take into account the
// (cached) results.
// Current implementation: for each transaction we keep both the raw transaction and the
// parsed one (2x overhead) plus a very generous 30% overhead for the indexes.
// Note: this value is needed in Java (at setup) and in order to circumvent the lack of
// f64 <-> double SBOR encoding, we keep it as an u32 percent.
pub const MEMPOOL_TRANSACTION_OVERHEAD_FACTOR_PERCENT: u32 = 230;

#[derive(Debug, Clone)]
pub struct MempoolData {
    /// The mempool transaction.
    /// The [`MempoolTransaction`] is stored in an [`Arc`] for performance, so it is cheap to clone
    /// it as part of mempool operations.
    pub transaction: Arc<MempoolTransaction>,
    /// The instant at which the transaction was added to the mempool.
    pub added_at: Instant,
    /// The source of the transaction.
    pub source: MempoolAddSource,
    /// The state version of this transaction's last successful execution.
    pub successfully_executed_at: StateVersion,
}

#[derive(Debug, Clone, Eq, PartialEq)] // (`Eq` for tests only)
pub struct MempoolTransaction {
    pub validated: Box<ValidatedNotarizedTransactionV1>,
    pub raw: RawNotarizedTransaction,
}

impl MempoolTransaction {
    pub fn tip_percentage(&self) -> u16 {
        self.validated
            .prepared
            .signed_intent
            .intent
            .header
            .inner
            .tip_percentage
    }

    pub fn end_epoch_exclusive(&self) -> Epoch {
        self.validated
            .prepared
            .signed_intent
            .intent
            .header
            .inner
            .end_epoch_exclusive
    }
}

impl HasIntentHash for MempoolTransaction {
    fn intent_hash(&self) -> IntentHash {
        self.validated.prepared.intent_hash()
    }
}

impl HasSignedIntentHash for MempoolTransaction {
    fn signed_intent_hash(&self) -> transaction::model::SignedIntentHash {
        self.validated.prepared.signed_intent_hash()
    }
}

impl HasNotarizedTransactionHash for MempoolTransaction {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        self.validated.prepared.notarized_transaction_hash()
    }
}

impl MempoolData {
    fn new(
        transaction: Arc<MempoolTransaction>,
        added_at: Instant,
        source: MempoolAddSource,
        successfully_executed_at: StateVersion,
    ) -> Self {
        Self {
            transaction,
            added_at,
            source,
            successfully_executed_at,
        }
    }
}

/// A transaction's proposal priority.
///
/// The greatest element (i.e. the one with the *highest* `tip_percentage`, and then the *oldest*
/// `added_at`) marks the "best" transaction (i.e. the one to be executed first).
#[derive(Clone, Eq, PartialEq)]
struct ProposalPriority {
    tip_percentage: u16,
    added_at: Instant,
}

impl Ord for ProposalPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip_percentage
            .cmp(&other.tip_percentage)
            // Note: the `reverse()` below is deliberate; this is why we cannot do `#[derive(Ord)]`
            .then_with(|| self.added_at.cmp(&other.added_at).reverse())
    }
}

impl PartialOrd for ProposalPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ProposalPriority {
    pub fn new(mempool_data: &MempoolData) -> Self {
        Self {
            tip_percentage: mempool_data.transaction.tip_percentage(),
            added_at: mempool_data.added_at,
        }
    }
}

/// An *exclusive* end epoch of a transaction's validity period.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct TransactionEndEpoch(Epoch);

impl TransactionEndEpoch {
    pub fn new(mempool_data: &MempoolData) -> Self {
        Self(mempool_data.transaction.end_epoch_exclusive())
    }
}

/// A state version on which a transaction was successfully executed.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ExecutionStateVersion(StateVersion);

impl ExecutionStateVersion {
    pub fn new(mempool_data: &MempoolData) -> Self {
        Self(mempool_data.successfully_executed_at)
    }
}

pub struct PriorityMempool {
    /// Max number of different (by [`NotarizedTransactionHash`]) transactions that can live at any moment of time in the mempool.
    remaining_transaction_count: u32,
    /// Max sum of transactions size that can live in [`self.data`].
    remaining_total_transactions_size: u64,
    /// Mapping from [`NotarizedTransactionHash`] to [`MempoolData`] containing [`MempoolTransaction`] with said payload hash.
    /// We use [`IndexMap`] for it's O(1) [`get_index`] needed for efficient random sampling.
    data: IndexMap<NotarizedTransactionHash, MempoolData>,
    /// Mapping from [`IntentHash`] to all transactions ([`NotarizedTransactionHash`]) that submit said intent.
    intent_lookup: HashMap<IntentHash, HashSet<NotarizedTransactionHash>>,
    /// Keeps ordering of the transactions by their [`ProposalPriority`].
    proposal_priority_index: SecondaryIndex<ProposalPriority, NotarizedTransactionHash>,
    /// Keeps ordering of the transactions by their validity end epoch.
    end_epoch_exclusive_index: SecondaryIndex<TransactionEndEpoch, NotarizedTransactionHash>,
    /// Keeps ordering of the transactions by the latest state version of their successful execution.
    executed_state_version_index: SecondaryIndex<ExecutionStateVersion, NotarizedTransactionHash>,
    /// Various metrics.
    metrics: MempoolMetrics,
}

impl PriorityMempool {
    pub fn new(config: MempoolConfig, metric_registry: &Registry) -> PriorityMempool {
        PriorityMempool {
            remaining_transaction_count: config.max_transaction_count,
            remaining_total_transactions_size: config.max_total_transactions_size,
            data: IndexMap::new(),
            intent_lookup: HashMap::new(),
            proposal_priority_index: SecondaryIndex::new(),
            end_epoch_exclusive_index: SecondaryIndex::new(),
            executed_state_version_index: SecondaryIndex::new(),
            metrics: MempoolMetrics::new(metric_registry),
        }
    }
}

impl PriorityMempool {
    /// Tries to add a new transaction into the mempool.
    /// Will return either a [`Vec`] of [`MempoolData`] that was evicted in order to fit the new transaction or an error
    /// if the mempool is full and the new transaction proposal priority is not better than what already exists.
    /// Returns an empty [`Vec`] if the transaction was already present in the mempool.
    pub fn add_transaction_if_not_present(
        &mut self,
        transaction: Arc<MempoolTransaction>,
        source: MempoolAddSource,
        added_at: Instant,
        executed_at: StateVersion,
    ) -> Result<Vec<MempoolData>, MempoolAddError> {
        let notarized_transaction_hash = transaction.notarized_transaction_hash();

        if self.contains_transaction(&notarized_transaction_hash) {
            return Ok(vec![]);
        }

        let intent_hash = transaction.intent_hash();
        let transaction_size = transaction.raw.0.len() as u64;

        let transaction_data = MempoolData::new(transaction, added_at, source, executed_at);
        let new_proposal_priority = ProposalPriority::new(&transaction_data);

        let mut total_transaction_size_free_space = self.remaining_total_transactions_size;
        let mut total_transaction_count_free_space = self.remaining_transaction_count;
        let mut to_be_removed = Vec::new();
        let mut priority_iter = self
            .proposal_priority_index
            .iter_values_from_least()
            .map(|hash| self.data.index(hash));
        // Collect the lowest priority transactions that are required to be evicted in order to add the new one.
        // Note: worst-case scenario is the biggest transaction that will be rejected against a mempool full of smallest
        // possible transactions. This can be mitigated with a dynamic segment tree which can do fast range sum queries,
        // in order to check rejection before actually getting the evicted transactions. With a minimum transaction of
        // 1024 bytes and current max transaction size of 1MB this should not be a problem yet.
        while total_transaction_size_free_space < transaction_size
            || total_transaction_count_free_space < 1
        {
            let lowest_priority_transaction = priority_iter.next();
            match lowest_priority_transaction {
                None => {
                    // Even with an empty mempool we are not able to fulfill the request.
                    warn!("Impossible to add new transaction. Mempool max size lower than transaction size!");
                    return Err(MempoolAddError::PriorityThresholdNotMet {
                        min_tip_percentage_required: None,
                        tip_percentage: transaction_data.transaction.tip_percentage(),
                    });
                }
                Some(mempool_data) => {
                    total_transaction_size_free_space +=
                        mempool_data.transaction.raw.0.len() as u64;
                    total_transaction_count_free_space += 1;
                    to_be_removed.push(mempool_data.clone());
                }
            }
        }
        drop(priority_iter);

        // Check the new transaction is better than all to be removed transactions.
        if !to_be_removed.is_empty() {
            let best_to_be_removed = to_be_removed.last().unwrap();
            if new_proposal_priority < ProposalPriority::new(best_to_be_removed) {
                let min_tip_percentage_required = best_to_be_removed
                    .transaction
                    .tip_percentage()
                    .checked_add(1);
                return Err(MempoolAddError::PriorityThresholdNotMet {
                    min_tip_percentage_required,
                    tip_percentage: transaction_data.transaction.tip_percentage(),
                });
            }
        }

        // Make room for new transaction
        for data in to_be_removed.iter() {
            self.remove_data(data.clone());
        }

        // Update remaining resources
        self.remaining_total_transactions_size -= transaction_size;
        self.remaining_transaction_count -= 1;

        // Update metrics as well
        self.metrics
            .current_total_transactions_size
            .add(transaction_size as i64);
        self.metrics.current_transactions.add(1);
        self.metrics.submission_added.with_label(source).inc();

        // Add new MempoolData
        if self
            .data
            .insert(notarized_transaction_hash, transaction_data.clone())
            .is_some()
        {
            // This should have been checked at the beginning of this method
            panic!("Broken precondition: Transaction already inside mempool.");
        }

        self.proposal_priority_index.insert_unique(
            ProposalPriority::new(&transaction_data),
            notarized_transaction_hash,
        );
        self.end_epoch_exclusive_index.insert_unique(
            TransactionEndEpoch::new(&transaction_data),
            notarized_transaction_hash,
        );
        self.executed_state_version_index.insert_unique(
            ExecutionStateVersion::new(&transaction_data),
            notarized_transaction_hash,
        );

        // Add intent lookup
        self.intent_lookup
            .entry(intent_hash)
            .and_modify(|e| {
                e.insert(notarized_transaction_hash);
            })
            .or_insert_with(|| HashSet::from([notarized_transaction_hash]));

        Ok(to_be_removed)
    }

    pub fn contains_transaction(
        &self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> bool {
        self.data.contains_key(notarized_transaction_hash)
    }

    pub fn observe_pending_execution_result(
        &mut self,
        notarized_transaction_hash: &NotarizedTransactionHash,
        attempt: &TransactionAttempt,
    ) {
        if attempt.rejection.is_some() {
            self.remove_by_notarized_transaction_hash(notarized_transaction_hash);
        } else if let AtState::Specific(specific_state) = &attempt.against_state {
            self.update_transaction_executed_state_version(
                notarized_transaction_hash,
                specific_state.committed_version(),
            );
        }
    }

    pub fn remove_by_intent_hash(&mut self, intent_hash: &IntentHash) -> Vec<MempoolData> {
        let data: Vec<_> = self
            .intent_lookup
            .get(intent_hash)
            .iter()
            .flat_map(|notarized_transaction_hashes| notarized_transaction_hashes.iter())
            .map(|notarized_transaction_hash| self.data.index(notarized_transaction_hash))
            .cloned()
            .collect();
        data.into_iter()
            .map(|data| {
                self.remove_data(data.clone());
                data
            })
            .collect()
    }

    pub fn remove_txns_where_end_epoch_expired(&mut self, epoch: Epoch) -> Vec<MempoolData> {
        let mempool_data = self.get_txns_where_end_epoch_expired(epoch);
        mempool_data
            .iter()
            .for_each(|data| self.remove_data(data.clone()));
        mempool_data
    }

    pub fn get_txns_where_end_epoch_expired(&self, epoch: Epoch) -> Vec<MempoolData> {
        self.end_epoch_exclusive_index
            .iter_values_from_least()
            .map(|hash| self.data.index(hash))
            .take_while(|mempool_data| mempool_data.transaction.end_epoch_exclusive() < epoch)
            .cloned()
            .collect()
    }

    pub fn get_count(&self) -> usize {
        self.data.len()
    }

    pub fn get_notarized_transaction_hashes_for_intent(
        &self,
        intent_hash: &IntentHash,
    ) -> Vec<NotarizedTransactionHash> {
        match self.intent_lookup.get(intent_hash) {
            Some(notarized_transaction_hashes) => {
                notarized_transaction_hashes.iter().cloned().collect()
            }
            None => vec![],
        }
    }

    pub fn all_hashes_iter(
        &self,
    ) -> impl Iterator<Item = (&IntentHash, &NotarizedTransactionHash)> {
        self.intent_lookup
            .iter()
            .flat_map(|(intent_hash, notarized_transaction_hashes)| {
                notarized_transaction_hashes
                    .iter()
                    .map(move |notarized_transaction_hash| {
                        (intent_hash, notarized_transaction_hash)
                    })
            })
    }

    pub fn iter_by_state_version(&self) -> impl Iterator<Item = MempoolData> + '_ {
        self.executed_state_version_index
            .iter_values_from_least()
            .map(|hash| self.data.get(hash).unwrap().clone())
    }

    pub fn get_payload(
        &self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<&MempoolTransaction> {
        Some(&self.data.get(notarized_transaction_hash)?.transaction)
    }

    /// Returns [`count`] randomly sampled transactions from the mempool.
    /// If count is higher than the mempool size, all transaction are returned (in random order).
    /// Complexity is given by [`sample`] which is usually O(count).
    pub fn get_k_random_transactions(&self, count: usize) -> Vec<Arc<MempoolTransaction>> {
        sample(
            &mut rand::thread_rng(),
            self.data.len(),
            min(count, self.data.len()),
        )
        .into_iter()
        .map(|index| self.data.get_index(index).unwrap().1.transaction.clone())
        .collect()
    }

    /// Picks an subset of transactions to form the proposal.
    /// Transactions are picked in the order of [`proposal_priority_index`].
    /// Obeys the given count/size limits and explicit exclusions.
    pub fn get_proposal_transactions(
        &self,
        max_count: usize,
        max_payload_size_bytes: u64,
        notarized_transaction_hashes_to_exclude: &HashSet<NotarizedTransactionHash>,
    ) -> Vec<Arc<MempoolTransaction>> {
        const MAX_TRANSACTIONS_TO_TRY: usize = 1000;
        let max_transactions_to_try = max(max_count, MAX_TRANSACTIONS_TO_TRY);

        let mut payload_size_so_far = 0;
        self.proposal_priority_index
            .iter_values_from_greatest()
            .filter(|hash| !notarized_transaction_hashes_to_exclude.contains(*hash))
            .take(max_transactions_to_try)
            .map(|hash| self.data.index(hash).transaction.clone())
            .filter(|transaction| {
                let increased_payload_size = payload_size_so_far + transaction.raw.0.len() as u64;
                let fits = increased_payload_size <= max_payload_size_bytes;
                if fits {
                    payload_size_so_far = increased_payload_size;
                }
                fits
            })
            .take(max_count)
            .collect()
    }

    // Only internals below:

    /// Removes the given item from the mempool's storage (and all indices).
    ///
    /// Note: assumes that the given transaction is currently in the mempool.
    fn remove_data(&mut self, data: MempoolData) {
        let notarized_transaction_hash = &data.transaction.notarized_transaction_hash();
        let intent_hash = &data.transaction.intent_hash();

        let transaction_size = data.transaction.raw.0.len();
        self.remaining_transaction_count += 1;
        self.remaining_total_transactions_size += transaction_size as u64;

        // Update metrics
        self.metrics
            .current_total_transactions_size
            .sub(transaction_size as i64);
        self.metrics.current_transactions.sub(1);

        self.data.remove(notarized_transaction_hash);

        // Update intent_lookup
        let payload_lookup = self
            .intent_lookup
            .get_mut(intent_hash)
            .expect("Mempool intent hash lookup out of sync on remove");

        if !payload_lookup.remove(notarized_transaction_hash) {
            panic!("Mempool intent hash lookup out of sync on remove");
        }
        if payload_lookup.is_empty() {
            self.intent_lookup.remove(intent_hash);
        }

        self.proposal_priority_index
            .remove_existing(ProposalPriority::new(&data), *notarized_transaction_hash);
        self.end_epoch_exclusive_index
            .remove_existing(TransactionEndEpoch::new(&data), *notarized_transaction_hash);
        self.executed_state_version_index.remove_existing(
            ExecutionStateVersion::new(&data),
            *notarized_transaction_hash,
        );
    }

    fn update_transaction_executed_state_version(
        &mut self,
        notarized_transaction_hash: &NotarizedTransactionHash,
        state_version: StateVersion,
    ) {
        if let Some(data) = self.data.get_mut(notarized_transaction_hash) {
            self.executed_state_version_index.remove_existing(
                ExecutionStateVersion::new(data),
                *notarized_transaction_hash,
            );
            data.successfully_executed_at = state_version;
            self.executed_state_version_index.insert_unique(
                ExecutionStateVersion::new(data),
                *notarized_transaction_hash,
            );
        }
    }

    fn remove_by_notarized_transaction_hash(
        &mut self,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<MempoolData> {
        let to_remove = self.data.get(notarized_transaction_hash).cloned();
        match &to_remove {
            None => {}
            Some(data) => self.remove_data(data.clone()),
        }
        to_remove
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::mempool::priority_mempool::*;

    fn create_fake_pub_key() -> PublicKey {
        PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]))
    }

    fn create_fake_signature() -> NotarySignatureV1 {
        NotarySignatureV1(SignatureV1::Secp256k1(Secp256k1Signature(
            [0; Secp256k1Signature::LENGTH],
        )))
    }

    fn create_fake_signature_with_public_key() -> IntentSignatureV1 {
        IntentSignatureV1(SignatureWithPublicKeyV1::Secp256k1 {
            signature: Secp256k1Signature([0; Secp256k1Signature::LENGTH]),
        })
    }

    fn create_fake_notarized_transaction(
        nonce: u32,
        sigs_count: usize,
        tip_percentage: u16,
    ) -> PreparedNotarizedTransactionV1 {
        NotarizedTransactionV1 {
            signed_intent: SignedIntentV1 {
                intent: IntentV1 {
                    header: TransactionHeaderV1 {
                        network_id: 1,
                        start_epoch_inclusive: Epoch::of(1),
                        end_epoch_exclusive: Epoch::of(2),
                        nonce,
                        notary_public_key: create_fake_pub_key(),
                        notary_is_signatory: false,
                        tip_percentage,
                    },
                    instructions: InstructionsV1(vec![]),
                    blobs: BlobsV1 { blobs: vec![] },
                    message: MessageV1::None,
                },
                intent_signatures: IntentSignaturesV1 {
                    signatures: vec![0; sigs_count]
                        .into_iter()
                        .map(|_| create_fake_signature_with_public_key())
                        .collect(),
                },
            },
            notary_signature: create_fake_signature(),
        }
        .prepare()
        .expect("Expected that it could be prepared")
    }

    fn create_fake_pending_transaction(
        nonce: u32,
        sigs_count: usize,
        tip_percentage: u16,
    ) -> Arc<MempoolTransaction> {
        Arc::new(MempoolTransaction {
            validated: Box::new(ValidatedNotarizedTransactionV1 {
                prepared: create_fake_notarized_transaction(nonce, sigs_count, tip_percentage),
                // Fake these
                encoded_instructions: vec![],
                signer_keys: vec![],
                num_of_signature_validations: 0,
            }),
            raw: RawNotarizedTransaction(vec![]),
        })
    }

    #[test]
    fn add_and_get_test() {
        let mt1 = create_fake_pending_transaction(1, 0, 0);
        let mt2 = create_fake_pending_transaction(2, 0, 0);
        let mt3 = create_fake_pending_transaction(3, 0, 0);
        let v1 = StateVersion::of(1);

        let registry = Registry::new();

        let mut mp = PriorityMempool::new(
            MempoolConfig {
                max_transaction_count: 5,
                max_total_transactions_size: 2 * 1024 * 1024,
            },
            &registry,
        );
        assert_eq!(mp.remaining_transaction_count, 5);
        assert_eq!(mp.get_count(), 0);

        mp.add_transaction_if_not_present(
            mt1.clone(),
            MempoolAddSource::CoreApi,
            Instant::now(),
            v1,
        )
        .unwrap();
        assert_eq!(mp.remaining_transaction_count, 4);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.contains_transaction(&mt1.notarized_transaction_hash()));

        mp.add_transaction_if_not_present(
            mt2.clone(),
            MempoolAddSource::MempoolSync,
            Instant::now(),
            v1,
        )
        .unwrap();
        assert_eq!(mp.remaining_transaction_count, 3);
        assert_eq!(mp.get_count(), 2);
        assert!(mp.contains_transaction(&mt1.notarized_transaction_hash()));
        assert!(mp.contains_transaction(&mt2.notarized_transaction_hash()));

        let rem = mp.remove_by_intent_hash(&mt1.intent_hash());
        assert!(rem.iter().any(|d| d.transaction == mt1));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains_key(&mt2.notarized_transaction_hash()));
        assert!(!mp.data.contains_key(&mt1.notarized_transaction_hash()));

        let rem = mp.remove_by_intent_hash(&mt2.intent_hash());
        assert!(rem.iter().any(|d| d.transaction == mt2));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 0);
        assert!(!mp.data.contains_key(&mt2.notarized_transaction_hash()));
        assert!(!mp.data.contains_key(&mt1.notarized_transaction_hash()));

        // mempool is empty. Should return no transactions.
        assert!(mp.remove_by_intent_hash(&mt3.intent_hash()).is_empty());
        assert!(mp.remove_by_intent_hash(&mt2.intent_hash()).is_empty());
        assert!(mp.remove_by_intent_hash(&mt1.intent_hash()).is_empty());
    }

    #[test]
    fn test_intent_lookup() {
        let intent_1_payload_1 = create_fake_pending_transaction(1, 1, 0);
        let intent_1_payload_2 = create_fake_pending_transaction(1, 2, 0);
        let intent_1_payload_3 = create_fake_pending_transaction(1, 3, 0);
        let intent_2_payload_1 = create_fake_pending_transaction(2, 1, 0);
        let intent_2_payload_2 = create_fake_pending_transaction(2, 2, 0);
        let v1 = StateVersion::of(1);

        let registry = Registry::new();

        let mut mp = PriorityMempool::new(
            MempoolConfig {
                max_transaction_count: 10,
                max_total_transactions_size: 2 * 1024 * 1024,
            },
            &registry,
        );
        assert!(mp
            .add_transaction_if_not_present(
                intent_1_payload_1.clone(),
                MempoolAddSource::CoreApi,
                Instant::now(),
                v1
            )
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction_if_not_present(
                intent_1_payload_2.clone(),
                MempoolAddSource::CoreApi,
                Instant::now(),
                v1
            )
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction_if_not_present(
                intent_1_payload_3,
                MempoolAddSource::MempoolSync,
                Instant::now(),
                v1
            )
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction_if_not_present(
                intent_2_payload_1.clone(),
                MempoolAddSource::CoreApi,
                Instant::now(),
                v1
            )
            .unwrap()
            .is_empty());

        assert_eq!(mp.get_count(), 4);
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_2_payload_1.intent_hash())
                .len(),
            1
        );
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_1_payload_1.intent_hash())
                .len(),
            3
        );
        mp.remove_by_notarized_transaction_hash(&intent_1_payload_2.notarized_transaction_hash());
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_1_payload_1.intent_hash())
                .len(),
            2
        );
        let removed_data = mp
            .remove_by_notarized_transaction_hash(&intent_2_payload_2.notarized_transaction_hash());
        assert!(removed_data.is_none());
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_2_payload_2.intent_hash())
                .len(),
            1
        );
        let removed_data = mp
            .remove_by_notarized_transaction_hash(&intent_2_payload_1.notarized_transaction_hash());
        assert!(removed_data.is_some());
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_2_payload_2.intent_hash())
                .len(),
            0
        );
        assert!(mp
            .add_transaction_if_not_present(
                intent_2_payload_1,
                MempoolAddSource::MempoolSync,
                Instant::now(),
                v1
            )
            .unwrap()
            .is_empty());

        mp.remove_by_intent_hash(&intent_1_payload_2.intent_hash());
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_1_payload_1.intent_hash())
                .len(),
            0
        );

        assert!(mp
            .add_transaction_if_not_present(
                intent_2_payload_2.clone(),
                MempoolAddSource::CoreApi,
                Instant::now(),
                v1
            )
            .unwrap()
            .is_empty());
        assert_eq!(
            mp.get_notarized_transaction_hashes_for_intent(&intent_2_payload_2.intent_hash())
                .len(),
            2
        );
        mp.remove_by_intent_hash(&intent_2_payload_2.intent_hash());
        assert_eq!(mp.get_count(), 0);
    }

    #[test]
    fn test_proposal_priority_ordering() {
        let mt1 = create_fake_pending_transaction(1, 0, 10);
        let mt2 = create_fake_pending_transaction(2, 0, 20);

        let now = Instant::now();
        let time_point = [now + Duration::from_secs(1), now + Duration::from_secs(2)];

        let md1 = MempoolData::new(
            mt1.clone(),
            time_point[0],
            MempoolAddSource::CoreApi,
            StateVersion::of(3),
        );

        let md2 = MempoolData::new(
            mt1,
            time_point[1],
            MempoolAddSource::CoreApi,
            StateVersion::of(3),
        );

        // For same tip_percentage, earliest seen transaction is prioritized.
        assert!(ProposalPriority::new(&md1) > ProposalPriority::new(&md2));

        let md3 = MempoolData::new(
            mt2,
            time_point[0],
            MempoolAddSource::CoreApi,
            StateVersion::of(3),
        );

        // Highest tip percentage is always prioritized.
        assert!(ProposalPriority::new(&md3) > ProposalPriority::new(&md1));
        assert!(ProposalPriority::new(&md3) > ProposalPriority::new(&md2));
    }

    #[test]
    fn test_proposal_priority_add_eviction() {
        let mt1 = create_fake_pending_transaction(1, 0, 10);
        let mt2 = create_fake_pending_transaction(1, 0, 20);
        let mt3 = create_fake_pending_transaction(2, 0, 20);
        let mt4 = create_fake_pending_transaction(1, 0, 30);
        let mt5 = create_fake_pending_transaction(1, 0, 40);
        let mt6 = create_fake_pending_transaction(2, 0, 40);
        let mt7 = create_fake_pending_transaction(3, 0, 40);
        let mt8 = create_fake_pending_transaction(4, 0, 40);
        let mt9 = create_fake_pending_transaction(5, 0, 40);

        let now = Instant::now();
        let v1 = StateVersion::of(1);
        let time_point = [
            now + Duration::from_secs(1),
            now + Duration::from_secs(2),
            now + Duration::from_secs(3),
        ];

        let registry = Registry::new();

        let mut mp = PriorityMempool::new(
            MempoolConfig {
                max_transaction_count: 4,
                max_total_transactions_size: 2 * 1024 * 1024,
            },
            &registry,
        );

        assert!(mp
            .add_transaction_if_not_present(
                mt4.clone(),
                MempoolAddSource::CoreApi,
                time_point[0],
                v1
            )
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction_if_not_present(
                mt2.clone(),
                MempoolAddSource::CoreApi,
                time_point[1],
                v1
            )
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction_if_not_present(
                mt3.clone(),
                MempoolAddSource::MempoolSync,
                time_point[0],
                v1
            )
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction_if_not_present(
                mt1.clone(),
                MempoolAddSource::CoreApi,
                time_point[0],
                v1
            )
            .unwrap()
            .is_empty());

        let evicted = mp
            .add_transaction_if_not_present(mt5, MempoolAddSource::CoreApi, time_point[1], v1)
            .unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt1);

        // mt2 should be evicted before mt3 because of lower time spent in the mempool
        let evicted = mp
            .add_transaction_if_not_present(mt6, MempoolAddSource::CoreApi, time_point[1], v1)
            .unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt2);

        let evicted = mp
            .add_transaction_if_not_present(mt7, MempoolAddSource::CoreApi, time_point[1], v1)
            .unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt3);

        let evicted = mp
            .add_transaction_if_not_present(mt8, MempoolAddSource::CoreApi, time_point[1], v1)
            .unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt4);

        assert!(mp
            .add_transaction_if_not_present(mt9, MempoolAddSource::CoreApi, time_point[2], v1)
            .is_err());
    }

    #[test]
    fn test_duplicate_txn_not_inserted() {
        let mempool_txn = create_fake_pending_transaction(1, 0, 10);

        let now = Instant::now();
        let v1 = StateVersion::of(1);

        let registry = Registry::new();

        let mut mempool = PriorityMempool::new(
            MempoolConfig {
                max_transaction_count: 1,
                max_total_transactions_size: 1024 * 1024,
            },
            &registry,
        );

        // Inserting the same transaction twice should be a non-panicking no-op
        assert!(mempool
            .add_transaction_if_not_present(mempool_txn.clone(), MempoolAddSource::CoreApi, now, v1)
            .unwrap()
            .is_empty());
        assert!(mempool
            .add_transaction_if_not_present(
                mempool_txn.clone(),
                MempoolAddSource::MempoolSync,
                now + Duration::from_secs(1),
                v1
            )
            .unwrap()
            .is_empty());
    }
}
