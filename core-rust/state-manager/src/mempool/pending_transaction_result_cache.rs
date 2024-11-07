use crate::prelude::*;

use lru::LruCache;
use std::{
    collections::hash_map::Entry,
    fmt,
    num::NonZeroUsize,
    ops::Add,
    time::{Duration, SystemTime},
};

pub type ExecutionRejectionReason = RejectionReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolRejectionReason {
    TransactionIntentAlreadyCommitted(AlreadyCommittedError),
    SubintentAlreadyFinalized(SubintentAlreadyFinalizedError),
    FromExecution(Box<ExecutionRejectionReason>),
    ValidationError(TransactionValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlreadyCommittedError {
    pub notarized_transaction_hash: NotarizedTransactionHash,
    pub committed_state_version: StateVersion,
    pub committed_notarized_transaction_hash: NotarizedTransactionHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubintentAlreadyFinalizedError {
    pub subintent_hash: SubintentHash,
    pub committed_state_version: StateVersion,
    pub committed_transaction_intent_hash: TransactionIntentHash,
    pub committed_notarized_transaction_hash: NotarizedTransactionHash,
}

impl From<TransactionValidationError> for MempoolRejectionReason {
    fn from(value: TransactionValidationError) -> Self {
        Self::ValidationError(value)
    }
}

impl MempoolRejectionReason {
    pub fn is_from_execution(&self) -> bool {
        match self {
            MempoolRejectionReason::TransactionIntentAlreadyCommitted(_) => false,
            MempoolRejectionReason::SubintentAlreadyFinalized(_) => false,
            MempoolRejectionReason::FromExecution(_) => true,
            MempoolRejectionReason::ValidationError(_) => false,
        }
    }

    pub fn is_permanent_for_payload(&self, at_state: &AtState) -> bool {
        if self.is_from_execution() && !at_state.can_mark_permanent_rejections() {
            return false;
        }
        self.permanence().is_permanent_for_payload()
    }

    pub fn is_permanent_for_intent(&self, at_state: &AtState) -> bool {
        if self.is_from_execution() && !at_state.can_mark_permanent_rejections() {
            return false;
        }
        self.permanence().is_permanent_for_intent()
    }

    pub fn transaction_intent_already_committed_error(
        &self,
        at_state: &AtState,
    ) -> Option<&AlreadyCommittedError> {
        if !at_state.can_mark_permanent_rejections() {
            return None;
        }

        match self {
            MempoolRejectionReason::TransactionIntentAlreadyCommitted(already_committed_error) => {
                Some(already_committed_error)
            }
            MempoolRejectionReason::SubintentAlreadyFinalized(_) => None,
            MempoolRejectionReason::FromExecution(_) => None,
            MempoolRejectionReason::ValidationError(_) => None,
        }
    }

    pub fn permanence(&self) -> RejectionPermanence {
        match self {
            MempoolRejectionReason::TransactionIntentAlreadyCommitted(_)
            | MempoolRejectionReason::SubintentAlreadyFinalized(_) => {
                // These are permanent for the intent - because even other, non-committed transactions
                // of the same intent will fail with `ExecutionRejectionReason::IntentHashPreviouslyCommitted`
                RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
            }
            MempoolRejectionReason::FromExecution(rejection_error) => match **rejection_error {
                ExecutionRejectionReason::BootloadingError(_) => {
                    RejectionPermanence::default_temporary()
                }
                ExecutionRejectionReason::SuccessButFeeLoanNotRepaid => {
                    RejectionPermanence::default_temporary()
                }
                ExecutionRejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(_) => {
                    RejectionPermanence::default_temporary()
                }
                ExecutionRejectionReason::TransactionEpochNotYetValid { valid_from, .. } => {
                    RejectionPermanence::Temporary {
                        retry: RetrySettings::FromEpoch { epoch: valid_from },
                    }
                }
                ExecutionRejectionReason::TransactionEpochNoLongerValid { .. } => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
                }
                ExecutionRejectionReason::TransactionProposerTimestampNotYetValid {
                    valid_from_inclusive,
                    ..
                } => RejectionPermanence::Temporary {
                    retry: RetrySettings::FromProposerTimestamp {
                        proposer_timestamp: valid_from_inclusive,
                    },
                },
                ExecutionRejectionReason::TransactionProposerTimestampNoLongerValid { .. } => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
                }
                ExecutionRejectionReason::IntentHashPreviouslyCommitted(_) => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
                }
                ExecutionRejectionReason::IntentHashPreviouslyCancelled(_) => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
                }
                ExecutionRejectionReason::SubintentsNotYetSupported => {
                    RejectionPermanence::wait_for_protocol_update()
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
                TransactionValidationError::SignatureValidationError { .. } => {
                    RejectionPermanence::PermanentForPayload
                }
                TransactionValidationError::TransactionVersionNotPermitted(_) => {
                    RejectionPermanence::wait_for_protocol_update()
                }
                // The subintent structure is a property of the intent
                TransactionValidationError::SubintentStructureError { .. } => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
                }
                TransactionValidationError::IntentValidationError { .. } => {
                    RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionPermanence {
    PermanentForPayload,
    PermanentForAnyPayloadWithThisTransactionIntent,
    Temporary { retry: RetrySettings },
}

impl RejectionPermanence {
    pub fn wait_for_protocol_update() -> Self {
        // For want of something better, let's just wait for 5 minutes
        Self::Temporary {
            retry: RetrySettings::AfterDelay {
                base_delay: Duration::from_secs(5 * 60),
            },
        }
    }

    pub fn default_temporary() -> Self {
        // Wait 2 minutes in case things clear up
        Self::Temporary {
            retry: RetrySettings::AfterDelay {
                base_delay: Duration::from_secs(2 * 60),
            },
        }
    }

    pub fn is_permanent_for_payload(&self) -> bool {
        match self {
            RejectionPermanence::PermanentForPayload => true,
            RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent => true,
            RejectionPermanence::Temporary { .. } => false,
        }
    }

    pub fn is_permanent_for_intent(&self) -> bool {
        match self {
            RejectionPermanence::PermanentForPayload => false,
            RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent => true,
            RejectionPermanence::Temporary { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrySettings {
    AfterDelay { base_delay: Duration },
    FromEpoch { epoch: Epoch },
    FromProposerTimestamp { proposer_timestamp: Instant },
}

impl fmt::Display for MempoolRejectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MempoolRejectionReason::SubintentAlreadyFinalized(error) => {
                write!(f, "Subintent already finalized: {error:?}")
            }
            MempoolRejectionReason::TransactionIntentAlreadyCommitted(error) => {
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
        self.against_state.can_mark_permanent_rejections()
    }

    pub fn marks_permanent_rejection_for_payload(&self) -> bool {
        if let Some(rejection_reason) = &self.rejection {
            rejection_reason.is_permanent_for_payload(&self.against_state)
        } else {
            false
        }
    }

    pub fn marks_permanent_rejection_for_intent(&self) -> bool {
        if let Some(rejection_reason) = &self.rejection {
            rejection_reason.is_permanent_for_intent(&self.against_state)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtState {
    // We might need this to be versioned by protocol update later...
    Static,
    Specific(AtSpecificState),
}

impl AtState {
    pub fn can_mark_permanent_rejections(&self) -> bool {
        match self {
            AtState::Static => true,
            AtState::Specific(specific) => match specific {
                AtSpecificState::Committed { .. } => true,
                AtSpecificState::PendingPreparingVertices { .. } => false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtSpecificState {
    Committed {
        state_version: StateVersion,
    },
    PendingPreparingVertices {
        base_committed_state_version: StateVersion,
        pending_transactions_root: TransactionTreeHash,
    },
}

impl AtSpecificState {
    pub fn committed_version(&self) -> StateVersion {
        match self {
            Self::Committed { state_version } => *state_version,
            Self::PendingPreparingVertices {
                base_committed_state_version,
                ..
            } => *base_committed_state_version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryFrom {
    Never,
    FromTime(SystemTime),
    FromEpoch(Epoch),
    Whenever,
}

#[derive(Debug, Clone)]
pub struct PendingExecutedTransaction {
    pub executable: ExecutableTransaction,
    pub user_hashes: UserTransactionHashes,
    pub latest_attempt_against_state: AtSpecificState,
}

impl PendingExecutedTransaction {
    pub fn new(
        executable: ExecutableTransaction,
        user_hashes: UserTransactionHashes,
        against_state: AtState,
    ) -> Self {
        let AtState::Specific(latest_attempt_against_state) = against_state else {
            panic!("transaction must have been executed against some state")
        };
        Self {
            executable,
            user_hashes,
            latest_attempt_against_state,
        }
    }
}

impl PendingTransactionRecord {
    pub fn new(invalid_from_epoch: Option<Epoch>, attempt: TransactionAttempt) -> Self {
        let mut new_record = Self {
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
    /// * If `check == CheckMetadata::Cached`, the latest attempt must be a rejection.
    ///   This precondition is met if the record/metadata come from a call using `ForceRecalculation::IfCachedAsValid`
    pub fn should_accept_into_mempool(
        self,
        check: CheckMetadata,
    ) -> Result<PendingExecutedTransaction, MempoolAddRejection> {
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
            CheckMetadata::Fresh(StaticValidation::Valid {
                executable,
                user_hashes,
            }) => Ok(PendingExecutedTransaction::new(
                executable,
                user_hashes,
                self.latest_attempt.against_state,
            )),
            CheckMetadata::Fresh(StaticValidation::Invalid) => {
                panic!("A statically invalid transaction should already have been handled in the above")
            }
        }
    }

    pub fn most_applicable_status(&self) -> &TransactionAttempt {
        self.earliest_permanent_rejection
            .as_ref()
            .unwrap_or(&self.latest_attempt)
    }

    pub fn most_applicable_rejection(&self) -> Option<&MempoolRejectionReason> {
        self.most_applicable_status().rejection.as_ref()
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
                    RejectionPermanence::Temporary {
                        retry:
                            RetrySettings::FromProposerTimestamp {
                                proposer_timestamp:
                                    Instant {
                                        seconds_since_unix_epoch,
                                    },
                            },
                    } => {
                        u64::try_from(seconds_since_unix_epoch)
                            .ok()
                            // Add one more second so we don't risk retrying before the timestamp on ledger has updated
                            .and_then(|seconds_since_unix_epoch| {
                                seconds_since_unix_epoch.checked_add(1)
                            })
                            .and_then(|retry_in_seconds| {
                                SystemTime::UNIX_EPOCH
                                    .checked_add(Duration::from_secs(retry_in_seconds))
                            })
                            .map(RetryFrom::FromTime)
                            .unwrap_or(RetryFrom::Never)
                    }
                    RejectionPermanence::PermanentForPayload
                    | RejectionPermanence::PermanentForAnyPayloadWithThisTransactionIntent => {
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
    pending_transaction_records: LruCache<
        NotarizedTransactionHash,
        (
            PendingTransactionRecord,
            TransactionIntentHash,
            Vec<SubintentHash>,
        ),
    >,
    // INVARIANT: The `intent_lookup` and `subintent_lookup` are kept exactly in sync with
    // pending_transaction_records, and provide, an inverse lookup from respectively:
    // * The intent hash in the `PendingTransactionRecord` to the notarized hash
    // * Any subintent hash in the `PendingTransactionRecord` to the notarized hash
    intent_lookup: HashMap<TransactionIntentHash, HashSet<NotarizedTransactionHash>>,
    subintent_lookup: HashMap<SubintentHash, HashSet<NotarizedTransactionHash>>,
    recently_committed_intents: LruCache<TransactionIntentHash, CommittedIntentRecord>,
    recently_finalized_subintents: LruCache<SubintentHash, CommittedSubintentRecord>,
}

impl PendingTransactionResultCache {
    pub fn new(
        pending_txn_records_max_count: NonZeroUsize,
        committed_intents_max_size: NonZeroUsize,
        committed_subintents_max_size: NonZeroUsize,
    ) -> Self {
        PendingTransactionResultCache {
            pending_transaction_records: LruCache::new(pending_txn_records_max_count),
            intent_lookup: HashMap::new(),
            subintent_lookup: HashMap::new(),
            recently_committed_intents: LruCache::new(committed_intents_max_size),
            recently_finalized_subintents: LruCache::new(committed_subintents_max_size),
        }
    }

    /// Note - the invalid_from_epoch only needs to be provided if the attempt is not a permanent rejection
    pub fn track_transaction_result(
        &mut self,
        user_transaction_hashes: UserTransactionHashes,
        invalid_from_epoch: Option<Epoch>,
        attempt: TransactionAttempt,
    ) -> PendingTransactionRecord {
        let existing_record = self
            .pending_transaction_records
            .get_mut(&user_transaction_hashes.notarized_transaction_hash);

        let is_permanent_rejection = attempt.marks_permanent_rejection_for_intent();

        if let Some((record, _, _)) = existing_record {
            record.track_attempt(attempt);
            return record.clone();
        }

        let new_record = PendingTransactionRecord::new(invalid_from_epoch, attempt);

        // If it's a permanent rejection, then:
        // - It could be statically invalid
        let subintent_hashes_to_store = if is_permanent_rejection {
            vec![]
        } else {
            user_transaction_hashes.non_root_subintent_hashes
        };

        self.handled_added(
            user_transaction_hashes.transaction_intent_hash,
            user_transaction_hashes.notarized_transaction_hash,
            subintent_hashes_to_store.as_slice(),
        );

        let pending_record = (
            new_record.clone(),
            user_transaction_hashes.transaction_intent_hash,
            subintent_hashes_to_store,
        );

        // NB - removed is the item kicked out of the LRU cache if it's at capacity
        let removed = self.pending_transaction_records.push(
            user_transaction_hashes.notarized_transaction_hash,
            pending_record,
        );

        if let Some((notarized_transaction_hash, removed)) = removed {
            let (_, transaction_intent_hash, subintent_hashes) = removed;
            self.handled_removed(
                notarized_transaction_hash,
                transaction_intent_hash,
                subintent_hashes,
            );
        }

        new_record
    }

    pub fn track_committed_transactions(
        &mut self,
        current_timestamp: SystemTime,
        committed_transactions: Vec<CommittedUserTransactionIdentifiers>,
    ) {
        for committed_transaction in committed_transactions {
            let notarized_transaction_hash = committed_transaction.notarized_transaction_hash;
            let transaction_intent_hash = committed_transaction.transaction_intent_hash;

            // Note - we keep the relevant statuses of all known payloads for the intent in the cache
            // so that we can still serve status responses for them - we just ensure we mark them as rejected
            for nullification in committed_transaction.nullifications {
                let Nullification::Intent { intent_hash, .. } = nullification;
                let nullified_records = match &intent_hash {
                    IntentHash::Transaction(_) => {
                        self.recently_committed_intents.push(
                            transaction_intent_hash,
                            CommittedIntentRecord {
                                state_version: committed_transaction.state_version,
                                notarized_transaction_hash,
                                timestamp: current_timestamp,
                            },
                        );
                        self.intent_lookup.get(&transaction_intent_hash)
                    }
                    IntentHash::Subintent(subintent_hash) => {
                        self.recently_finalized_subintents.push(
                            *subintent_hash,
                            CommittedSubintentRecord {
                                state_version: committed_transaction.state_version,
                                transaction_intent_hash,
                                notarized_transaction_hash,
                                timestamp: current_timestamp,
                            },
                        );
                        self.subintent_lookup.get(subintent_hash)
                    }
                };
                if let Some(nullified_hashes) = nullified_records {
                    for cached_payload_hash in nullified_hashes {
                        let (record, _, _) = self
                            .pending_transaction_records
                            .peek_mut(cached_payload_hash)
                            .expect(
                                "intent or subintent lookup out of sync with rejected payloads",
                            );

                        let reason = match intent_hash {
                            IntentHash::Transaction(_) => {
                                MempoolRejectionReason::TransactionIntentAlreadyCommitted(
                                    AlreadyCommittedError {
                                        notarized_transaction_hash: *cached_payload_hash,
                                        committed_state_version: committed_transaction
                                            .state_version,
                                        committed_notarized_transaction_hash:
                                            notarized_transaction_hash,
                                    },
                                )
                            }
                            IntentHash::Subintent(subintent_hash) => {
                                MempoolRejectionReason::SubintentAlreadyFinalized(
                                    SubintentAlreadyFinalizedError {
                                        subintent_hash,
                                        committed_transaction_intent_hash: committed_transaction
                                            .transaction_intent_hash,
                                        committed_state_version: committed_transaction
                                            .state_version,
                                        committed_notarized_transaction_hash:
                                            notarized_transaction_hash,
                                    },
                                )
                            }
                        };

                        // We even overwrite the record for transaction which got committed here
                        // because this is a cache for pending transactions, and it can't be re-committed
                        record.track_attempt(TransactionAttempt {
                            rejection: Some(reason),
                            against_state: AtState::Specific(AtSpecificState::Committed {
                                state_version: committed_transaction.state_version,
                            }),
                            timestamp: current_timestamp,
                        })
                    }
                }
            }
        }
    }

    pub fn get_pending_transaction_record(
        &mut self,
        user_hashes: UserTransactionHashes,
    ) -> Option<PendingTransactionRecord> {
        if let Some((record, _, _)) = self
            .pending_transaction_records
            .get(&user_hashes.notarized_transaction_hash)
        {
            return Some(record.clone());
        }
        if let Some(committed_intent_record) = self
            .recently_committed_intents
            .get(&user_hashes.transaction_intent_hash)
        {
            // We might not have a pending transaction record for this, but we know it has to be rejected
            // due to the committed intent cache - so let's create and return a transient committed record for it
            return Some(PendingTransactionRecord::new(
                None,
                TransactionAttempt {
                    rejection: Some(MempoolRejectionReason::TransactionIntentAlreadyCommitted(
                        AlreadyCommittedError {
                            notarized_transaction_hash: user_hashes.notarized_transaction_hash,
                            committed_state_version: committed_intent_record.state_version,
                            committed_notarized_transaction_hash: committed_intent_record
                                .notarized_transaction_hash,
                        },
                    )),
                    against_state: AtState::Specific(AtSpecificState::Committed {
                        state_version: committed_intent_record.state_version,
                    }),
                    timestamp: committed_intent_record.timestamp,
                },
            ));
        }
        for subintent_hash in user_hashes.non_root_subintent_hashes {
            if let Some(committed_subintent_record) =
                self.recently_finalized_subintents.get(&subintent_hash)
            {
                // We might not have a pending transaction record for this, but we know it has to be rejected
                // due to the committed subintent cache - so let's create and return a transient committed record for it
                return Some(PendingTransactionRecord::new(
                    None,
                    TransactionAttempt {
                        rejection: Some(MempoolRejectionReason::SubintentAlreadyFinalized(
                            SubintentAlreadyFinalizedError {
                                subintent_hash,
                                committed_transaction_intent_hash: committed_subintent_record
                                    .transaction_intent_hash,
                                committed_state_version: committed_subintent_record.state_version,
                                committed_notarized_transaction_hash: committed_subintent_record
                                    .notarized_transaction_hash,
                            },
                        )),
                        against_state: AtState::Specific(AtSpecificState::Committed {
                            state_version: committed_subintent_record.state_version,
                        }),
                        timestamp: committed_subintent_record.timestamp,
                    },
                ));
            }
        }

        None
    }

    pub fn peek_all_known_payloads_for_intent(
        &self,
        intent_hash: &TransactionIntentHash,
    ) -> HashMap<NotarizedTransactionHash, PendingTransactionRecord> {
        match self.intent_lookup.get(intent_hash) {
            Some(payload_hashes) => payload_hashes
                .iter()
                .map(|payload_hash| {
                    let (record, _, _) = self
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
        intent_hash: TransactionIntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
        subintent_hashes: &[SubintentHash],
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
        // Add the subintent hash <-> payload hash lookup
        for subintent_hash in subintent_hashes {
            match self.subintent_lookup.entry(*subintent_hash) {
                Entry::Occupied(mut e) => {
                    e.get_mut().insert(notarized_transaction_hash);
                }
                Entry::Vacant(e) => {
                    e.insert(HashSet::from([notarized_transaction_hash]));
                }
            }
        }
    }

    fn handled_removed(
        &mut self,
        notarized_transaction_hash: NotarizedTransactionHash,
        intent_hash: TransactionIntentHash,
        subintent_hashes: Vec<SubintentHash>,
    ) {
        // Remove the intent hash <-> payload hash lookup
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
        for subintent_hash in subintent_hashes {
            match self.subintent_lookup.entry(subintent_hash) {
                Entry::Occupied(e) if e.get().len() == 1 => {
                    e.remove_entry();
                }
                Entry::Occupied(mut e) if e.get().len() > 1 => {
                    e.get_mut().remove(&notarized_transaction_hash);
                }
                Entry::Occupied(_) => {
                    // num_hashes == 0
                    panic!("Invalid subintent_lookup state");
                }
                Entry::Vacant(_) => {
                    panic!("Invalid subintent_lookup state");
                }
            }
        }
    }
}

struct CommittedIntentRecord {
    state_version: StateVersion,
    notarized_transaction_hash: NotarizedTransactionHash,
    timestamp: SystemTime,
}

struct CommittedSubintentRecord {
    state_version: StateVersion,
    transaction_intent_hash: TransactionIntentHash,
    notarized_transaction_hash: NotarizedTransactionHash,
    timestamp: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_engine::system::system_modules::costing::{CostingError, FeeReserveError};

    fn user_payload_hash(nonce: u8) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from(blake2b_256_hash([0, nonce]))
    }

    fn intent_hash(nonce: u8) -> TransactionIntentHash {
        TransactionIntentHash::from(blake2b_256_hash([1, nonce]))
    }

    #[test]
    fn add_evict_and_peek_by_intent_test() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;

        let mut cache = create_subject(rejection_limit, recently_committed_intents_limit, 1);

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
                ExecutionRejectionReason::BootloadingError(
                    BootloadingError::FailedToApplyDeferredCosts(CostingError::FeeReserveError(
                        FeeReserveError::Overflow,
                    )),
                ),
            ))),
            against_state: AtState::Specific(AtSpecificState::Committed {
                state_version: StateVersion::pre_genesis(),
            }),
            timestamp: SystemTime::now(),
        };

        // Start by adding 3 payloads against first intent hash. These all fit in, but cache is full
        cache.track_transaction_result(
            user_hashes(intent_hash_1, payload_hash_1),
            None,
            example_attempt_1.clone(),
        );
        cache.track_transaction_result(
            user_hashes(intent_hash_1, payload_hash_2),
            Some(Epoch::of(0)),
            example_attempt_2.clone(),
        );
        cache.track_transaction_result(
            user_hashes(intent_hash_1, payload_hash_3),
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
            user_hashes(intent_hash_2, payload_hash_4),
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
        cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_2));
        cache.track_transaction_result(
            user_hashes(intent_hash_3, payload_hash_5),
            None,
            example_attempt_1.clone(),
        );
        cache.track_transaction_result(
            user_hashes(intent_hash_3, payload_hash_6),
            None,
            example_attempt_1,
        );
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

        let mut cache = create_subject(rejection_limit, recently_committed_intents_limit, 1);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);

        cache.track_committed_transactions(
            now,
            vec![CommittedUserTransactionIdentifiers {
                state_version: StateVersion::of(1),
                transaction_intent_hash: intent_hash_1,
                notarized_transaction_hash: payload_hash_1,
                nullifications: vec![Nullification::Intent {
                    intent_hash: IntentHash::Transaction(intent_hash_1),
                    expiry_epoch: Epoch::of(15),
                }],
            }],
        );
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_1));
        assert!(record.is_some());

        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_2, payload_hash_2));
        assert!(record.is_none());
    }

    #[test]
    fn successes_and_temporary_rejections_are_marked_as_should_recalculate_appropriately() {
        let rejection_limit = 3;
        let recently_committed_intents_limit = 1;

        let start = SystemTime::now();
        let far_in_future = start.add(Duration::from_secs(u32::MAX as u64));
        let little_in_future = start.add(Duration::from_secs(1));

        let mut cache = create_subject(rejection_limit, recently_committed_intents_limit, 1);

        let payload_hash_1 = user_payload_hash(1);
        let payload_hash_2 = user_payload_hash(2);
        let payload_hash_3 = user_payload_hash(3);
        let payload_hash_4 = user_payload_hash(4);

        let intent_hash_1 = intent_hash(1);
        let intent_hash_2 = intent_hash(2);

        let attempt_with_temporary_rejection = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::FromExecution(Box::new(
                ExecutionRejectionReason::BootloadingError(
                    BootloadingError::FailedToApplyDeferredCosts(CostingError::FeeReserveError(
                        FeeReserveError::Overflow,
                    )),
                ),
            ))),
            against_state: AtState::Specific(AtSpecificState::Committed {
                state_version: StateVersion::pre_genesis(),
            }),
            timestamp: start,
        };
        let attempt_with_rejection_until_epoch_10 = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::FromExecution(Box::new(
                ExecutionRejectionReason::TransactionEpochNotYetValid {
                    valid_from: Epoch::of(10),
                    current_epoch: Epoch::of(9),
                },
            ))),
            against_state: AtState::Specific(AtSpecificState::Committed {
                state_version: StateVersion::of(10000),
            }),
            timestamp: start,
        };
        let attempt_with_permanent_rejection = TransactionAttempt {
            rejection: Some(MempoolRejectionReason::ValidationError(
                TransactionValidationError::TransactionTooLarge,
            )),
            against_state: AtState::Specific(AtSpecificState::Committed {
                state_version: StateVersion::pre_genesis(),
            }),
            timestamp: start,
        };
        let attempt_with_no_rejection = TransactionAttempt {
            rejection: None,
            against_state: AtState::Specific(AtSpecificState::Committed {
                state_version: StateVersion::pre_genesis(),
            }),
            timestamp: start,
        };

        // Permanent Rejection
        cache.track_transaction_result(
            user_hashes(intent_hash_1, payload_hash_1),
            None,
            attempt_with_permanent_rejection,
        );
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_1));
        // Even far in future, a permanent rejection is still there and never ready for recalculation
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(Epoch::of(0), far_in_future));

        // Temporary Rejection
        cache.track_transaction_result(
            user_hashes(intent_hash_1, payload_hash_2),
            Some(Epoch::of(50)),
            attempt_with_temporary_rejection,
        );
        // A little in future, a temporary rejection is not ready for recalculation
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_2));
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(Epoch::of(0), little_in_future));

        // Far in future, a temporary rejection is ready for recalculation
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_2));
        assert!(record.is_some());
        assert!(record
            .unwrap()
            .should_recalculate(Epoch::of(0), far_in_future));

        // No rejection
        cache.track_transaction_result(
            user_hashes(intent_hash_1, payload_hash_3),
            Some(Epoch::of(50)),
            attempt_with_no_rejection,
        );

        // A little in future, a no-rejection result is not ready for recalculation
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_3));
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(Epoch::of(0), little_in_future));

        // Far in future, a no-rejection result is ready for recalculation
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_1, payload_hash_3));
        assert!(record.is_some());
        assert!(record
            .unwrap()
            .should_recalculate(Epoch::of(0), far_in_future));

        // Temporary Rejection with recalculation from epoch 10
        cache.track_transaction_result(
            user_hashes(intent_hash_2, payload_hash_4),
            Some(Epoch::of(50)),
            attempt_with_rejection_until_epoch_10,
        );

        // Still at epoch 9, not yet ready for recalculation
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_2, payload_hash_4));
        let current_epoch = Epoch::of(9);
        assert!(record.is_some());
        assert!(!record
            .unwrap()
            .should_recalculate(current_epoch, little_in_future));

        // Now at epoch 10, now ready for recalculation
        let record =
            cache.get_pending_transaction_record(user_hashes(intent_hash_2, payload_hash_4));
        let current_epoch = Epoch::of(10);
        assert!(record.is_some());
        assert!(record
            .unwrap()
            .should_recalculate(current_epoch, little_in_future));
    }

    fn user_hashes(
        transaction_intent_hash: TransactionIntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
    ) -> UserTransactionHashes {
        UserTransactionHashes {
            transaction_intent_hash,
            signed_transaction_intent_hash: SignedTransactionIntentHash::from_bytes([0; 32]),
            notarized_transaction_hash,
            non_root_subintent_hashes: vec![],
        }
    }

    fn create_subject(
        rejection_limit: usize,
        recently_committed_intents_limit: usize,
        recently_committed_subintents_limit: usize,
    ) -> PendingTransactionResultCache {
        PendingTransactionResultCache::new(
            NonZeroUsize::new(rejection_limit).unwrap(),
            NonZeroUsize::new(recently_committed_intents_limit).unwrap(),
            NonZeroUsize::new(recently_committed_subintents_limit).unwrap(),
        )
    }
}
