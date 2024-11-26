use crate::prelude::*;

pub fn to_api_transaction_intent_hash(transaction_intent_hash: &TransactionIntentHash) -> String {
    to_hex(transaction_intent_hash)
}

pub fn to_api_signed_transaction_intent_hash(
    signed_transaction_intent_hash: &SignedTransactionIntentHash,
) -> String {
    to_hex(signed_transaction_intent_hash)
}

pub fn to_api_notarized_transaction_hash(
    notarized_transaction_hash: &NotarizedTransactionHash,
) -> String {
    to_hex(notarized_transaction_hash)
}

pub fn to_api_subintent_hash(subintent_hash: &SubintentHash) -> String {
    to_hex(subintent_hash)
}

pub fn to_api_ledger_hash(ledger_hash: &LedgerTransactionHash) -> String {
    to_hex(ledger_hash)
}

pub fn to_api_hash_bech32m<T: IsTransactionHash>(
    context: &MappingContext,
    hash: &T,
) -> Result<String, MappingError> {
    context
        .transaction_hash_encoder
        .encode(hash)
        .map_err(|err| MappingError::InvalidTransactionHash { encode_error: err })
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

pub fn to_api_schema_hash(hash: &SchemaHash) -> String {
    to_hex(hash)
}

pub fn to_api_code_hash(hash: &CodeHash) -> String {
    to_hex(hash)
}

pub fn extract_transaction_intent_hash(
    context: &ExtractionContext,
    hash_str: String,
) -> Result<TransactionIntentHash, ExtractionError> {
    from_hex(&hash_str)
        .ok()
        .and_then(|bytes| Hash::try_from(bytes.as_slice()).ok())
        .map(TransactionIntentHash::from_hash)
        .or_else(|| {
            context
                .transaction_hash_decoder
                .validate_and_decode(&hash_str)
                .ok()
        })
        .ok_or(ExtractionError::InvalidHash)
}

pub fn extract_notarized_transaction_hash(
    context: &ExtractionContext,
    hash_str: String,
) -> Result<NotarizedTransactionHash, ExtractionError> {
    from_hex(&hash_str)
        .ok()
        .and_then(|bytes| Hash::try_from(bytes.as_slice()).ok())
        .map(NotarizedTransactionHash::from_hash)
        .or_else(|| {
            context
                .transaction_hash_decoder
                .validate_and_decode(&hash_str)
                .ok()
        })
        .ok_or(ExtractionError::InvalidHash)
}
