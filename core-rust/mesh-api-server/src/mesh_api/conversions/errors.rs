use crate::engine_prelude::*;
use crate::mesh_api::*;

#[derive(Debug, Clone)]
#[allow(unused)] // Debug is ignored for dead code analysis, but is used in the error messages
pub enum MappingError {
    InvalidTransactionHash {
        encode_error: TransactionHashBech32EncodeError,
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
    KeyValueStoreEntryUnexpectedlyAbsent,
    ProofNotFound,
    InvalidResource {
        message: String,
    },
    InvalidAccount {
        message: String,
    },
    InvalidTransactionIdentifier {
        message: String,
    },
    /// An error occurring when the contents of some Node-maintained index table do not match the
    /// Engine-owned data (most likely due to a bug on either side).
    InternalIndexDataMismatch {
        message: String,
    },
    TransactionNotFound,
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
    InvalidInteger { message: String },
    InvalidHash,
    InvalidAddress,
    InvalidAccount { message: String },
    InvalidBlockIdentifier { message: String },
    InvalidCurrency { message: String },
    NotFound,
    InvalidCurveType(models::CurveType),
    InvalidSecp256k1PublicKey(String),
    InvalidEd25519PublicKey(String),
    InvalidSignatureType(models::SignatureType),
    InvalidSecp256k1Signature(String),
    InvalidEd25519Signature(String),
    InvalidAmount(models::Amount),
}

impl ExtractionError {
    pub(crate) fn into_response_error(self, field_name: &str) -> ResponseError {
        match self {
            ExtractionError::InvalidBlockIdentifier { message } => {
                ResponseError::from(ApiError::InvalidBlockIdentifier).with_details(message)
            }
            ExtractionError::InvalidAccount { message } => {
                ResponseError::from(ApiError::InvalidAccount).with_details(message)
            }
            ExtractionError::InvalidCurrency { message } => {
                ResponseError::from(ApiError::InvalidCurrency).with_details(message)
            }
            _ => ResponseError::from(ApiError::InvalidRequest).with_details(format!(
                "Could not extract {field_name} from request, {:?}",
                self
            )),
        }
    }
}
