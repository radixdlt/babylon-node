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
    formatter: Arc<Formatter>,
}

impl CommittabilityValidator {
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        execution_configurator: Arc<ExecutionConfigurator>,
        transaction_validator: Arc<RwLock<TransactionValidator>>,
        formatter: Arc<Formatter>,
    ) -> Self {
        Self {
            database,
            execution_configurator,
            transaction_validator,
            formatter,
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

    pub fn current_epoch(&self) -> Epoch {
        self.database.snapshot().get_epoch_and_round().0
    }

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

        trace!(
            "Starting mempool execution of {}",
            user_hashes
                .transaction_intent_hash
                .display(&*self.formatter),
        );

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
