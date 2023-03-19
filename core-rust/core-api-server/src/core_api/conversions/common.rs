use radix_engine::types::{scrypto_decode, scrypto_encode, ScryptoEncode};
use radix_engine_common::data::scrypto::{ScryptoValue, SerializableScryptoValue};
use serde_json::to_value;

use crate::core_api::*;

#[tracing::instrument(skip_all)]
pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn encodable_to_api_sbor_data(
    context: &MappingContext,
    value: &impl ScryptoEncode,
) -> Result<models::SborData, MappingError> {
    scrypto_bytes_to_api_sbor_data(
        context,
        &scrypto_encode(value).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Could not encode sbor for SBOR data".to_string(),
        })?,
    )
}

pub fn scrypto_bytes_to_api_sbor_data(
    context: &MappingContext,
    scrypto_bytes: &[u8],
) -> Result<models::SborData, MappingError> {
    let scrypto_value =
        scrypto_decode::<ScryptoValue>(scrypto_bytes).map_err(|err| MappingError::InvalidSbor {
            decode_error: err,
            bytes: scrypto_bytes.to_vec(),
        })?;
    scrypto_value_to_api_sbor_data(context, scrypto_bytes, &scrypto_value)
}

pub fn scrypto_value_to_api_sbor_data(
    context: &MappingContext,
    scrypto_bytes: &[u8],
    scrypto_value: &ScryptoValue,
) -> Result<models::SborData, MappingError> {
    let json =
        to_value(scrypto_value.simple_serializable(&context.bech32_encoder)).map_err(|err| {
            MappingError::SborSerializationError {
                message: err.to_string(),
            }
        })?;
    Ok(models::SborData {
        data_hex: to_hex(scrypto_bytes),
        data_json: Some(json),
    })
}
