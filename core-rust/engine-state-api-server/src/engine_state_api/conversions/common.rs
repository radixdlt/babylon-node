use crate::engine_prelude::*;
use itertools::Either;

use state_manager::{LedgerStateSummary, StateVersion};

use crate::engine_state_api::*;

#[tracing::instrument(skip_all)]
pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Vec<u8>, ExtractionError> {
    hex::decode(v).map_err(|_| ExtractionError::InvalidHex)
}

pub fn to_api_ledger_state_summary(
    mapping_context: &MappingContext,
    state_summary: &LedgerStateSummary,
) -> Result<models::LedgerStateSummary, MappingError> {
    Ok(models::LedgerStateSummary {
        state_version: to_api_state_version(state_summary.state_version)?,
        header_summary: Box::new(to_api_ledger_header_summary(
            mapping_context,
            state_summary,
        )?),
    })
}

pub fn to_api_ledger_header_summary(
    mapping_context: &MappingContext,
    state_summary: &LedgerStateSummary,
) -> Result<models::LedgerHeaderSummary, MappingError> {
    let hashes = &state_summary.hashes;
    Ok(models::LedgerHeaderSummary {
        epoch_round: Box::new(to_api_epoch_round(mapping_context, state_summary)?),
        ledger_hashes: Box::new(models::LedgerHashes {
            state_tree_hash: to_api_state_tree_hash(&hashes.state_root),
            transaction_tree_hash: to_api_transaction_tree_hash(&hashes.transaction_root),
            receipt_tree_hash: to_api_receipt_tree_hash(&hashes.receipt_root),
        }),
        proposer_timestamp: Box::new(to_api_consensus_instant(
            state_summary.proposer_timestamp_ms,
        )?),
    })
}

pub fn to_api_epoch_round(
    context: &MappingContext,
    ledger_header: &LedgerStateSummary,
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
    Ok(match exactly_one_of("name", name, "index", index)? {
        Either::Left(name) => RichIndexInput::Name(name),
        Either::Right(index) => RichIndexInput::Index(extract_api_u8_as_i32(index)?),
    })
}

pub fn extract_api_sbor_data(
    context: &ExtractionContext,
    sbor_data: models::SborData,
) -> Result<ScryptoValue, ExtractionError> {
    let models::SborData {
        raw_hex,
        programmatic_json,
    } = sbor_data;
    match exactly_one_of("raw_hex", raw_hex, "programmatic_json", programmatic_json)? {
        Either::Left(raw_hex) => {
            scrypto_decode(&from_hex(raw_hex)?).map_err(ExtractionError::InvalidSbor)
        }
        Either::Right(value) => ProgrammaticJsonDecoder::new(context).decode(value),
    }
}

pub fn to_api_sbor_data(
    context: &MappingContext,
    sbor_data: SborData,
) -> Result<models::SborData, MappingError> {
    Ok(models::SborData {
        raw_hex: if context.sbor_options.include_raw_hex {
            Some(to_hex(sbor_data.as_bytes()))
        } else {
            None
        },
        programmatic_json: if context.sbor_options.include_programmatic_json {
            Some(sbor_data.into_programmatic_json(context)?)
        } else {
            None
        },
    })
}

pub fn to_api_public_key(public_key: &PublicKey) -> models::PublicKey {
    match public_key {
        PublicKey::Secp256k1(key) => models::PublicKey::EcdsaSecp256k1PublicKey {
            key_hex: to_hex(key.to_vec()),
        },
        PublicKey::Ed25519(key) => models::PublicKey::EddsaEd25519PublicKey {
            key_hex: to_hex(key.to_vec()),
        },
    }
}

pub fn to_api_url(url: UncheckedUrl) -> String {
    url.0
}

pub fn to_api_origin(origin: UncheckedOrigin) -> String {
    origin.0
}

pub fn to_api_royalty_amount(royalty_amount: &RoyaltyAmount) -> Option<models::RoyaltyAmount> {
    match royalty_amount {
        RoyaltyAmount::Free => None,
        RoyaltyAmount::Xrd(amount) => Some(models::RoyaltyAmount {
            amount: to_api_decimal(amount),
            unit: models::royalty_amount::Unit::XRD,
        }),
        RoyaltyAmount::Usd(amount) => Some(models::RoyaltyAmount {
            amount: to_api_decimal(amount),
            unit: models::royalty_amount::Unit::USD,
        }),
    }
}

// Note: we currently only support version-based (and always optional) `at_ledger_state` parameter,
// hence this convenience method wins.
pub fn extract_opt_ledger_state_selector(
    selector: Option<&models::LedgerStateSelector>,
) -> Result<Option<StateVersion>, ExtractionError> {
    selector.map(extract_ledger_state_selector).transpose()
}

pub fn extract_ledger_state_selector(
    selector: &models::LedgerStateSelector,
) -> Result<StateVersion, ExtractionError> {
    Ok(match selector {
        models::LedgerStateSelector::VersionLedgerStateSelector { state_version } => {
            extract_state_version(*state_version)?
        }
    })
}

/// An input specification of a [`RichIndex`] (a number outputted together with an optional name).
/// Such index may be unambiguously specified either by a number or by a name.
/// See [`extract_api_rich_index_input()`].
pub enum RichIndexInput {
    Name(String),
    Index(u8),
}

fn exactly_one_of<L, R>(
    left_name: impl Into<String>,
    left_value: Option<L>,
    right_name: impl Into<String>,
    right_value: Option<R>,
) -> Result<Either<L, R>, ExtractionError> {
    if let Some(left_value) = left_value {
        if right_value.is_some() {
            Err(ExtractionError::InvalidFieldAlternativesUsage {
                alternatives: vec![left_name.into(), right_name.into()],
                present_count: 2,
            })
        } else {
            Ok(Either::Left(left_value))
        }
    } else if let Some(right_value) = right_value {
        Ok(Either::Right(right_value))
    } else {
        Err(ExtractionError::InvalidFieldAlternativesUsage {
            alternatives: vec![left_name.into(), right_name.into()],
            present_count: 0,
        })
    }
}
