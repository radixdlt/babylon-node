use crate::prelude::*;

pub fn to_api_hash_bech32m<T: IsTransactionHash>(
    context: &MappingContext,
    hash: &T,
) -> Result<String, MappingError> {
    context
        .transaction_hash_encoder
        .encode(hash)
        .map_err(|err| MappingError::InvalidTransactionHash { encode_error: err })
}
