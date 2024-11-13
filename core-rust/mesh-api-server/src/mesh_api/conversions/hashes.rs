use crate::prelude::*;

pub fn to_api_transaction_hash_bech32m<T: IsTransactionHash>(
    context: &MappingContext,
    hash: &T,
) -> Result<String, MappingError> {
    context
        .transaction_hash_encoder
        .encode(hash)
        .map_err(|err| MappingError::InvalidTransactionHash { encode_error: err })
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
