use crate::transaction::ledger_transaction::LedgerTransaction;
use radix_engine::types::scrypto_decode;
use transaction::errors::TransactionValidationError;
use transaction::model::{Executable, NotarizedTransaction};
use transaction::validation::ValidationConfig;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator,
};

use super::PreparedLedgerTransaction;

pub struct UserTransactionValidator {
    pub validation_config: ValidationConfig,
    pub intent_hash_manager: TestIntentHashManager,
}

// NB - For alphanet, we allow transactions of up to 24MB, up from
// 4MB MAX_PAYLOAD_SIZE in the radixdlt-scrypto codebase
// This limit will likely need reducing after some review
pub const OVERRIDE_MAX_PAYLOAD_SIZE: usize = 24 * 1024 * 1024;

// Add a few extra bytes for the enum disciminator at the start(!)
pub const OVERRIDE_LEDGER_MAX_PAYLOAD_SIZE: usize = OVERRIDE_MAX_PAYLOAD_SIZE + 10;

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
    pub fn validate_and_create_executable<'a>(
        &self,
        transaction: &'a NotarizedTransaction,
    ) -> Result<Executable<'a>, TransactionValidationError> {
        let validator = NotarizedTransactionValidator::new(self.validation_config);

        validator.validate(transaction, &self.intent_hash_manager)
    }
}

pub struct LedgerTransactionValidator {
    pub validation_config: ValidationConfig,
    pub intent_hash_manager: TestIntentHashManager,
}

impl LedgerTransactionValidator {
    pub fn parse_unvalidated_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<LedgerTransaction, TransactionValidationError> {
        if transaction_payload.len() > OVERRIDE_LEDGER_MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: LedgerTransaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    pub fn validate_and_create_executable<'a>(
        &self,
        prepared_transaction: &'a PreparedLedgerTransaction<'a>,
    ) -> Result<Executable<'a>, TransactionValidationError> {
        let validator = NotarizedTransactionValidator::new(self.validation_config);
        match prepared_transaction {
            PreparedLedgerTransaction::User(notarized_transaction) => {
                validator.validate(notarized_transaction, &self.intent_hash_manager)
            }
            PreparedLedgerTransaction::Validator(validator_transaction) => {
                Ok(validator_transaction.get_executable())
            }
        }
    }
}
