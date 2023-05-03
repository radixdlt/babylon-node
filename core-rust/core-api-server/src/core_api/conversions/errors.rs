use radix_engine::types::NonFungibleIdType;
use radix_engine_interface::data::scrypto::model::ParseNonFungibleLocalIdError;
use sbor::{DecodeError, EncodeError};
use tracing::warn;
use transaction::errors::TransactionValidationError;

use crate::core_api::*;

/// Should be used when there's an error mapping to an API response
#[derive(Debug, Clone)]
pub enum MappingError {
    SubstateKeyMappingError {
        entity_type_hex: String,
        module_id: u8,
        substate_key_hex: String,
        message: String,
    },
    SubstateValueMappingError {
        bytes: Vec<u8>,
        message: String,
    },
    EntityTypeError,
    ModuleTypeError {
        message: String,
    },
    ScryptoValueDecode {
        decode_error: DecodeError,
        bytes: Vec<u8>,
    },
    InvalidSbor {
        decode_error: String,
        bytes: Vec<u8>,
    },
    SborEncodeError {
        encode_error: EncodeError,
        message: String,
    },
    InvalidManifest {
        message: String,
    },
    MismatchedSubstateId {
        message: String,
    },
    IntegerError {
        message: String,
    },
}

impl<E: ErrorDetails> From<MappingError> for ResponseError<E> {
    fn from(mapping_error: MappingError) -> Self {
        warn!(?mapping_error, "Error mapping response on Core API");
        server_error("Server error mapping response")
    }
}

/// Should be used when extracting values from a client request
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum ExtractionError {
    InvalidInteger {
        message: String,
    },
    InvalidHex,
    InvalidSignature,
    InvalidPublicKey,
    InvalidHash,
    InvalidTransaction(TransactionValidationError),
    InvalidAddress,
    InvalidNonFungibleId(ParseNonFungibleLocalIdError),
    WrongNonFungibleIdType {
        expected: NonFungibleIdType,
        actual: NonFungibleIdType,
    },
}

impl ExtractionError {
    pub(crate) fn into_response_error<E: ErrorDetails>(self, field_name: &str) -> ResponseError<E> {
        client_error(format!(
            "Error extracting {field_name} from request: {self:?}"
        ))
    }
}

impl From<TransactionValidationError> for ExtractionError {
    fn from(err: TransactionValidationError) -> Self {
        ExtractionError::InvalidTransaction(err)
    }
}

impl From<ParseNonFungibleLocalIdError> for ExtractionError {
    fn from(err: ParseNonFungibleLocalIdError) -> Self {
        ExtractionError::InvalidNonFungibleId(err)
    }
}
