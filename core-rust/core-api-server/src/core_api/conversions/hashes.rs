use state_manager::{transaction::*, ReceiptTreeHash, StateHash, TransactionTreeHash};
use transaction::prelude::*;

use crate::core_api::*;

#[allow(dead_code)]
pub fn to_api_intent_hash(intent_hash: &IntentHash) -> String {
    to_hex(intent_hash)
}

pub fn to_api_signed_intent_hash(signatures_hash: &SignedIntentHash) -> String {
    to_hex(signatures_hash)
}

pub fn to_api_notarized_transaction_hash(payload_hash: &NotarizedTransactionHash) -> String {
    to_hex(payload_hash)
}

pub fn to_api_ledger_hash(ledger_hash: &LedgerTransactionHash) -> String {
    to_hex(ledger_hash)
}

pub fn to_api_state_tree_hash(state_tree_hash: &StateHash) -> String {
    to_hex(state_tree_hash)
}

pub fn to_api_transaction_tree_hash(transaction_tree_hash: &TransactionTreeHash) -> String {
    to_hex(transaction_tree_hash)
}

pub fn to_api_receipt_tree_hash(receipt_tree_hash: &ReceiptTreeHash) -> String {
    to_hex(receipt_tree_hash)
}

pub fn to_api_hash(hash: &Hash) -> String {
    to_hex(hash)
}

pub fn extract_intent_hash(hash_str: String) -> Result<IntentHash, ExtractionError> {
    Ok(IntentHash::from_hash(Hash(
        from_hex(hash_str)?
            .try_into()
            .map_err(|_| ExtractionError::InvalidHash)?,
    )))
}

pub fn extract_notarized_transaction_hash(
    hash_str: String,
) -> Result<NotarizedTransactionHash, ExtractionError> {
    Ok(NotarizedTransactionHash::from_hash(Hash(
        from_hex(hash_str)?
            .try_into()
            .map_err(|_| ExtractionError::InvalidHash)?,
    )))
}
