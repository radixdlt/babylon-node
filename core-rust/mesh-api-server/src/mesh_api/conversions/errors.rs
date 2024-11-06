use crate::engine_prelude::*;
use crate::mesh_api::*;

/// Should be used when there's an error mapping to an API response
#[derive(Debug, Clone)]
pub enum MappingError {
    SubstateValue {
        bytes: Vec<u8>,
        message: String,
    },
    EntityTypeError,
    SborEncodeError {
        encode_error: EncodeError,
        message: String,
    },
    InvalidEntityAddress {
        encode_error: AddressBech32EncodeError,
    },
    MismatchedSubstateId {
        message: String,
    },
    IntegerError {
        message: String,
    },
    ProofNotFound,
    InvalidResource {
        message: String,
    },
}

impl From<MappingError> for ResponseError {
    fn from(mapping_error: MappingError) -> Self {
        ResponseError::from(ApiError::ResponseRenderingError)
            .with_details(format!("{:?}", mapping_error))
    }
}

/// Should be used when extracting values from a client request
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
#[allow(unused)] // Fields are used in Debug implementations, but it's not enough to satisfy the lint
pub enum ExtractionError {
    InvalidInteger {
        message: String,
    },
    InvalidHex,
    InvalidHash,
    InvalidSbor(DecodeError),
    InvalidTransaction(TransactionValidationError),
    InvalidAddress,
    InvalidNonFungibleId(ParseNonFungibleLocalIdError),
    InvalidFieldAlternativesUsage {
        alternatives: Vec<String>,
        present_count: usize,
    },
    InvalidSemverString,
    InvalidProgrammaticJson {
        message: String,
    },
    DifferentFilterAcrossPages,
    InvalidAccount {
        message: String,
    },
    InvalidBlockIdentifier {
        message: String,
    },
    InvalidCurrency {
        message: String,
    },
}

impl ExtractionError {
    pub(crate) fn into_response_error(self, field_name: &str) -> ResponseError {
        // TODO make sure ExtractionError map to more adequate ApiError
        // variant than ApiError::InvalidRequest
        ResponseError::from(ApiError::InvalidRequest)
            .with_details(format!("Could not extract {field_name} from request"))
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
