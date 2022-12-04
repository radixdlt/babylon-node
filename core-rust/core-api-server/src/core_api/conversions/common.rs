use radix_engine::types::{scrypto_decode, Bech32Encoder};
use radix_engine_interface::data::{ScryptoValue, SerializableScryptoValue};
use serde_json::to_value;

use crate::core_api::*;

#[tracing::instrument(skip_all)]
pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn scrypto_bytes_to_api_sbor_data(
    bech32_encoder: &Bech32Encoder,
    scrypto_bytes: &[u8],
) -> Result<models::SborData, MappingError> {
    let scrypto_value =
        scrypto_decode::<ScryptoValue>(scrypto_bytes).map_err(|err| MappingError::InvalidSbor {
            decode_error: err,
            bytes: scrypto_bytes.to_vec(),
        })?;
    scrypto_value_to_api_sbor_data(bech32_encoder, scrypto_bytes, &scrypto_value)
}

pub fn scrypto_value_to_api_sbor_data(
    bech32_encoder: &Bech32Encoder,
    scrypto_bytes: &[u8],
    scrypto_value: &ScryptoValue,
) -> Result<models::SborData, MappingError> {
    let json = to_value(scrypto_value.simple_serializable(bech32_encoder)).map_err(|err| {
        MappingError::SborSerializationError {
            message: err.to_string(),
            bytes: scrypto_bytes.to_vec(),
        }
    })?;
    Ok(models::SborData {
        data_hex: to_hex(scrypto_bytes),
        data_json: Some(json),
    })
}
