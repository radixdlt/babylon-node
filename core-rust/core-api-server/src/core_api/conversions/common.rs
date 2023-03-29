use radix_engine::types::{scrypto_encode, ScryptoCustomTypeExtension, ScryptoEncode};
use sbor::serde_serialization::{
    SborPayloadWithoutSchema, SchemalessSerializationContext, SerializationMode,
};
use serde_json::to_value;
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
        SborPayloadWithoutSchema::<ScryptoCustomTypeExtension>::new(scrypto_sbor_bytes)
            .serializable(SchemalessSerializationContext {
                mode: SerializationMode::Invertible,
                custom_context: (&context.bech32_encoder).into(),
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
