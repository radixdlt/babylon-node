use crate::scrypto_prelude::*;

use crate::{
    transaction::{CheckMetadata, StaticValidation},
    CommittedUserTransactionIdentifiers, MempoolAddRejection, StateVersion,
};

use lru::LruCache;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt,
    num::NonZeroUsize,
    ops::Add,
    time::{Duration, SystemTime},
};

pub type ExecutionRejectionReason = radix_engine::errors::RejectionReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolRejectionReason {
    AlreadyCommitted(AlreadyCommittedError),
    FromExecution(Box<ExecutionRejectionReason>),
    ValidationError(TransactionValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlreadyCommittedError {
    pub notarized_transaction_hash: NotarizedTransactionHash,
    pub committed_state_version: StateVersion,
    pub committed_notarized_transaction_hash: NotarizedTransactionHash,
}

impl From<TransactionValidationError> for MempoolRejectionReason {
    fn from(value: TransactionValidationError) -> Self {
        Self::ValidationError(value)
    }
}

impl MempoolRejectionReason {
    pub fn is_permanent_for_payload(&self) -> bool {
        self.permanence().is_permanent_for_payload()
    }

    pub fn is_permanent_for_intent(&self) -> bool {
        self.permanence().is_permanent_for_intent()
    }

    pub fn is_rejected_because_intent_already_committed(&self) -> bool {
        match self {
            MempoolRejectionReason::AlreadyCommitted(_) => true,
            MempoolRejectionReason::FromExecution(rejection_reason) => match **rejection_reason {
                ExecutionRejectionReason::SuccessButFeeLoanNotRepaid => false,
                ExecutionRejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(_) => false,
                ExecutionRejectionReason::TransactionEpochNotYetValid { .. } => false,
                ExecutionRejectionReason::TransactionEpochNoLongerValid { .. } => false,
                ExecutionRejectionReason::IntentHashPreviouslyCommitted => true,
                ExecutionRejectionReason::IntentHashPreviouslyCancelled => true,
            },
            MempoolRejectionReason::ValidationError(_) => false,
        }
    }

    pub fn already_committed_error(&self) -> Option<&AlreadyCommittedError> {
        match self {
            MempoolRejectionReason::AlreadyCommitted(error) => Some(error),
            _ => None,
        }
    }

    pub fn permanence(&self) -> RejectionPermanence {
        match self {
            MempoolRejectionReason::AlreadyCommitted(_) => {
                // This is permanent for the intent - because even other, non-committed transactions
                // of the same intent will fail with `ExecutionRejectionReason::IntentHashPreviouslyCommitted`
                RejectionPermanence::PermanentForAnyPayloadWithThisIntent
            }
            MempoolRejectionReason::FromExecution(rejection_error) => match **rejection_error {
                ExecutionRejectionReason::SuccessButFeeLoanNotRepaid => {
                    RejectionPermanence::Temporary {
                        retry: RetrySettings::AfterDelay {
                            base_delay: Duration::from_secs(2 * 60),
                        },
                    }
                }
                ExecutionRejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(_) => {
                    RejectionPermanence::Temporary {
                        retry: RetrySettings::AfterDelay {
                            base_delay: Duration::from_secs(2 * 60),
                        },
                    }
                }
                ExecutionRejectionReason::TransactionEpochNotYetValid { valid_from, .. } => {
                    RejectionPermanence::Temporary {
                        retry: RetrySettings::FromEpoch { epoch: valid_from },
                    }
                }
                ExecutionRejectionReason::TransactionEpochNoLongerValid { .. } => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
                ExecutionRejectionReason::IntentHashPreviouslyCommitted => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
                ExecutionRejectionReason::IntentHashPreviouslyCancelled => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
            },
            MempoolRejectionReason::ValidationError(validation_error) => match validation_error {
                // The size is a property of the payload, not the intent
                TransactionValidationError::TransactionTooLarge => {
                    RejectionPermanence::PermanentForPayload
                }
                // The serialization is a property of the payload, not the intent
                TransactionValidationError::PrepareError(_) => {
                    RejectionPermanence::PermanentForPayload
                }
                // The serialization is a property of the payload, not the intent
                TransactionValidationError::EncodeError(_) => {
                    RejectionPermanence::PermanentForPayload
                }
                // The signature validity is a property of the payload, not the intent
                TransactionValidationError::SignatureValidationError(_) => {
                    RejectionPermanence::PermanentForPayload
                }
                // This is permanent for the intent - because all intents share the same header
                TransactionValidationError::HeaderValidationError(_) => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
                // This is permanent for the intent - because all intents share the same manifest
                TransactionValidationError::IdValidationError(_) => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
                // This is permanent for the intent - because all intents share the same manifest
                TransactionValidationError::CallDataValidationError(_) => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
                // This is permanent for the intent - because all intents share the same manifest
                TransactionValidationError::InvalidMessage(_) => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisIntent
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionPermanence {
    PermanentForPayload,
    PermanentForAnyPayloadWithThisIntent,
    Temporary { retry: RetrySettings },
}

impl RejectionPermanence {
    pub fn is_permanent_for_payload(&self) -> bool {
        match self {
            RejectionPermanence::PermanentForPayload => true,
            RejectionPermanence::PermanentForAnyPayloadWithThisIntent => true,
            RejectionPermanence::Temporary { .. } => false,
        }
    }

    pub fn is_permanent_for_intent(&self) -> bool {
        match self {
            RejectionPermanence::PermanentForPayload => false,
            RejectionPermanence::PermanentForAnyPayloadWithThisIntent => true,
            RejectionPermanence::Temporary { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrySettings {
    AfterDelay { base_delay: Duration },
    FromEpoch { epoch: Epoch },
}

impl fmt::Display for MempoolRejectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MempoolRejectionReason::AlreadyCommitted(error) => {
                write!(f, "Already committed: {error:?}")
            }
            MempoolRejectionReason::FromExecution(rejection_error) => {
                write!(f, "{rejection_error}")
            }
            MempoolRejectionReason::ValidationError(validation_error) => {
                write!(f, "Validation Error: {validation_error:?}")
            }
        }
    }
}

/// This records details about the history of attempting to run the given pending transaction payload.
///
/// The aim is to steer the following decisions:
/// - Should we accept the payload (back) into the mempool?
/// - Should we include the payload in mempool sync responses?
/// - Should we drop the payload from our mempool?
/// - What information should we return from the status API
/// - Should we include the payload in proposals or is it too risky?
///
/// We separate `latest_rejection_against_committed_state` from `latest_rejection_against_prepared_state` so that
/// the API can distinguish permanent rejections from non-permanent rejections.
#[derive(Debug, Clone)]
pub struct PendingTransactionRecord {
    pub intent_hash: IntentHash,
    /// Only needs to be specified if the rejection isn't permanent
    pub intent_invalid_from_epoch: Option<Epoch>,
    pub latest_attempt: TransactionAttempt,
    pub earliest_permanent_rejection: Option<TransactionAttempt>,
    pub latest_rejection_against_committed_state: Option<TransactionAttempt>,
    pub latest_rejection_against_prepared_state: Option<TransactionAttempt>,
    pub retry_from: RetryFrom,
    pub non_rejection_count: u32,
    pub rejection_count: u32,
    pub first_tracked_timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAttempt {
    pub rejection: Option<MempoolRejectionReason>,
    pub against_state: AtState,
    pub timestamp: SystemTime,
}

impl TransactionAttempt {
    pub fn was_against_permanent_state(&self) -> bool {
        match self.against_state {
            AtState::Static => true,
            AtState::Committed { .. } => true,
            AtState::PendingPreparingVertices { .. } => false,
        }
    }

    pub fn marks_permanent_rejection_for_payload(&self) -> bool {
        if self.was_against_permanent_state() {
            if let Some(rejection_reason) = &self.rejection {
                return rejection_reason.is_permanent_for_payload();
            }
        }
        false
    }

    pub fn marks_permanent_rejection_for_intent(&self) -> bool {
        if self.was_against_permanent_state() {
            if let Some(rejection_reason) = &self.rejection {
                return rejection_reason.is_permanent_for_intent();
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtState {
    // We might need this to be versioned by protocol update later...
    Static,
    Committed {
        state_version: StateVersion,
    },
    PendingPreparingVertices {
        base_committed_state_version: StateVersion,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryFrom {
    Never,
    FromTime(SystemTime),
    FromEpoch(Epoch),
    Whenever,
}

impl PendingTransactionRecord {
    pub fn new(
        intent_hash: IntentHash,
        invalid_from_epoch: Option<Epoch>,
        attempt: TransactionAttempt,
    ) -> Self {
        let mut new_record = Self {
            intent_hash,
            intent_invalid_from_epoch: invalid_from_epoch,
            first_tracked_timestamp: attempt.timestamp,
            latest_attempt: attempt.clone(),
            // Fields which will be updated by the `update_record_details_for_attempt` method
            earliest_permanent_rejection: None,
            latest_rejection_against_committed_state: None,
            latest_rejection_against_prepared_state: None,
            retry_from: RetryFrom::Whenever,
            rejection_count: 0,
            non_rejection_count: 0,
        };

        new_record.update_record_details_for_attempt(attempt);

        new_record
    }

    pub fn track_attempt(&mut self, attempt: TransactionAttempt) {
        self.latest_attempt = attempt.clone();
        self.update_record_details_for_attempt(attempt);
    }

    fn update_record_details_for_attempt(&mut self, attempt: TransactionAttempt) {
        if self.earliest_permanent_rejection.is_none()
            && attempt.marks_permanent_rejection_for_payload()
        {
            self.earliest_permanent_rejection = Some(attempt.clone());
        }

        self.update_retry_from();

        match &attempt.rejection {
            None => {
                self.non_rejection_count += 1;
            }
            Some(_) => {
                self.rejection_count += 1;
                if attempt.was_against_permanent_state() {
                    self.latest_rejection_against_committed_state = Some(attempt);
                } else {
                    self.latest_rejection_against_prepared_state = Some(attempt);
                }
            }
        }
    }

    pub fn should_recalculate(&self, current_epoch: Epoch, current_timestamp: SystemTime) -> bool {
        match self.retry_from {
            RetryFrom::Never => false,
            RetryFrom::Whenever => true,
            RetryFrom::FromEpoch(retry_after_epoch) => retry_after_epoch <= current_epoch,
            RetryFrom::FromTime(retry_after_timestamp) => {
                retry_after_timestamp <= current_timestamp
            }
        }
    }

    /// Precondition:
    /// * If check = CheckMetadata::Cached, the latest attempt must be a rejection
    /// This precondition is met if the record/metadata come from a call using ForceRecalculation::IfCachedAsValid
    pub fn should_accept_into_mempool(
        self,
        check: CheckMetadata,
    ) -> Result<Box<ValidatedNotarizedTransactionV1>, MempoolAddRejection> {
        if let Some(permanent_rejection) = self.earliest_permanent_rejection {
            return Err(MempoolAddRejection {
                reason: permanent_rejection.rejection.unwrap(),
                against_state: permanent_rejection.against_state,
                retry_from: self.retry_from,
                was_cached: check.was_cached(),
                invalid_from_epoch: self.intent_invalid_from_epoch,
            });
        }
        if let Some(rejection_reason) = self.latest_attempt.rejection {
            // Regardless of whether it was a rejection against committed or prepared state,
            // let's block it from coming into our mempool for a while
            return Err(MempoolAddRejection {
                reason: rejection_reason,
                against_state: self.latest_attempt.against_state,
                retry_from: self.retry_from,
                was_cached: check.was_cached(),
                invalid_from_epoch: self.intent_invalid_from_epoch,
            });
        }
        match check {
            CheckMetadata::Cached => {
                panic!("Precondition was not met - the result was cached, but the latest attempt was not a rejection")
            }
            CheckMetadata::Fresh(StaticValidation::Valid(transaction)) => Ok(transaction),
            CheckMetadata::Fresh(StaticValidation::Invalid) => {
                panic!("A statically invalid transaction should already have been handled in the above")
            }
        }
    }

    pub fn most_applicable_status(&self) -> Option<&MempoolRejectionReason> {
        self.earliest_permanent_rejection
            .as_ref()
            .and_then(|r| r.rejection.as_ref())
            .or(self.latest_attempt.rejection.as_ref())
    }

    /// This should be called after permanent rejection is set but before the counts are updated
    fn update_retry_from(&mut self) {
        let attempt = &self.latest_attempt;
        let previous_rejection_count = self.rejection_count;
        let previous_non_rejection_count = self.non_rejection_count;

        if self.earliest_permanent_rejection.is_some() {
            self.retry_from = RetryFrom::Never;
            return;
        }

        let new_retry_from = match &attempt.rejection {
            Some(rejection_reason) => {
                match rejection_reason.permanence() {
                    RejectionPermanence::Temporary {
                        retry: RetrySettings::FromEpoch { epoch },
                    } => RetryFrom::FromEpoch(epoch),
                    RejectionPermanence::Temporary {
                        retry: RetrySettings::AfterDelay { base_delay },
                    } => {
                        // Use exponential back-off.
                        // Previous rejections increase the exponent, previous non-rejections decrease it by half as much
                        let base: f32 = 2.0;
                        let exponent: f32 = (previous_rejection_count as f32)
                            - ((previous_non_rejection_count as f32) / 2f32);
                        let multiplier: f32 = base.powf(exponent);

                        let delay = base_delay.mul_f32(multiplier).min(MAX_RECALCULATION_DELAY);

                        RetryFrom::FromTime(attempt.timestamp.add(delay))
                    }
                    _ => {
                        // If RejectionPermanence was Permanent, this has already been handled
                        return;
                    }
                }
            }
            None => {
                // Transaction was not rejected
                // Use a flat delay to check it's still not rejected again soon (eg to catch a fee-vault now being out of money)
                let delay = NON_REJECTION_RECALCULATION_DELAY;

                RetryFrom::FromTime(attempt.timestamp.add(delay))
            }
        };

        self.retry_from = new_retry_from;
    }
}

const NON_REJECTION_RECALCULATION_DELAY: Duration = Duration::from_secs(120);
const MAX_RECALCULATION_DELAY: Duration = Duration::from_secs(1000);

pub struct PendingTransactionResultCache {
    pending_transaction_records: LruCache<NotarizedTransactionHash, PendingTransactionRecord>,
    intent_lookup: HashMap<IntentHash, HashSet<NotarizedTransactionHash>>,
    recently_committed_intents: LruCache<IntentHash, CommittedIntentRecord>,
}

impl PendingTransactionResultCache {
    pub fn new(pending_txn_records_max_count: u32, committed_intents_max_size: u32) -> Self {
        PendingTransactionResultCache {
            pending_transaction_records: LruCache::new(
                NonZeroUsize::new(pending_txn_records_max_count as usize).unwrap(),
            ),
            intent_lookup: HashMap::new(),
            recently_committed_intents: LruCache::new(
                NonZeroUsize::new(committed_intents_max_size as usize).unwrap(),
            ),
        }
    }

    /// Note - the invalid_from_epoch only needs to be provided if the attempt is not a permanent rejection
    pub fn track_transaction_result(
        &mut self,
        intent_hash: IntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
        invalid_from_epoch: Option<Epoch>,
        attempt: TransactionAttempt,
    ) -> PendingTransactionRecord {
        let existing_record = self
            .pending_transaction_records
            .get_mut(&notarized_transaction_hash);

        if let Some(record) = existing_record {
            record.track_attempt(attempt);
            return record.clone();
        }

        let new = PendingTransactionRecord::new(intent_hash, invalid_from_epoch, attempt);

        // NB - removed is the item kicked out of the LRU cache if it's at capacity
        let removed = self
            .pending_transaction_records
            .push(notarized_transaction_hash, new.clone());

        self.handled_added(intent_hash, notarized_transaction_hash);
        if let Some((p, r)) = removed {
            self.handled_removed(p, r);
        }

        new
    }

    pub fn track_committed_transactions(
        &mut self,
        current_timestamp: SystemTime,
        committed_transactions: Vec<CommittedUserTransactionIdentifiers>,
    ) {
        for committed_transaction in committed_transactions {
            let committed_intent_hash = committed_transaction.intent_hash;
            let committed_notarized_transaction_hash =
                committed_transaction.notarized_transaction_hash;
            // Note - we keep the relevant statuses of all known payloads for the intent in the cache
            // so that we can still serve status responses for them - we just ensure we mark them as rejected
            self.recently_committed_intents.push(
                committed_intent_hash,
                CommittedIntentRecord {
                    state_version: committed_transaction.state_version,
                    notarized_transaction_hash: committed_notarized_transaction_hash,
                    timestamp: current_timestamp,
                },
            );

            if let Some(payload_hashes) = self.intent_lookup.get(&committed_intent_hash) {
                for cached_payload_hash in payload_hashes {
                    let record = self
                        .pending_transaction_records
                        .peek_mut(cached_payload_hash)
                        .expect("Intent lookup out of sync with rejected payloads");

                    // We even overwrite the record for transaction which got committed here
                    // because this is a cache for pending transactions, and it can't be re-committed
                    record.track_attempt(TransactionAttempt {
                        rejection: Some(MempoolRejectionReason::AlreadyCommitted(
                            AlreadyCommittedError {
                                notarized_transaction_hash: *cached_payload_hash,
                                committed_state_version: committed_transaction.state_version,
                                committed_notarized_transaction_hash,
                            },
                        )),
                        against_state: AtState::Committed {
                            state_version: committed_transaction.state_version,
                        },
                        timestamp: current_timestamp,
                    })
                }
            }
        }
    }

    pub fn get_pending_transaction_record(
        &mut self,
        intent_hash: &IntentHash,
        notarized_transaction_hash: &NotarizedTransactionHash,
    ) -> Option<PendingTransactionRecord> {
        if let Some(x) = self
            .pending_transaction_records
            .get(notarized_transaction_hash)
        {
            return Some(x.clone());
        }
        // We might not have a pending transaction record for this, but we know it has to be rejected due to the committed intent cache
        // So let's create and return a transient committed record for it
        if let Some(committed_intent_record) = self.recently_committed_intents.get(intent_hash) {
            return Some(PendingTransactionRecord::new(
                *intent_hash,
                None,
                TransactionAttempt {
                    rejection: Some(MempoolRejectionReason::AlreadyCommitted(
                        AlreadyCommittedError {
                            notarized_transaction_hash: *notarized_transaction_hash,
                            committed_state_version: committed_intent_record.state_version,
                            committed_notarized_transaction_hash: committed_intent_record
                                .notarized_transaction_hash,
                        },
                    )),
                    against_state: AtState::Committed {
                        state_version: committed_intent_record.state_version,
                    },
                    timestamp: committed_intent_record.timestamp,
                },
            ));
        }

        None
    }

    pub fn peek_all_known_payloads_for_intent(
        &self,
        intent_hash: &IntentHash,
    ) -> HashMap<NotarizedTransactionHash, PendingTransactionRecord> {
        match self.intent_lookup.get(intent_hash) {
            Some(payload_hashes) => payload_hashes
                .iter()
                .map(|payload_hash| {
                    let record = self
                        .pending_transaction_records
                        .peek(payload_hash)
                        .expect("Intent lookup out of sync with rejected payloads");
                    (*payload_hash, record.clone())
                })
                .collect::<HashMap<_, _>>(),
            None => HashMap::new(),
        }
    }

    fn handled_added(
        &mut self,
        intent_hash: IntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
    ) {
        // Add the intent hash <-> payload hash lookup
        match self.intent_lookup.entry(intent_hash) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(notarized_transaction_hash);
            }
            Entry::Vacant(e) => {
                e.insert(HashSet::from([notarized_transaction_hash]));
            }
        }
    }

    fn handled_removed(
        &mut self,
        notarized_transaction_hash: NotarizedTransactionHash,
        rejection_record: PendingTransactionRecord,
    ) {
        // Remove the intent hash <-> payload hash lookup
        let intent_hash = rejection_record.intent_hash;
        match self.intent_lookup.entry(intent_hash) {
            Entry::Occupied(e) if e.get().len() == 1 => {
                e.remove_entry();
            }
            Entry::Occupied(mut e) if e.get().len() > 1 => {
                e.get_mut().remove(&notarized_transaction_hash);
            }
            Entry::Occupied(_) => {
                // num_payload_hashes == 0
                panic!("Invalid intent_lookup state");
            }
            Entry::Vacant(_) => {
                panic!("Invalid intent_lookup state");
            }
        }
    }
}

struct CommittedIntentRecord {
    state_version: StateVersion,
    notarized_transaction_hash: NotarizedTransactionHash,
    timestamp: SystemTime,
}

#[cfg(test)]
mod tests {

    use super::*;

    fn user_payload_hash(nonce: u8) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from(blake2b_256_hash([0, nonce]))
    }

    fn intent_hash(nonce: u8) -> IntentHash {
        IntentHash::from(blake2b_256_hash([1, nonce]))
    }

    #[test]
    fn add_evict_and_peek_by_intent_test() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;

        let mut cache =
            PendingTransactionResultCache::new(rejection_limit, recently_committed_intents_limit);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);
        let payload_hash_3 = user_payload_hash(3);
        let payload_hash_4 = user_payload_hash(4);
        let payload_hash_5 = user_payload_hash(5);
        let payload_hash_6 = user_payload_hash(6);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);
        let intent_hash_3 = intent_hash(3);

        let example_attempt_1 = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::ValidationError(
                TransactionValidationError::TransactionTooLarge,
            )),
            against_state: AtState::Static,
            timestamp: SystemTime::now(),
        };

        let example_attempt_2 = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::FromExecution(Box::new(
                ExecutionRejectionReason::SuccessButFeeLoanNotRepaid,
            ))),
            against_state: AtState::Committed {
                state_version: StateVersion::pre_genesis(),
            },
            timestamp: SystemTime::now(),
        };

        // Start by adding 3 payloads against first intent hash. These all fit in, but cache is full
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_1,
            None,
            example_attempt_1.clone(),
        );
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_2,
            Some(Epoch::of(0)),
            example_attempt_2.clone(),
        );
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_3,
            None,
            example_attempt_1.clone(),
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .len(),
            3
        );

        // Now add another rejection - the first rejection (intent_1, payload_1, reason_1) should drop out
        cache.track_transaction_result(
            intent_hash_2,
            payload_hash_4,
            None,
            example_attempt_1.clone(),
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .len(),
            2
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .get(&payload_hash_2)
                .unwrap()
                .latest_attempt,
            example_attempt_2
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .get(&payload_hash_3)
                .unwrap()
                .latest_attempt,
            example_attempt_1
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_2)
                .len(),
            1
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_2)
                .get(&payload_hash_4)
                .unwrap()
                .latest_attempt,
            example_attempt_1
        );

        // Reading transaction status should jump payload 2 back to the top of the cache
        // So (intent_1, payload_3, reason_1) and (intent_2, payload_4, reason_1) should drop out instead
        cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_2);
        cache.track_transaction_result(
            intent_hash_3,
            payload_hash_5,
            None,
            example_attempt_1.clone(),
        );
        cache.track_transaction_result(intent_hash_3, payload_hash_6, None, example_attempt_1);
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .len(),
            1
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .get(&payload_hash_2)
                .unwrap()
                .latest_attempt,
            example_attempt_2
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_2)
                .len(),
            0
        );
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_3)
                .len(),
            2
        );
    }

    #[test]
    fn committed_transaction_checks() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;
        let now = SystemTime::now();

        let mut cache =
            PendingTransactionResultCache::new(rejection_limit, recently_committed_intents_limit);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);

        cache.track_committed_transactions(
            now,
            vec![CommittedUserTransactionIdentifiers {
                state_version: StateVersion::of(1),
                intent_hash: intent_hash_1,
                notarized_transaction_hash: payload_hash_1,
            }],
        );
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_1);
        assert!(record.is_some());

        let record = cache.get_pending_transaction_record(&intent_hash_2, &payload_hash_2);
        assert!(record.is_none());
    }

    #[test]
    fn successes_and_temporary_rejections_are_marked_as_should_recalculate_appropriately() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;

        let start = SystemTime::now();
        let far_in_future = start.add(Duration::from_secs(u32::MAX as u64));
        let little_in_future = start.add(Duration::from_secs(1));

        let mut cache =
            PendingTransactionResultCache::new(rejection_limit, recently_committed_intents_limit);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);
        let payload_hash_3 = user_payload_hash(3);
        let payload_hash_4 = user_payload_hash(4);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);

        let attempt_with_temporary_rejection = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::FromExecution(Box::new(
                ExecutionRejectionReason::SuccessButFeeLoanNotRepaid,
            ))),
            against_state: AtState::Committed {
                state_version: StateVersion::pre_genesis(),
            },
            timestamp: start,
        };
        let attempt_with_rejection_until_epoch_10 = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::FromExecution(Box::new(
                ExecutionRejectionReason::TransactionEpochNotYetValid {
                    valid_from: Epoch::of(10),
                    current_epoch: Epoch::of(9),
                },
            ))),
            against_state: AtState::Committed {
                state_version: StateVersion::of(10000),
            },
            timestamp: start,
        };
        let attempt_with_permanent_rejection = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::ValidationError(
                TransactionValidationError::TransactionTooLarge,
            )),
            against_state: AtState::Committed {
                state_version: StateVersion::pre_genesis(),
            },
            timestamp: start,
        };
        let attempt_with_no_rejection = TransactionAttempt {
            rejection: None,
            against_state: AtState::Committed {
                state_version: StateVersion::pre_genesis(),
            },
            timestamp: start,
        };

        // Permanent Rejection
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_1,
            None,
            attempt_with_permanent_rejection,
        );
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_1);
        // Even far in future, a permanent rejection is still there and never ready for recalculation
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(Epoch::of(0), far_in_future));

        // Temporary Rejection
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_2,
            Some(Epoch::of(50)),
            attempt_with_temporary_rejection,
        );
        // A little in future, a temporary rejection is not ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_2);
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(Epoch::of(0), little_in_future));

        // Far in future, a temporary rejection is ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_2);
        assert!(record.is_some());
        assert!(record
            .unwrap()
            .should_recalculate(Epoch::of(0), far_in_future));

        // No rejection
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_3,
            Some(Epoch::of(50)),
            attempt_with_no_rejection,
        );

        // A little in future, a no-rejection result is not ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_3);
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(Epoch::of(0), little_in_future));

        // Far in future, a no-rejection result is ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_3);
        assert!(record.is_some());
        assert!(record
            .unwrap()
            .should_recalculate(Epoch::of(0), far_in_future));

        // Temporary Rejection with recalculation from epoch 10
        cache.track_transaction_result(
            intent_hash_2,
            payload_hash_4,
            Some(Epoch::of(50)),
            attempt_with_rejection_until_epoch_10,
        );

        // Still at epoch 9, not yet ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_2, &payload_hash_4);
        let current_epoch = Epoch::of(9);
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(current_epoch, little_in_future));

        // Now at epoch 10, now ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_2, &payload_hash_4);
        let current_epoch = Epoch::of(10);
        assert!(record.is_some());
        assert!(record
            .unwrap()
            .should_recalculate(current_epoch, little_in_future));
    }
}
