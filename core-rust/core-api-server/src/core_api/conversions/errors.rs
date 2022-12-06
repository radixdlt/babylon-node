use radix_engine::types::{AddressError, ParseNonFungibleIdError};
use radix_engine_interface::data::ScryptoValueDecodeError;
use sbor::{DecodeError, EncodeError};
use tracing::warn;
use transaction::errors::TransactionValidationError;

use crate::core_api::{client_error, server_error, RequestHandlingError};

/// Should be used when there's an error mapping to an API response
#[derive(Debug, Clone)]
pub enum MappingError {
    UnsupportedSubstatePersisted {
        message: String,
    },
    TransientSubstatePersisted {
        message: String,
    },
    TransientRENodePersisted {
        message: String,
    },
    ScryptoValueDecode {
        decode_error: ScryptoValueDecodeError,
        bytes: Vec<u8>,
    },
    InvalidSbor {
        decode_error: DecodeError,
        bytes: Vec<u8>,
    },
    SborEncodeError {
        encode_error: EncodeError,
        message: String,
    },
    SborSerializationError {
        message: String,
        bytes: Vec<u8>,
    },
    InvalidComponentStateEntities {
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
    NotXrdError {
        message: String,
    },
}

impl From<MappingError> for RequestHandlingError {
    fn from(mapping_error: MappingError) -> Self {
        warn!(?mapping_error, "Error mapping response on Core API");
        server_error("Server error mapping response")
    }
}

/// Should be used when extracting values from a client request
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum ExtractionError {
    InvalidInteger { message: String },
    InvalidHex,
    InvalidSignature,
    InvalidPublicKey,
    InvalidHash,
    InvalidTransaction(TransactionValidationError),
    InvalidAddress(AddressError),
    InvalidNonFungibleId(ParseNonFungibleIdError),
}

impl ExtractionError {
    pub(crate) fn into_response_error(self, field_name: &str) -> RequestHandlingError {
        client_error(&format!(
            "Error extracting {} from request: {:?}",
            field_name, self
        ))
    }
}

impl From<TransactionValidationError> for ExtractionError {
    fn from(err: TransactionValidationError) -> Self {
        ExtractionError::InvalidTransaction(err)
    }
}

impl From<ParseNonFungibleIdError> for ExtractionError {
    fn from(err: ParseNonFungibleIdError) -> Self {
        ExtractionError::InvalidNonFungibleId(err)
    }
}
