use crate::prelude::*;

pub fn extract_transaction_intent_hash(
    context: &ExtractionContext,
    hash_str: String,
) -> Result<TransactionIntentHash, ExtractionError> {
    hex::decode(&hash_str)
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

pub fn to_api_transaction_hash_bech32m<T: IsTransactionHash>(
    context: &MappingContext,
    hash: &T,
) -> Result<String, MappingError> {
    context
        .transaction_hash_encoder
        .encode(hash)
        .map_err(|err| MappingError::InvalidTransactionHash { encode_error: err })
}

pub fn to_mesh_api_transaction_identifier_from_hash(hash: String) -> models::TransactionIdentifier {
    models::TransactionIdentifier { hash }
}

pub fn to_mesh_api_transaction_identifier(
    mapping_context: &MappingContext,
    transaction_identifiers: &CommittedTransactionIdentifiers,
) -> Result<models::TransactionIdentifier, MappingError> {
    let transaction_identifier = match transaction_identifiers.transaction_hashes.as_user() {
        // Unfortunately non-user transactions don't have txid, let's use ledger_transaction_hash as
        // transaction_identifier.
        // In case if needed to map it to state version, it is possible using
        // `get_txn_state_version_by_identifier()`
        None => to_api_transaction_hash_bech32m(
            mapping_context,
            &transaction_identifiers
                .transaction_hashes
                .ledger_transaction_hash,
        )?,
        Some(user_hashes) => {
            to_api_transaction_hash_bech32m(mapping_context, &user_hashes.transaction_intent_hash)?
        }
    };

    Ok(to_mesh_api_transaction_identifier_from_hash(
        transaction_identifier,
    ))
}
