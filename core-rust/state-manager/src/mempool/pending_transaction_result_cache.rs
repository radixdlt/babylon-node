use transaction::errors::TransactionValidationError;

use crate::{IntentHash, MempoolAddRejection, UserPayloadHash};

use lru::LruCache;
use radix_engine::errors::RejectionError;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt,
    num::NonZeroUsize,
    ops::Add,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    FromExecution(Box<RejectionError>),
    ValidationError(TransactionValidationError),
    IntentHashCommitted,
    /// This is temporary until we get better execution limits
    ExecutionTookTooLong {
        time_limit_ms: u32,
    },
}

impl RejectionReason {
    pub fn is_permanent_for_payload(&self) -> bool {
        self.permanence().is_permanent_for_payload()
    }

    pub fn is_permanent_for_intent(&self) -> bool {
        self.permanence().is_permanent_for_intent()
    }

    pub fn is_rejected_because_intent_already_committed(&self) -> bool {
        match self {
            RejectionReason::FromExecution(rejection_error) => match **rejection_error {
                RejectionError::SuccessButFeeLoanNotRepaid => false,
                RejectionError::ErrorBeforeFeeLoanRepaid(_) => false,
                RejectionError::TransactionEpochNotYetValid { .. } => false,
                RejectionError::TransactionEpochNoLongerValid { .. } => false,
                // I've left this match statement all explicitly false because in the future we'll have duplicate Intent Hash here,
                // and we'll need to mark it true and I want to catch it as a compile error when it's introduced
            },
            RejectionReason::IntentHashCommitted => true,
            _ => false,
        }
    }

    pub fn permanence(&self) -> RejectionPermanence {
        match self {
            RejectionReason::FromExecution(rejection_error) => match **rejection_error {
                RejectionError::SuccessButFeeLoanNotRepaid => RejectionPermanence::Temporary {
                    base_allow_retry_after: Duration::from_secs(2 * 60),
                },
                RejectionError::ErrorBeforeFeeLoanRepaid(_) => RejectionPermanence::Temporary {
                    base_allow_retry_after: Duration::from_secs(2 * 60),
                },
                RejectionError::TransactionEpochNotYetValid { .. } => {
                    RejectionPermanence::Temporary {
                        base_allow_retry_after: Duration::from_secs(2 * 60),
                    }
                }
                RejectionError::TransactionEpochNoLongerValid { .. } => {
                    RejectionPermanence::PermamentForAnyPayloadWithThisIntent
                }
            },
            RejectionReason::ValidationError(validation_error) => match validation_error {
                // The size is a property of the payload, not the intent
                TransactionValidationError::TransactionTooLarge => {
                    RejectionPermanence::PermamentForPayload
                }
                // The serialization is a property of the payload, not the intent
                TransactionValidationError::SerializationError(_) => {
                    RejectionPermanence::PermamentForPayload
                }
                // The serialization is a property of the payload, not the intent
                TransactionValidationError::DeserializationError(_) => {
                    RejectionPermanence::PermamentForPayload
                }
                // The signature validity is a property of the payload, not the intent
                TransactionValidationError::SignatureValidationError(_) => {
                    RejectionPermanence::PermamentForPayload
                }
                // This isn't actually possible to get on the node - but it would mark a permanent intent rejection
                TransactionValidationError::IntentHashRejected => {
                    RejectionPermanence::PermamentForAnyPayloadWithThisIntent
                }
                // This is permanent for the intent - because all intents share the same header
                TransactionValidationError::HeaderValidationError(_) => {
                    RejectionPermanence::PermamentForAnyPayloadWithThisIntent
                }
                // This is permanent for the intent - because all intents share the same manifest
                TransactionValidationError::IdValidationError(_) => {
                    RejectionPermanence::PermamentForAnyPayloadWithThisIntent
                }
                // This is permanent for the intent - because all intents share the same manifest
                TransactionValidationError::CallDataValidationError(_) => {
                    RejectionPermanence::PermamentForAnyPayloadWithThisIntent
                }
            },
            RejectionReason::IntentHashCommitted => {
                RejectionPermanence::PermamentForAnyPayloadWithThisIntent
            }
            // Temporary until we have better execution limits
            RejectionReason::ExecutionTookTooLong { .. } => RejectionPermanence::Temporary {
                base_allow_retry_after: Duration::from_secs(10 * 60),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionPermanence {
    PermamentForPayload,
    PermamentForAnyPayloadWithThisIntent,
    Temporary { base_allow_retry_after: Duration },
}

impl RejectionPermanence {
    pub fn is_permanent_for_payload(&self) -> bool {
        match self {
            RejectionPermanence::PermamentForPayload => true,
            RejectionPermanence::PermamentForAnyPayloadWithThisIntent => true,
            RejectionPermanence::Temporary { .. } => false,
        }
    }

    pub fn is_permanent_for_intent(&self) -> bool {
        match self {
            RejectionPermanence::PermamentForPayload => false,
            RejectionPermanence::PermamentForAnyPayloadWithThisIntent => true,
            RejectionPermanence::Temporary { .. } => false,
        }
    }
}

impl fmt::Display for RejectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RejectionReason::FromExecution(rejection_error) => write!(f, "{}", rejection_error),
            RejectionReason::ValidationError(validation_error) => {
                write!(f, "Validation Error: {:?}", validation_error)
            }
            RejectionReason::IntentHashCommitted => write!(f, "Intent hash already committed"),
            // Temporary until we have better execution limits
            RejectionReason::ExecutionTookTooLong { time_limit_ms } => write!(
                f,
                "Execution took longer than max time allowed: {}",
                time_limit_ms
            ),
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
    pub intent_invalid_from_epoch: u64,
    pub latest_attempt: TransactionAttempt,
    pub earliest_permanent_rejection: Option<TransactionAttempt>,
    pub latest_rejection_against_committed_state: Option<TransactionAttempt>,
    pub latest_rejection_against_prepared_state: Option<TransactionAttempt>,
    pub recalculation_due: RecalculationDue,
    pub non_rejection_count: u32,
    pub rejection_count: u32,
    pub first_tracked_timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAttempt {
    pub rejection: Option<RejectionReason>,
    pub against_state: AtState,
    pub timestamp: SystemTime,
}

impl TransactionAttempt {
    pub fn was_against_committed_state(&self) -> bool {
        match self.against_state {
            AtState::Committed { .. } => true,
            AtState::PendingPreparingVertices { .. } => false,
        }
    }

    pub fn marks_permanent_rejection_for_payload(&self) -> bool {
        if self.was_against_committed_state() {
            if let Some(rejection_reason) = &self.rejection {
                return rejection_reason.is_permanent_for_payload();
            }
        }
        false
    }

    pub fn marks_permanent_rejection_for_intent(&self) -> bool {
        if self.was_against_committed_state() {
            if let Some(rejection_reason) = &self.rejection {
                return rejection_reason.is_permanent_for_intent();
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtState {
    Committed { state_version: u64 },
    PendingPreparingVertices { base_committed_state_version: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecalculationDue {
    Never,
    From(SystemTime),
    Whenever,
}

impl PendingTransactionRecord {
    pub fn new(
        intent_hash: IntentHash,
        invalid_from_epoch: u64,
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
            recalculation_due: RecalculationDue::Whenever,
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

        self.update_recalculation_due();

        match &attempt.rejection {
            None => {
                self.non_rejection_count += 1;
            }
            Some(_) => {
                self.rejection_count += 1;
                if attempt.was_against_committed_state() {
                    self.latest_rejection_against_committed_state = Some(attempt);
                } else {
                    self.latest_rejection_against_prepared_state = Some(attempt);
                }
            }
        }
    }

    pub fn should_recalculate(&self, current_timestamp: SystemTime) -> bool {
        match self.recalculation_due {
            RecalculationDue::Never => false,
            RecalculationDue::Whenever => true,
            RecalculationDue::From(recalculate_after) => recalculate_after <= current_timestamp,
        }
    }

    pub fn should_accept_into_mempool(self, was_cached: bool) -> Result<(), MempoolAddRejection> {
        if let Some(permanent_rejection) = self.earliest_permanent_rejection {
            return Err(MempoolAddRejection {
                reason: permanent_rejection.rejection.unwrap(),
                against_state: permanent_rejection.against_state,
                recalculation_due: self.recalculation_due,
                was_cached,
                invalid_from_epoch: self.intent_invalid_from_epoch,
            });
        }
        if let Some(rejection_reason) = self.latest_attempt.rejection {
            // Regardless of whether it was a rejection against committed or prepared state,
            // let's block it from coming into our mempool for a while
            return Err(MempoolAddRejection {
                reason: rejection_reason,
                against_state: self.latest_attempt.against_state,
                recalculation_due: self.recalculation_due,
                was_cached,
                invalid_from_epoch: self.intent_invalid_from_epoch,
            });
        }
        Ok(())
    }

    pub fn most_applicable_status(&self) -> Option<&RejectionReason> {
        self.earliest_permanent_rejection
            .as_ref()
            .and_then(|r| r.rejection.as_ref())
            .or(self.latest_attempt.rejection.as_ref())
    }

    /// This should be called after permanent rejection is set but before the counts are updated
    fn update_recalculation_due(&mut self) {
        let attempt = &self.latest_attempt;
        let previous_rejection_count = self.rejection_count;
        let previous_non_rejection_count = self.non_rejection_count;

        if self.earliest_permanent_rejection.is_some() {
            self.recalculation_due = RecalculationDue::Never;
            return;
        }

        let new_recalculation_due = match &attempt.rejection {
            Some(rejection_reason) => {
                match rejection_reason.permanence() {
                    RejectionPermanence::Temporary {
                        base_allow_retry_after,
                    } => {
                        // Use exponential back-off.
                        // Previous rejections increase the exponent, previous non-rejections decrease it by half as much
                        let base: f32 = 2.0;
                        let exponent: f32 = (previous_rejection_count as f32)
                            - ((previous_non_rejection_count as f32) / 2f32);
                        let multiplier: f32 = base.powf(exponent);

                        let delay = base_allow_retry_after
                            .mul_f32(multiplier)
                            .min(MAX_RECALCULATION_DELAY);

                        RecalculationDue::From(attempt.timestamp.add(delay))
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

                RecalculationDue::From(attempt.timestamp.add(delay))
            }
        };

        self.recalculation_due = new_recalculation_due;
    }
}

const NON_REJECTION_RECALCULATION_DELAY: Duration = Duration::from_secs(120);
const MAX_RECALCULATION_DELAY: Duration = Duration::from_secs(1000);

pub struct PendingTransactionResultCache {
    pending_transaction_records: LruCache<UserPayloadHash, PendingTransactionRecord>,
    intent_lookup: HashMap<IntentHash, HashSet<UserPayloadHash>>,
    recently_committed_intents: LruCache<IntentHash, (u64, SystemTime)>,
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

    pub fn track_transaction_result(
        &mut self,
        intent_hash: IntentHash,
        payload_hash: UserPayloadHash,
        invalid_from_epoch: u64,
        attempt: TransactionAttempt,
    ) {
        let existing_record = self.pending_transaction_records.get_mut(&payload_hash);

        if let Some(record) = existing_record {
            record.track_attempt(attempt);
            return;
        }

        // NB - removed is the item kicked out of the LRU cache if it's at capacity
        let removed = self.pending_transaction_records.push(
            payload_hash,
            PendingTransactionRecord::new(intent_hash, invalid_from_epoch, attempt),
        );

        self.handled_added(intent_hash, payload_hash);
        if let Some((p, r)) = removed {
            self.handled_removed(p, r);
        }
    }

    pub fn track_committed_transactions(
        &mut self,
        current_timestamp: SystemTime,
        previous_state_version: u64,
        hashes: Vec<IntentHash>,
    ) {
        let mut resultant_state_version = previous_state_version;
        for intent_hash in hashes {
            // Note - we keep the relevant statuses of all known payloads for the intent in the cache
            // so that we can still serve status responses for them - we just ensure we mark them as rejected
            resultant_state_version += 1;

            self.recently_committed_intents
                .push(intent_hash, (resultant_state_version, current_timestamp));

            if let Some(payload_hashes) = self.intent_lookup.get(&intent_hash) {
                for cached_payload_hash in payload_hashes {
                    let record = self
                        .pending_transaction_records
                        .peek_mut(cached_payload_hash)
                        .expect("Intent lookup out of sync with rejected payloads");

                    // We even overwrite the record for transaction which got committed here
                    // because this is a cache for pending transactions, and it can't be re-committed
                    record.track_attempt(TransactionAttempt {
                        rejection: Some(RejectionReason::IntentHashCommitted),
                        against_state: AtState::Committed {
                            state_version: resultant_state_version,
                        },
                        timestamp: current_timestamp,
                    })
                }
            }
        }
    }

    pub fn get_pending_transaction_record<'a>(
        &'a mut self,
        intent_hash: &IntentHash,
        payload_hash: &UserPayloadHash,
        invalid_from_epoch: u64,
    ) -> Option<PendingTransactionRecord> {
        if let Some(x) = self.pending_transaction_records.get(payload_hash) {
            return Some(x.clone());
        }
        // We might not have a pending transaction record for this, but we know it has to be rejected due to the committed intent cache
        // So let's create and return a transient committed record for it
        if let Some((committed_at_state_version, timestamp)) =
            self.recently_committed_intents.get(intent_hash)
        {
            return Some(PendingTransactionRecord::new(
                *intent_hash,
                invalid_from_epoch,
                TransactionAttempt {
                    rejection: Some(RejectionReason::IntentHashCommitted),
                    against_state: AtState::Committed {
                        state_version: *committed_at_state_version,
                    },
                    timestamp: *timestamp,
                },
            ));
        }

        None
    }

    pub fn peek_all_known_payloads_for_intent(
        &self,
        intent_hash: &IntentHash,
    ) -> HashMap<UserPayloadHash, PendingTransactionRecord> {
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

    fn handled_added(&mut self, intent_hash: IntentHash, payload_hash: UserPayloadHash) {
        // Add the intent hash <-> payload hash lookup
        match self.intent_lookup.entry(intent_hash) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(payload_hash);
            }
            Entry::Vacant(e) => {
                e.insert(HashSet::from([payload_hash]));
            }
        }
    }

    fn handled_removed(
        &mut self,
        payload_hash: UserPayloadHash,
        rejection_record: PendingTransactionRecord,
    ) {
        // Remove the intent hash <-> payload hash lookup
        let intent_hash = rejection_record.intent_hash;
        match self.intent_lookup.entry(intent_hash) {
            Entry::Occupied(e) if e.get().len() == 1 => {
                e.remove_entry();
            }
            Entry::Occupied(mut e) if e.get().len() > 1 => {
                e.get_mut().remove(&payload_hash);
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

#[cfg(test)]
mod tests {
    use radix_engine::types::sha256_twice;

    use super::*;

    fn user_payload_hash(nonce: u8) -> UserPayloadHash {
        UserPayloadHash::from_raw_bytes(sha256_twice([0, nonce]).0)
    }

    fn intent_hash(nonce: u8) -> IntentHash {
        IntentHash::from_raw_bytes(sha256_twice([1, nonce]).0)
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
            rejection: Some(RejectionReason::ValidationError(
                TransactionValidationError::TransactionTooLarge,
            )),
            against_state: AtState::Committed { state_version: 0 },
            timestamp: SystemTime::now(),
        };

        let example_attempt_2 = TransactionAttempt {
            rejection: Some(RejectionReason::FromExecution(Box::new(
                RejectionError::SuccessButFeeLoanNotRepaid,
            ))),
            against_state: AtState::Committed { state_version: 0 },
            timestamp: SystemTime::now(),
        };

        // Start by adding 3 payloads against first intent hash. These all fit in, but cache is full
        cache.track_transaction_result(intent_hash_1, payload_hash_1, 0, example_attempt_1.clone());
        cache.track_transaction_result(intent_hash_1, payload_hash_2, 0, example_attempt_2.clone());
        cache.track_transaction_result(intent_hash_1, payload_hash_3, 0, example_attempt_1.clone());
        assert_eq!(
            cache
                .peek_all_known_payloads_for_intent(&intent_hash_1)
                .len(),
            3
        );

        // Now add another rejection - the first rejection (intent_1, payload_1, reason_1) should drop out
        cache.track_transaction_result(intent_hash_2, payload_hash_4, 0, example_attempt_1.clone());
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
        cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_2, 0);
        cache.track_transaction_result(intent_hash_3, payload_hash_5, 0, example_attempt_1.clone());
        cache.track_transaction_result(intent_hash_3, payload_hash_6, 0, example_attempt_1);
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

        cache.track_committed_transactions(now, 0, vec![intent_hash_1]);
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_1, 0);
        assert!(record.is_some());

        let record = cache.get_pending_transaction_record(&intent_hash_2, &payload_hash_2, 0);
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

        let intent_hash_1 = intent_hash(1);

        let attempt_with_temporary_rejection = TransactionAttempt {
            rejection: Some(RejectionReason::FromExecution(Box::new(
                RejectionError::SuccessButFeeLoanNotRepaid,
            ))),
            against_state: AtState::Committed { state_version: 0 },
            timestamp: start,
        };
        let attempt_with_permanent_rejection = TransactionAttempt {
            rejection: Some(RejectionReason::ValidationError(
                TransactionValidationError::TransactionTooLarge,
            )),
            against_state: AtState::Committed { state_version: 0 },
            timestamp: start,
        };
        let attempt_with_no_rejection = TransactionAttempt {
            rejection: None,
            against_state: AtState::Committed { state_version: 0 },
            timestamp: start,
        };

        // Permanent Rejection
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_1,
            0,
            attempt_with_permanent_rejection,
        );
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_1, 0);
        // Even far in future, a permanent rejection is still there and never ready for recalculation
        assert!(record.is_some());
        assert!(!record.unwrap().should_recalculate(far_in_future));

        // Temporary Rejection
        cache.track_transaction_result(
            intent_hash_1,
            payload_hash_2,
            0,
            attempt_with_temporary_rejection,
        );
        // A little in future, a temporary rejection is not ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_2, 0);
        assert!(record.is_some());
        assert!(!record.unwrap().should_recalculate(little_in_future));

        // Far in future, a temporary rejection is ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_2, 0);
        assert!(record.is_some());
        assert!(record.unwrap().should_recalculate(far_in_future));

        // No rejection
        cache.track_transaction_result(intent_hash_1, payload_hash_3, 0, attempt_with_no_rejection);

        // A little in future, a no-rejection result is not ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_3, 0);
        assert!(record.is_none());
        assert!(!record.unwrap().should_recalculate(little_in_future));

        // Far in future, a no-rejection result is ready for recalculation
        let record = cache.get_pending_transaction_record(&intent_hash_1, &payload_hash_3, 0);
        assert!(record.is_none());
        assert!(record.unwrap().should_recalculate(far_in_future));
    }
}
