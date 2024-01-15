use radix_engine_common::address::AddressBech32EncodeError;
use radix_engine_interface::data::scrypto::model::ParseNonFungibleLocalIdError;
use sbor::{DecodeError, EncodeError};
use tracing::warn;
use transaction::errors::TransactionValidationError;

use crate::browse_api::*;

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
    IntegerError {
        message: String,
    },
}

impl From<MappingError> for ResponseError {
    fn from(mapping_error: MappingError) -> Self {
        warn!(?mapping_error, "Error mapping response on Browse API");
        server_error("Server error mapping response")
    }
}

/// Should be used when extracting values from a client request
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum ExtractionError {
    InvalidInteger { message: String },
    InvalidHex,
    InvalidHash,
    InvalidSbor(DecodeError),
    InvalidTransaction(TransactionValidationError),
    InvalidAddress,
    InvalidNonFungibleId(ParseNonFungibleLocalIdError),
    InvalidFieldAlternativesUsage,
    InvalidSemverString,
    InvalidProgrammaticJson { message: String },
}

impl ExtractionError {
    pub(crate) fn into_response_error(self, field_name: &str) -> ResponseError {
        client_error(
            format!("Error extracting {field_name} from request: {self:?}"),
            models::ErrorDetails::InvalidRequestDetails {},
        )
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

pub fn to_api_requested_item_type(item_kind: ItemKind) -> models::RequestedItemType {
    match item_kind {
        ItemKind::Blueprint => models::RequestedItemType::Blueprint,
        ItemKind::Schema => models::RequestedItemType::Schema,
        ItemKind::Entity => models::RequestedItemType::Entity,
        ItemKind::Module => models::RequestedItemType::Module,
        ItemKind::Field => models::RequestedItemType::Field,
        ItemKind::Collection => models::RequestedItemType::Collection,
        ItemKind::EntryKey => models::RequestedItemType::EntryKey,
    }
}
