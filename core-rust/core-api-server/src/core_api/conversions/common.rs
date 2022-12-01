use radix_engine::types::{Bech32Encoder, scrypto_decode};
use radix_engine_interface::data::ScryptoValue;

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
    let scrypto_value = scrypto_decode::<ScryptoValue>(scrypto_bytes)
        .map_err(|err| MappingError::InvalidSbor {
            decode_error: err,
            bytes: scrypto_bytes.to_vec(),
        })?;
    Ok(models::SborData {
        data_hex: to_hex(scrypto_bytes),
        data_json: Some(convert_scrypto_sbor_value_to_json(
            bech32_encoder,
            &scrypto_value,
        )),
    })
}

use serde_json::Value as JsonValue;

pub fn convert_scrypto_sbor_value_to_json(
    bech32_encoder: &Bech32Encoder,
    scrypto_value: &ScryptoValue,
) -> JsonValue {
    match scrypto_value {
        sbor::SborValue::Unit => todo!(),
        sbor::SborValue::Bool { value } => todo!(),
        sbor::SborValue::I8 { value } => todo!(),
        sbor::SborValue::I16 { value } => todo!(),
        sbor::SborValue::I32 { value } => todo!(),
        sbor::SborValue::I64 { value } => todo!(),
        sbor::SborValue::I128 { value } => todo!(),
        sbor::SborValue::U8 { value } => todo!(),
        sbor::SborValue::U16 { value } => todo!(),
        sbor::SborValue::U32 { value } => todo!(),
        sbor::SborValue::U64 { value } => todo!(),
        sbor::SborValue::U128 { value } => todo!(),
        sbor::SborValue::String { value } => todo!(),
        sbor::SborValue::Struct { fields } => todo!(),
        sbor::SborValue::Enum { discriminator, fields } => todo!(),
        sbor::SborValue::Array { element_type_id, elements } => todo!(),
        sbor::SborValue::Tuple { elements } => todo!(),
        sbor::SborValue::Custom { value } => todo!(),
    }
    convert_custom_payloads_recursive(
        bech32_encoder,
        serde_json::to_value(scrypto_value).expect("JSON serialize error"),
    )
}

/// This is a slightly non-ideal implementation; copied from PTE - where we just
/// modify the Custom types after they are output from JSON encoding the any::Value.
/// We'll likely change this in future when we change SBOR JSON encoding.
fn convert_custom_payloads_recursive(
    bech32_encoder: &Bech32Encoder,
    value: JsonValue,
) -> JsonValue {
    match value {
        JsonValue::Null => JsonValue::Null,
        JsonValue::Bool(v) => JsonValue::Bool(v),
        JsonValue::Number(v) => JsonValue::Number(v),
        JsonValue::String(v) => JsonValue::String(v),
        JsonValue::Array(values) => JsonValue::Array(
            values
                .into_iter()
                .map(|e| convert_custom_payloads_recursive(bech32_encoder, e))
                .collect(),
        ),
        JsonValue::Object(fields) => {
            if fields.get("type") == Some(&JsonValue::String("Custom".into())) {
                if let Some(JsonValue::Number(type_id)) = fields.get("type_id") {
                    if let Some(JsonValue::String(bytes)) = fields.get("bytes") {
                        let type_id = type_id.as_u64().unwrap() as u8;
                        let bytes = hex::decode(bytes).unwrap();
                        let type_name = {
                            let mut buf = String::new();
                            ScryptoValueFormatter::format_type_id(&mut buf, type_id)
                                .expect("Could not format type id");
                            buf
                        };
                        let value = {
                            let mut buf = String::new();
                            ScryptoValueFormatter::format_custom_value(
                                &mut buf,
                                type_id,
                                &bytes,
                                &ScryptoValueFormatterContext::no_manifest_context(Some(
                                    bech32_encoder,
                                )),
                            )
                            .expect("Could not format custom value");
                            buf
                        };
                        return serde_json::json!({
                            "type": type_name,
                            "value": value
                        });
                    }
                };
            }

            JsonValue::Object(
                fields
                    .into_iter()
                    .map(|(k, v)| (k, convert_custom_payloads_recursive(bech32_encoder, v)))
                    .collect(),
            )
        }
    }
}
