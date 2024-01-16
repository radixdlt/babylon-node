use state_manager::{ReceiptTreeHash, StateHash, TransactionTreeHash};
use transaction::prelude::*;

use crate::browse_api::*;

pub fn to_api_state_tree_hash(state_tree_hash: &StateHash) -> String {
    to_hex(state_tree_hash)
}

pub fn to_api_transaction_tree_hash(transaction_tree_hash: &TransactionTreeHash) -> String {
    to_hex(transaction_tree_hash)
}

pub fn to_api_receipt_tree_hash(receipt_tree_hash: &ReceiptTreeHash) -> String {
    to_hex(receipt_tree_hash)
}

pub fn to_api_schema_hash(hash: &SchemaHash) -> String {
    to_hex(hash)
}

pub fn extract_schema_hash(hash_str: &str) -> Result<SchemaHash, ExtractionError> {
    Hash::from_str(hash_str)
        .map(SchemaHash::from_hash)
        .map_err(|_| ExtractionError::InvalidHash)
}
