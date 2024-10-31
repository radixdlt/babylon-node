use crate::prelude::*;

#[tracing::instrument(skip_all)]
pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn extract_from_sbor_hex_string<T: ScryptoDecode>(
    sbor_hex_string: &String,
) -> Result<T, ExtractionError> {
    let sbor_bytes = from_hex(sbor_hex_string)?;
    scrypto_decode(&sbor_bytes).map_err(ExtractionError::InvalidSbor)
}

pub fn to_api_sbor_hex_string<T: ScryptoEncode>(
    sbor_encodable: &T,
) -> Result<String, MappingError> {
    let sbor_bytes =
        scrypto_encode(sbor_encodable).map_err(|error| MappingError::SborEncodeError {
            encode_error: error,
            message: "while rendering sbor hex string".to_string(),
        })?;
    Ok(to_hex(sbor_bytes))
}
