use models::{
    lts_transaction_submit_error_details::LtsTransactionSubmitErrorDetails,
    transaction_submit_error_details::TransactionSubmitErrorDetails,
};
use radix_engine::types::{scrypto_encode, ScryptoCustomTypeExtension, ScryptoEncode};
use sbor::serde_serialization::{
    SborPayloadWithoutSchema, SchemalessSerializationContext, SerializationMode,
};
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

impl ErrorDetails for TransactionSubmitErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::TransactionSubmitErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

impl ErrorDetails for LtsTransactionSubmitErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::LtsTransactionSubmitErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
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
