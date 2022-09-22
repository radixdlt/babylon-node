use sbor::decode_any;

use crate::core_api::*;

pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn scrypto_bytes_to_api_sbor_data(
    scrypto_bytes: &[u8],
) -> Result<models::SborData, MappingError> {
    let scrypto_value = decode_any(scrypto_bytes).map_err(|err| MappingError::InvalidSbor {
        decode_error: err,
        bytes: scrypto_bytes.to_vec(),
    })?;
    Ok(models::SborData {
        data_hex: to_hex(scrypto_bytes),
        data_json: Some(serde_json::to_value(&scrypto_value).expect("JSON serialize error")),
    })
}
