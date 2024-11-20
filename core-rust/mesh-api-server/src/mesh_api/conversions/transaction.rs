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
    state_version: StateVersion,
) -> Result<models::TransactionIdentifier, MappingError> {
    let transaction_identifier = match transaction_identifiers.transaction_hashes.as_user() {
        // Unfortunately non-user transactions don't have txid, let's use state_version as
        // transaction_identifier.
        // TODO:MESH Perhaps we should use ledger_transaction_hash in this case?
        // I believe it can also be bech32 encoded; and we have an index for
        // ledger hash => state version which can be used to resolve the
        // state version if we need to do an arbitrary transaction read in any endpoint.
        None => format!("state_version_{}", state_version),
        Some(user_hashes) => {
            to_api_transaction_hash_bech32m(mapping_context, &user_hashes.transaction_intent_hash)?
        }
    };

    Ok(to_mesh_api_transaction_identifier_from_hash(
        transaction_identifier,
    ))
}
