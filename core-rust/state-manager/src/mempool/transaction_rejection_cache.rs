use radix_engine::engine::RejectionError;
use transaction::errors::{HeaderValidationError, TransactionValidationError};

use crate::{IntentHash, UserPayloadHash};

use lru::LruCache;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    FromExecution(Box<RejectionError>),
    ValidationError(TransactionValidationError),
    IntentHashCommitted,
}

impl RejectionReason {
    pub fn is_permanent(&self) -> bool {
        match self {
            RejectionReason::FromExecution(_) => false,
            // TODO - need to distinguish between epoch too low and epoch too high
            RejectionReason::ValidationError(
                TransactionValidationError::HeaderValidationError(
                    HeaderValidationError::OutOfEpochRange,
                ),
            ) => false,
            RejectionReason::ValidationError(_) => true,
            RejectionReason::IntentHashCommitted => true,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct RejectionRecord {
    pub intent_hash: IntentHash,
    pub reason: RejectionReason,
    pub tracked_timestamp: Instant,
}

pub struct RejectionCache {
    rejected_payloads: LruCache<UserPayloadHash, RejectionRecord>,
    intent_lookup: HashMap<IntentHash, HashSet<UserPayloadHash>>,
    recently_committed_intents: LruCache<IntentHash, ()>,
    max_time_to_live_for_temporary_rejections: Duration,
}

impl RejectionCache {
    pub fn new(
        rejected_payloads_max_size: u32,
        committed_intents_max_size: u32,
        max_time_to_live_for_temporary_rejections: Duration,
    ) -> Self {
        RejectionCache {
            rejected_payloads: LruCache::new(
                NonZeroUsize::new(rejected_payloads_max_size as usize).unwrap(),
            ),
            intent_lookup: HashMap::new(),
            recently_committed_intents: LruCache::new(
                NonZeroUsize::new(committed_intents_max_size as usize).unwrap(),
            ),
            max_time_to_live_for_temporary_rejections,
        }
    }

    pub fn track_rejection(
        &mut self,
        intent_hash: IntentHash,
        payload_hash: UserPayloadHash,
        reason: RejectionReason,
    ) {
        let removed = self.rejected_payloads.push(
            payload_hash,
            RejectionRecord {
                reason,
                intent_hash,
                tracked_timestamp: Instant::now(),
            },
        );
        self.handled_added(intent_hash, payload_hash);
        if let Some((p, r)) = removed {
            self.handled_removed(p, r)
        }
    }

    pub fn track_committed_transactions(&mut self, intent_hashes: Vec<IntentHash>) {
        for intent_hash in intent_hashes {
            // Note - we keep the relevant rejection/s in the cache so that we can still serve status responses for them
            self.recently_committed_intents.push(intent_hash, ());
        }
    }

    pub fn get_rejection_status<'a>(
        &'a mut self,
        intent_hash: &IntentHash,
        payload_hash: &UserPayloadHash,
    ) -> Option<&'a RejectionReason> {
        if self.recently_committed_intents.get(intent_hash).is_some() {
            return Some(&RejectionReason::IntentHashCommitted);
        }
        let cached_status = self.rejected_payloads.get(payload_hash)?.clone();
        if self.should_be_forgotten(&cached_status) {
            let removed = self.rejected_payloads.pop(payload_hash);
            if let Some(r) = removed {
                self.handled_removed(*payload_hash, r)
            }
            return None;
        }
        Some(&self.rejected_payloads.get(payload_hash)?.reason)
    }

    pub fn peek_all_rejected_payloads_for_intent(
        &self,
        intent_hash: &IntentHash,
    ) -> HashMap<UserPayloadHash, RejectionRecord> {
        match self.intent_lookup.get(intent_hash) {
            Some(payload_hashes) => payload_hashes
                .iter()
                .map(|payload_hash| {
                    let rejection_record = self
                        .rejected_payloads
                        .peek(payload_hash)
                        .expect("Intent lookup out of sync with rejected payloads");
                    (*payload_hash, rejection_record.clone())
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
        rejection_record: RejectionRecord,
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

    fn should_be_forgotten(&self, rejection_record: &RejectionRecord) -> bool {
        !rejection_record.reason.is_permanent()
            && rejection_record.tracked_timestamp + self.max_time_to_live_for_temporary_rejections
                < Instant::now()
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

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
        // TTL should be long enough to not interfere in the standard test run
        let max_ttl = Duration::from_millis(100);
        let mut cache =
            RejectionCache::new(rejection_limit, recently_committed_intents_limit, max_ttl);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);
        let payload_hash_3 = user_payload_hash(3);
        let payload_hash_4 = user_payload_hash(4);
        let payload_hash_5 = user_payload_hash(5);
        let payload_hash_6 = user_payload_hash(6);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);
        let intent_hash_3 = intent_hash(3);

        let reason_1 =
            RejectionReason::ValidationError(TransactionValidationError::TransactionTooLarge);
        let reason_2 =
            RejectionReason::FromExecution(Box::new(RejectionError::SuccessButFeeLoanNotRepaid));

        // Start by adding 3 payloads against first intent hash. These all fit in, but cache is full
        cache.track_rejection(intent_hash_1, payload_hash_1, reason_1.clone());
        cache.track_rejection(intent_hash_1, payload_hash_2, reason_2.clone());
        cache.track_rejection(intent_hash_1, payload_hash_3, reason_1.clone());
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .len(),
            3
        );

        // Now add another rejection - the first rejection (intent_1, payload_1, reason_1) should drop out
        cache.track_rejection(intent_hash_2, payload_hash_4, reason_1.clone());
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .len(),
            2
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .get(&payload_hash_2)
                .unwrap()
                .reason,
            reason_2
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .get(&payload_hash_3)
                .unwrap()
                .reason,
            reason_1
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_2)
                .len(),
            1
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_2)
                .get(&payload_hash_4)
                .unwrap()
                .reason,
            reason_1
        );

        // Reading transaction status should jump payload 2 back to the top of the cache
        // So (intent_1, payload_3, reason_1) and (intent_2, payload_4, reason_1) should drop out instead
        cache.get_rejection_status(&intent_hash_1, &payload_hash_2);
        cache.track_rejection(intent_hash_3, payload_hash_5, reason_1.clone());
        cache.track_rejection(intent_hash_3, payload_hash_6, reason_1);
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .len(),
            1
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .get(&payload_hash_2)
                .unwrap()
                .reason,
            reason_2
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_2)
                .len(),
            0
        );
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_3)
                .len(),
            2
        );
    }

    #[test]
    fn committed_transaction_checks() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;
        // TTL should be long enough to not interfere in the standard test run
        let max_ttl = Duration::from_millis(100);
        let mut cache =
            RejectionCache::new(rejection_limit, recently_committed_intents_limit, max_ttl);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);

        cache.track_committed_transactions(vec![intent_hash_1]);
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_1)
            .is_some());
        assert!(cache
            .get_rejection_status(&intent_hash_2, &payload_hash_2)
            .is_none());
    }

    #[test]
    fn temporary_rejections_are_evicted_after_ttl() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;
        // TTL should be long enough to not interfere in the standard test run
        let max_ttl = Duration::from_millis(20);
        let not_past_ttl = Duration::from_millis(10);
        let past_ttl = Duration::from_millis(25);
        let mut cache =
            RejectionCache::new(rejection_limit, recently_committed_intents_limit, max_ttl);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);

        let intent_hash_1 = intent_hash(1);

        let temporary_reason =
            RejectionReason::FromExecution(Box::new(RejectionError::SuccessButFeeLoanNotRepaid));
        let permanent_reason =
            RejectionReason::ValidationError(TransactionValidationError::TransactionTooLarge);

        cache.track_rejection(intent_hash_1, payload_hash_1, permanent_reason);
        cache.track_rejection(intent_hash_1, payload_hash_2, temporary_reason);
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_1)
            .is_some());
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_2)
            .is_some());

        // Wait a little - we should still return temporary rejection reason
        // Ideally we'd do this by mocking out time, but this will do for now
        thread::sleep(not_past_ttl);
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .len(),
            2
        );
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_1)
            .is_some());
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_2)
            .is_some());

        // Wait until ttl should have expired - we should only retain permanent rejection reason
        // Note that both are still around for peeks until we do a get, when the temporary reason is evicted
        thread::sleep(past_ttl - not_past_ttl);
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .len(),
            2
        );
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_1)
            .is_some());
        assert!(cache
            .get_rejection_status(&intent_hash_1, &payload_hash_2)
            .is_none());
        assert_eq!(
            cache
                .peek_all_rejected_payloads_for_intent(&intent_hash_1)
                .len(),
            1
        );
    }
}
