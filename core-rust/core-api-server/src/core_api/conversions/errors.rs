use radix_engine::types::AddressError;
use sbor::DecodeError;
use transaction::errors::TransactionValidationError;

use crate::core_api::{client_error, server_error, RequestHandlingError};

/// Should be used when there's an error mapping to an API response
#[derive(Debug, Clone)]
pub enum MappingError {
    VirtualRootSubstatePersisted {
        message: String,
    },
    VirtualSubstateDownedWithInvalidParent {
        message: String,
    },
    TransientSubstatePersisted {
        message: String,
    },
    InvalidRootEntity {
        message: String,
    },
    InvalidSbor {
        decode_error: DecodeError,
        bytes: Vec<u8>,
    },
    InvalidComponentStateEntities {
        message: String,
    },
    MismatchedSubstateId {
        message: String,
    },
    IntegerError {
        message: String,
    },
}

impl From<MappingError> for RequestHandlingError {
    fn from(_: MappingError) -> Self {
        // TODO - replace with warn when we have logging
        // println!("Error mapping response on Core API: {:?}", mapping_error);
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
