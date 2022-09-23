use crate::transaction::types::Transaction;
use crate::transaction::ValidatorTransaction;
use scrypto::buffer::scrypto_decode;
use transaction::errors::TransactionValidationError;
use transaction::model::{NotarizedTransaction, Validated};
use transaction::validation::MAX_PAYLOAD_SIZE;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator,
};

pub struct UserTransactionValidator {
    pub validator: NotarizedTransactionValidator,
    pub intent_hash_manager: TestIntentHashManager,
}

impl UserTransactionValidator {
    /// Checks the Payload max size, and SBOR decodes to a NotarizedTransaction if the size is okay
    pub fn parse_unvalidated_user_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<NotarizedTransaction, TransactionValidationError> {
        if transaction_payload.len() > MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: NotarizedTransaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    /// Performs static validation only
    pub fn parse_and_validate_user_transaction_slice(
        &self,
        transaction_payload: &[u8],
    ) -> Result<Validated<NotarizedTransaction>, TransactionValidationError> {
        let notarized_transaction =
            Self::parse_unvalidated_user_transaction_from_slice(transaction_payload)?;
        self.validate_user_transaction(notarized_transaction)
    }

    /// Performs static validation only
    pub fn validate_user_transaction(
        &self,
        transaction: NotarizedTransaction,
    ) -> Result<Validated<NotarizedTransaction>, TransactionValidationError> {
        self.validator
            .validate(transaction, &self.intent_hash_manager)
    }
}

pub struct CommittedTransactionValidator {
    pub validator: NotarizedTransactionValidator,
    pub intent_hash_manager: TestIntentHashManager,
}

impl CommittedTransactionValidator {
    pub fn parse_unvalidated_transaction_from_slice(
        transaction_payload: &[u8],
    ) -> Result<Transaction, TransactionValidationError> {
        let transaction: Transaction = scrypto_decode(transaction_payload)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    pub fn parse_and_validate_transaction_slice(
        &self,
        transaction_payload: &[u8],
    ) -> Result<Validated<Transaction>, TransactionValidationError> {
        let transaction = Self::parse_unvalidated_transaction_from_slice(transaction_payload)?;
        self.validate_transaction(transaction)
    }

    fn validate_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<Validated<Transaction>, TransactionValidationError> {
        match transaction {
            Transaction::User(notarized_transaction) => self
                .validator
                .validate(notarized_transaction, &self.intent_hash_manager)
                .map(|validated| Validated {
                    transaction: Transaction::User(validated.transaction),
                    transaction_hash: validated.transaction_hash,
                    instructions: validated.instructions,
                    initial_proofs: validated.initial_proofs,
                    cost_unit_limit: validated.cost_unit_limit,
                    tip_percentage: validated.tip_percentage,
                    blobs: validated.blobs,
                }),
            Transaction::Validator(validator_transaction) => {
                let validated: Validated<ValidatorTransaction> = validator_transaction.into();
                Ok(Validated {
                    transaction: Transaction::Validator(validated.transaction),
                    transaction_hash: validated.transaction_hash,
                    instructions: validated.instructions,
                    initial_proofs: validated.initial_proofs,
                    cost_unit_limit: validated.cost_unit_limit,
                    tip_percentage: validated.tip_percentage,
                    blobs: validated.blobs,
                })
            }
        }
    }
}
