use crate::transaction::types::LedgerTransaction;
use scrypto::buffer::scrypto_decode;
use transaction::errors::TransactionValidationError;
use transaction::model::{Executable, NotarizedTransaction};
use transaction::validation::ValidationConfig;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator,
};

pub struct UserTransactionValidator {
    pub base_validation_config: ValidationConfig,
    pub intent_hash_manager: TestIntentHashManager,
}

// NB - For alphanet, we allow transactions of up to 24MB, up from
// 4MB MAX_PAYLOAD_SIZE in the radixdlt-scrypto codebase
// This limit will likely need reducing after some review
pub const OVERRIDE_MAX_PAYLOAD_SIZE: usize = 24 * 1024 * 1024;

impl UserTransactionValidator {
    /// Checks the Payload max size, and SBOR decodes to a NotarizedTransaction if the size is okay
    pub fn parse_unvalidated_user_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<NotarizedTransaction, TransactionValidationError> {
        if transaction_payload.len() > OVERRIDE_MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: NotarizedTransaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    /// Performs static validation only
    pub fn parse_and_validate_user_transaction_slice(
        &self,
        epoch: u64, // Temporary
        transaction_payload: &[u8],
    ) -> Result<ValidatedTransaction<NotarizedTransaction>, TransactionValidationError> {
        let notarized_transaction =
            Self::parse_unvalidated_user_transaction_from_slice(transaction_payload)?;
        self.validate_user_transaction(epoch, notarized_transaction)
    }

    /// Performs static validation only
    pub fn validate_user_transaction(
        &self,
        epoch: u64, // Temporary
        transaction: NotarizedTransaction,
    ) -> Result<ValidatedTransaction<NotarizedTransaction>, TransactionValidationError> {
        let mut config = self.base_validation_config;
        config.current_epoch = epoch;
        let validator = NotarizedTransactionValidator::new(config);

        validator
            .validate(transaction.clone(), &self.intent_hash_manager)
            .map(|executable| ValidatedTransaction {
                transaction,
                executable,
            })
    }
}

pub struct ValidatedTransaction<T> {
    pub transaction: T,
    pub executable: Executable,
}

pub struct CommittedTransactionValidator {
    pub base_validation_config: ValidationConfig,
    pub intent_hash_manager: TestIntentHashManager,
}

impl CommittedTransactionValidator {
    pub fn parse_unvalidated_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<LedgerTransaction, TransactionValidationError> {
        let transaction: LedgerTransaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    pub fn parse_and_validate_transaction_slice(
        &self,
        epoch: u64, // Temporary
        transaction_payload: &[u8],
    ) -> Result<ValidatedTransaction<LedgerTransaction>, TransactionValidationError> {
        // TODO: Need a good way to do payload transaction size here
        let transaction = Self::parse_unvalidated_transaction_from_slice(transaction_payload)?;
        self.validate_transaction(epoch, transaction)
    }

    fn validate_transaction(
        &self,
        epoch: u64, // Temporary
        transaction: LedgerTransaction,
    ) -> Result<ValidatedTransaction<LedgerTransaction>, TransactionValidationError> {
        let mut config = self.base_validation_config;
        config.current_epoch = epoch;
        let validator = NotarizedTransactionValidator::new(config);
        let executable = match transaction.clone() {
            LedgerTransaction::User(notarized_transaction) => {
                validator.validate(notarized_transaction, &self.intent_hash_manager)
            }
            LedgerTransaction::Validator(validator_transaction) => Ok(validator_transaction.into()),
        }?;
        Ok(ValidatedTransaction {
            transaction,
            executable,
        })
    }
}
