use radix_engine::types::*;

use sbor::representations::*;
use state_manager::LedgerHeader;

use crate::browse_api::*;

#[tracing::instrument(skip_all)]
pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn to_api_ledger_state_summary(
    mapping_context: &MappingContext,
    header: &LedgerHeader,
) -> Result<models::LedgerStateSummary, MappingError> {
    Ok(models::LedgerStateSummary {
        state_version: to_api_state_version(header.state_version)?,
        header_summary: Box::new(to_api_ledger_header_summary(mapping_context, header)?),
    })
}

pub fn to_api_ledger_header_summary(
    mapping_context: &MappingContext,
    header: &LedgerHeader,
) -> Result<models::LedgerHeaderSummary, MappingError> {
    let hashes = &header.hashes;
    Ok(models::LedgerHeaderSummary {
        epoch_round: Box::new(to_api_epoch_round(mapping_context, header)?),
        ledger_hashes: Box::new(models::LedgerHashes {
            state_tree_hash: to_api_state_tree_hash(&hashes.state_root),
            transaction_tree_hash: to_api_transaction_tree_hash(&hashes.transaction_root),
            receipt_tree_hash: to_api_receipt_tree_hash(&hashes.receipt_root),
        }),
        proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(
            header.proposer_timestamp_ms,
        )?),
    })
}

pub fn to_api_epoch_round(
    context: &MappingContext,
    ledger_header: &LedgerHeader,
) -> Result<models::EpochRound, MappingError> {
    Ok(models::EpochRound {
        epoch: to_api_epoch(context, ledger_header.epoch)?,
        round: to_api_round(ledger_header.round)?,
    })
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

pub fn extract_api_sbor_hex_string<T: ScryptoDecode>(
    sbor_hex_string: &String,
) -> Result<T, ExtractionError> {
    let sbor_bytes = from_hex(sbor_hex_string)?;
    scrypto_decode(&sbor_bytes).map_err(ExtractionError::InvalidSbor)
}

pub fn extract_api_rich_index_input(
    name: Option<String>,
    index: Option<i32>,
) -> Result<RichIndexInput, ExtractionError> {
    if let Some(name) = name {
        if index.is_some() {
            Err(ExtractionError::InvalidFieldAlternativesUsage)
        } else {
            Ok(RichIndexInput::Name(name))
        }
    } else if let Some(index) = index {
        Ok(RichIndexInput::Index(extract_api_u8_as_i32(index)?))
    } else {
        Err(ExtractionError::InvalidFieldAlternativesUsage)
    }
}

/// An input specification of a [`RichIndex`] (a number outputted together with an optional name).
/// Such index may be unambiguously specified either by a number or by a name.
/// See [`extract_api_rich_index_input()`].
pub enum RichIndexInput {
    Name(String),
    Index(u8),
}
