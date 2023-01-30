use crate::transaction::ledger_transaction::LedgerTransaction;
use radix_engine::types::scrypto_decode;
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::modules::auth::AuthAddresses;
use transaction::errors::TransactionValidationError;
use transaction::model::{Executable, NotarizedTransaction};
use transaction::validation::ValidationConfig;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator,
};

pub struct UserTransactionValidator {
    pub validation_config: ValidationConfig,
    pub intent_hash_manager: TestIntentHashManager,
}

// TODO: consider use of radix-engine-constans::MAX_TRANSACTION_SIZE here
pub const OVERRIDE_MAX_PAYLOAD_SIZE: usize = 1024 * 1024;

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
        payload_size: usize,
    ) -> Result<Executable<'a>, TransactionValidationError> {
        let validator = NotarizedTransactionValidator::new(self.validation_config);

        validator.validate(transaction, payload_size, &self.intent_hash_manager)
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
        ledger_transaction: &'a LedgerTransaction,
    ) -> Result<Executable<'a>, TransactionValidationError> {
        let validator = NotarizedTransactionValidator::new(self.validation_config);
        match ledger_transaction {
            LedgerTransaction::User(notarized_transaction) => {
                // TODO: Remove
                let payload_size = scrypto_encode(notarized_transaction).unwrap().len();
                validator.validate(
                    notarized_transaction,
                    payload_size,
                    &self.intent_hash_manager,
                )
            }
            LedgerTransaction::Validator(validator_transaction) => {
                let prepared = validator_transaction.prepare();
                Ok(prepared.to_executable())
            }
            LedgerTransaction::System(system_transaction) => {
                Ok(system_transaction.get_executable(vec![AuthAddresses::system_role()]))
            }
        }
    }
}
