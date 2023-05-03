use radix_engine::types::{scrypto_encode, ScryptoEncode};
use radix_engine_common::data::scrypto::{ScryptoRawPayload, ScryptoValueDisplayContext};
use sbor::representations::{SerializationMode, SerializationParameters};
use serde_json::to_value;

use state_manager::transaction::UserTransactionValidator;
use transaction::model::NotarizedTransaction;
use utils::ContextualSerialize;

use crate::core_api::*;

#[tracing::instrument(skip_all)]
pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn to_api_sbor_data_from_encodable(
    context: &MappingContext,
    value: &impl ScryptoEncode,
) -> Result<models::SborData, MappingError> {
    to_api_sbor_data_from_bytes(
        context,
        &scrypto_encode(value).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Could not encode sbor for SBOR data".to_string(),
        })?,
    )
}

pub fn to_api_sbor_data_from_bytes(
    context: &MappingContext,
    scrypto_sbor_bytes: &[u8],
) -> Result<models::SborData, MappingError> {
    let json = to_value(
        ScryptoRawPayload::new_from_valid_slice_with_checks(scrypto_sbor_bytes)
            .unwrap()
            .serializable(SerializationParameters::Schemaless {
                mode: SerializationMode::Programmatic,
                custom_context: ScryptoValueDisplayContext::with_optional_bech32(Some(
                    &context.bech32_encoder,
                )),
            }),
    )
    .map_err(|err| MappingError::InvalidSbor {
        decode_error: err.to_string(),
        bytes: scrypto_sbor_bytes.to_vec(),
    })?;
    Ok(models::SborData::new(
        to_hex(scrypto_sbor_bytes),
        Some(json),
    ))
}

pub fn extract_unvalidated_transaction(
    payload: &str,
) -> Result<NotarizedTransaction, ExtractionError> {
    let transaction_bytes = from_hex(payload)?;
    let notarized_transaction =
        UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
            &transaction_bytes,
        )?;
    Ok(notarized_transaction)
}
