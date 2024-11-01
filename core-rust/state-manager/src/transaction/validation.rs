use crate::prelude::*;

pub trait LedgerTransactionValidationErrorExtensions {
    fn into_user_validation_error(self) -> TransactionValidationError;
}

impl LedgerTransactionValidationErrorExtensions for LedgerTransactionValidationError {
    // Should only be called on errors from validating user transactions
    fn into_user_validation_error(self) -> TransactionValidationError {
        match self {
            LedgerTransactionValidationError::ValidationError(x) => x,
            LedgerTransactionValidationError::GenesisTransactionNotCurrentlyPermitted
            | LedgerTransactionValidationError::UserTransactionNotCurrentlyPermitted
            | LedgerTransactionValidationError::ValidateTransactionNotCurrentlyPermitted
            | LedgerTransactionValidationError::ProtocolUpdateNotCurrentlyPermitted
            | LedgerTransactionValidationError::FlashNotCurrentlyPermitted => {
                panic!("into_user_validation_error called unexpectedly on an incorrect transaction type error")
            }
        }
    }
}

/// A validator for `NotarizedTransaction`, deciding whether they would be rejected or not-rejected
/// (i.e. "committable") at a specific state of the `store`.
pub struct CommittabilityValidator {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    execution_configurator: Arc<ExecutionConfigurator>,
    transaction_validator: Arc<RwLock<TransactionValidator>>,
}

impl CommittabilityValidator {
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        execution_configurator: Arc<ExecutionConfigurator>,
        transaction_validator: Arc<RwLock<TransactionValidator>>,
    ) -> Self {
        Self {
            database,
            execution_configurator,
            transaction_validator,
        }
    }

    pub fn prepare_from_raw(
        &self,
        transaction: &RawNotarizedTransaction,
    ) -> Result<PreparedUserTransaction, PrepareError> {
        transaction.prepare(self.transaction_validator.read().preparation_settings())
    }

    pub fn validate(
        &self,
        transaction: PreparedUserTransaction,
    ) -> Result<ValidatedUserTransaction, TransactionValidationError> {
        transaction.validate(self.transaction_validator.read().deref())
    }
}

impl CommittabilityValidator {
    /// Determine whether it would be rejected given the current state of the substate store.
    pub fn check_for_rejection(
        &self,
        executable: &ExecutableTransaction,
        user_hashes: &UserTransactionHashes,
        timestamp: SystemTime,
    ) -> TransactionAttempt {
        let database = self.database.snapshot();
        let executed_at_state_version = database.max_state_version();

        let existing =
            database.get_txn_state_version_by_identifier(&user_hashes.transaction_intent_hash);

        if let Some(state_version) = existing {
            let committed_transaction_identifiers = database
                .get_committed_transaction_identifiers(state_version)
                .expect("transaction of a state version obtained from an index");

            return TransactionAttempt {
                rejection: Some(MempoolRejectionReason::TransactionIntentAlreadyCommitted(
                    AlreadyCommittedError {
                        notarized_transaction_hash: user_hashes.notarized_transaction_hash,
                        committed_state_version: state_version,
                        committed_notarized_transaction_hash: committed_transaction_identifiers
                            .transaction_hashes
                            .as_user()
                            .expect("non-user transaction located by intent hash")
                            .notarized_transaction_hash,
                    },
                )),
                against_state: AtState::Specific(AtSpecificState::Committed {
                    state_version: executed_at_state_version,
                }),
                timestamp,
            };
        }

        let receipt = self
            .execution_configurator
            .wrap_pending_transaction(executable, user_hashes)
            .execute_on(database.deref());

        let result = match receipt.result {
            TransactionResult::Reject(RejectResult { reason }) => {
                if matches!(
                    reason,
                    ExecutionRejectionReason::IntentHashPreviouslyCommitted(
                        IntentHash::Transaction(_)
                    )
                ) {
                    // Note - this panic protects against the invariant that already_committed_error()
                    panic!(
                        "[INVARIANT VIOLATION] When checking for rejection against a database snapshot, a transaction intent {:?} was not found in the Node's stores, but was reported as committed by the Engine",
                        user_hashes.transaction_intent_hash
                    );
                }
                Err(MempoolRejectionReason::FromExecution(Box::new(reason)))
            }
            TransactionResult::Commit(..) => Ok(()),
            TransactionResult::Abort(abort_result) => {
                // The transaction aborted after the fee loan was repaid - meaning the transaction result would get committed
                match abort_result.reason {
                    AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment => Ok(()),
                }
            }
        };

        TransactionAttempt {
            rejection: result.err(),
            against_state: AtState::Specific(AtSpecificState::Committed {
                state_version: executed_at_state_version,
            }),
            timestamp,
        }
    }
}

/// A caching wrapper for a `CommittabilityValidator`.
pub struct CachedCommittabilityValidator {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    committability_validator: Arc<RwLock<CommittabilityValidator>>,
    pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
}

impl CachedCommittabilityValidator {
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        committability_validator: Arc<RwLock<CommittabilityValidator>>,
        pending_transaction_result_cache: Arc<RwLock<PendingTransactionResultCache>>,
    ) -> Self {
        Self {
            database,
            committability_validator,
            pending_transaction_result_cache,
        }
    }

    pub fn prepare_from_raw(
        &self,
        transaction: &RawNotarizedTransaction,
    ) -> Result<PreparedUserTransaction, PrepareError> {
        self.committability_validator
            .read()
            .prepare_from_raw(transaction)
    }

    fn read_record(&self, prepared: &PreparedUserTransaction) -> Option<PendingTransactionRecord> {
        // Even though we only want to read the cache here, the LRU structs require a write lock
        self.pending_transaction_result_cache
            .write()
            .get_pending_transaction_record(
                &prepared.transaction_intent_hash(),
                &prepared.notarized_transaction_hash(),
            )
    }

    fn write_attempt(
        &self,
        metadata: TransactionMetadata,
        attempt: TransactionAttempt,
    ) -> PendingTransactionRecord {
        self.pending_transaction_result_cache
            .write()
            .track_transaction_result(
                metadata.intent_hash,
                metadata.notarized_transaction_hash,
                Some(metadata.end_epoch_exclusive),
                attempt,
            )
    }
}

struct TransactionMetadata {
    intent_hash: TransactionIntentHash,
    notarized_transaction_hash: NotarizedTransactionHash,
    end_epoch_exclusive: Epoch,
}

impl TransactionMetadata {
    pub fn read_from_user_executable(
        executable: &ExecutableTransaction,
        user_hashes: &UserTransactionHashes,
    ) -> Self {
        Self {
            intent_hash: user_hashes.transaction_intent_hash,
            notarized_transaction_hash: user_hashes.notarized_transaction_hash,
            end_epoch_exclusive: executable
                .overall_epoch_range()
                .expect("User executable transactions should have an epoch range")
                .end_epoch_exclusive,
        }
    }

    pub fn read_from_prepared(prepared: &PreparedUserTransaction) -> Self {
        Self {
            intent_hash: prepared.transaction_intent_hash(),
            notarized_transaction_hash: prepared.notarized_transaction_hash(),
            end_epoch_exclusive: match prepared {
                #[allow(deprecated)]
                PreparedUserTransaction::V1(prepared) => {
                    prepared
                        .signed_intent
                        .intent
                        .header
                        .inner
                        .end_epoch_exclusive
                }
                PreparedUserTransaction::V2(prepared) => {
                    let transaction_intent = &prepared.signed_intent.transaction_intent;

                    let root_intent_expiry_epoch = transaction_intent
                        .root_intent_core
                        .header
                        .inner
                        .end_epoch_exclusive;
                    let non_root_intent_expiry_epochs = transaction_intent
                        .non_root_subintents
                        .subintents
                        .iter()
                        .map(|subintent| subintent.intent_core.header.inner.end_epoch_exclusive);

                    // Unwrapping as we know it's non-empty
                    std::iter::once(root_intent_expiry_epoch)
                        .chain(non_root_intent_expiry_epochs)
                        .min()
                        .unwrap()
                }
            },
        }
    }
}

enum ShouldRecalculate {
    Yes,
    No(PendingTransactionRecord),
}

pub enum CheckMetadata {
    Cached,
    Fresh(StaticValidation),
}

impl CheckMetadata {
    pub fn was_cached(&self) -> bool {
        match self {
            Self::Cached => true,
            Self::Fresh(_) => false,
        }
    }
}

pub enum StaticValidation {
    Valid {
        executable: ExecutableTransaction,
        user_hashes: UserTransactionHashes,
    },
    Invalid,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ForceRecalculation {
    Yes,
    IfCachedAsValid,
    No,
}

impl CachedCommittabilityValidator {
    /// Reads the transaction rejection status from the cache, else calculates it fresh, using
    /// `CommittabilityValidator`.
    ///
    /// The result is stored in the cache.
    /// If the transaction is freshly rejected, the caller should perform additional cleanup,
    /// e.g. removing the transaction from the mempool.
    ///
    /// Its pending transaction record is returned, along with a boolean about whether the last
    /// attempt was cached, and the validated notarized transaction (if it's new)
    pub fn check_for_rejection_cached(
        &self,
        prepared: PreparedUserTransaction,
        force_recalculate: ForceRecalculation,
    ) -> (PendingTransactionRecord, CheckMetadata) {
        let current_time = SystemTime::now();

        if let ShouldRecalculate::No(record) =
            self.should_recalculate(&prepared, current_time, force_recalculate)
        {
            return (record, CheckMetadata::Cached);
        }

        let metadata = TransactionMetadata::read_from_prepared(&prepared);

        let read_committability_validator = self.committability_validator.read();
        match read_committability_validator.validate(prepared) {
            Ok(validated) => {
                // Transaction was valid - let's also attempt to execute it
                let user_hashes = validated.hashes();
                let executable = validated.create_executable();
                let attempt = read_committability_validator.check_for_rejection(
                    &executable,
                    &user_hashes,
                    current_time,
                );
                (
                    self.write_attempt(metadata, attempt),
                    CheckMetadata::Fresh(StaticValidation::Valid {
                        executable,
                        user_hashes,
                    }),
                )
            }
            Err(validation_error) => {
                // The transaction is statically invalid
                let attempt = TransactionAttempt {
                    rejection: Some(MempoolRejectionReason::ValidationError(validation_error)),
                    against_state: AtState::Static,
                    timestamp: current_time,
                };
                (
                    self.write_attempt(metadata, attempt),
                    CheckMetadata::Fresh(StaticValidation::Invalid),
                )
            }
        }
    }

    /// Recalculates (i.e. ignoring the cache) the given already-validatated transaction's status,
    /// using `CommittabilityValidator`.
    ///
    /// The result is stored in the cache.
    /// If the transaction is freshly rejected, the caller should perform additional cleanup,
    /// e.g. removing the transaction from the mempool.
    ///
    /// Returns the transaction's new pending transaction record.
    pub fn check_for_rejection_validated(
        &self,
        executable: &ExecutableTransaction,
        user_hashes: &UserTransactionHashes,
    ) -> PendingTransactionRecord {
        let metadata = TransactionMetadata::read_from_user_executable(executable, user_hashes);

        let attempt = self.committability_validator.read().check_for_rejection(
            executable,
            user_hashes,
            SystemTime::now(),
        );

        self.write_attempt(metadata, attempt)
    }

    fn should_recalculate(
        &self,
        prepared: &PreparedUserTransaction,
        current_time: SystemTime,
        force_recalculate: ForceRecalculation,
    ) -> ShouldRecalculate {
        if force_recalculate == ForceRecalculation::Yes {
            return ShouldRecalculate::Yes;
        }

        let current_epoch = self.database.snapshot().get_epoch_and_round().0;
        let record_option = self.read_record(prepared);

        if let Some(record) = record_option {
            if !record.should_recalculate(current_epoch, current_time) {
                if force_recalculate == ForceRecalculation::IfCachedAsValid
                    && record.latest_attempt.rejection.is_none()
                {
                    return ShouldRecalculate::Yes;
                }
                return ShouldRecalculate::No(record);
            }
        }
        ShouldRecalculate::Yes
    }
}
