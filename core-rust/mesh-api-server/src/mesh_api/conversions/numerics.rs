use crate::prelude::*;
use chrono::prelude::*;
use std::ops::RangeInclusive;

use regex::Regex;

const MAX_API_EPOCH: u64 = 10000000000;
const MAX_API_ROUND: u64 = 10000000000;
const MAX_API_STATE_VERSION: u64 = 100000000000000;
const DEFAULT_MAX_PAGE_SIZE: usize = 100; // Must match the OpenAPI's `MaxPageSize.maximum`

#[tracing::instrument(skip_all)]
pub fn to_api_epoch(mapping_context: &MappingContext, epoch: Epoch) -> Result<i64, MappingError> {
    if epoch.number() > MAX_API_EPOCH {
        if mapping_context.uncommitted_data {
            // If we're mapping uncommitted data, then it's possible that the epoch is purposefully invalid.
            // So saturate to MAX_API_EPOCH in this case.
            return Ok(MAX_API_EPOCH
                .try_into()
                .expect("Max epoch too large somehow"));
        }
        return Err(MappingError::IntegerError {
            message: "Epoch larger than max api epoch".to_owned(),
        });
    }
    Ok(epoch.number().try_into().expect("Epoch too large somehow"))
}

#[tracing::instrument(skip_all)]
pub fn to_api_round(round: Round) -> Result<i64, MappingError> {
    if round.number() > MAX_API_ROUND {
        return Err(MappingError::IntegerError {
            message: "Round larger than max api round".to_owned(),
        });
    }
    Ok(round.number().try_into().expect("Round too large somehow"))
}

#[tracing::instrument(skip_all)]
pub fn to_api_state_version(state_version: StateVersion) -> Result<i64, MappingError> {
    let state_version_number = state_version.number();
    if state_version_number > MAX_API_STATE_VERSION {
        return Err(MappingError::IntegerError {
            message: "State version larger than max api state version".to_owned(),
        });
    }
    Ok(state_version_number
        .try_into()
        .expect("State version too large somehow"))
}

pub fn extract_state_version(state_version_number: i64) -> Result<StateVersion, ExtractionError> {
    if !(0..=MAX_API_STATE_VERSION as i64).contains(&state_version_number) {
        return Err(ExtractionError::InvalidInteger {
            message: format!(
                "State version must be within [{}, {}]",
                0, MAX_API_STATE_VERSION
            ),
        });
    }
    Ok(StateVersion::of(
        state_version_number
            .try_into()
            .expect("already validated above"),
    ))
}

pub fn extract_blueprint_version(string: &str) -> Result<BlueprintVersion, ExtractionError> {
    let semver_parts = Regex::new(r"^(\d+)\.(\d+)\.(\d+)$")
        .ok()
        .and_then(|regex| regex.captures(string))
        .map(|captures| captures.extract::<3>().1)
        .ok_or(ExtractionError::InvalidSemverString)?;
    Ok(BlueprintVersion {
        major: u32::from_str(semver_parts[0]).map_err(|_| ExtractionError::InvalidSemverString)?,
        minor: u32::from_str(semver_parts[1]).map_err(|_| ExtractionError::InvalidSemverString)?,
        patch: u32::from_str(semver_parts[2]).map_err(|_| ExtractionError::InvalidSemverString)?,
    })
}

pub fn to_api_decimal(value: &Decimal) -> String {
    value.to_string()
}

pub fn to_api_u8_as_i32(input: u8) -> i32 {
    input.into()
}

pub fn to_api_u32_as_i64(input: u32) -> i64 {
    input.into()
}

pub fn to_api_i32_as_i64(input: i32) -> i64 {
    input.into()
}

pub fn to_api_index_as_i64(index: usize) -> Result<i64, MappingError> {
    index.try_into().map_err(|_| MappingError::IntegerError {
        message: "Index number too large".to_string(),
    })
}

pub fn to_api_u64_as_string(input: u64) -> String {
    input.to_string()
}

pub fn to_api_i64_as_string(input: i64) -> String {
    input.to_string()
}

/// A range of valid years accepted by the basic ISO 8601 (i.e. without extensions).
///
/// For those curious:
/// - The beginning of this range is the start of a Gregorian calendar.
/// - The end is simply the maximum fitting within 4 characters.
const ISO_8601_YEAR_RANGE: RangeInclusive<i32> = 1583..=9999;

fn to_canonical_rfc3339_string(date_time: NaiveDateTime) -> String {
    DateTime::<Utc>::from_utc(date_time, Utc).to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn to_second_precision_rfc3339_string(date_time: NaiveDateTime) -> String {
    DateTime::<Utc>::from_utc(date_time, Utc).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn extract_max_page_size(max_page_size: Option<i32>) -> Result<usize, ExtractionError> {
    let Some(max_page_size) = max_page_size else {
        return Ok(DEFAULT_MAX_PAGE_SIZE);
    };
    if max_page_size <= 0 {
        return Err(ExtractionError::InvalidInteger {
            message: "Max page size must be positive".to_owned(),
        });
    }
    if max_page_size > DEFAULT_MAX_PAGE_SIZE as i32 {
        return Err(ExtractionError::InvalidInteger {
            message: "Max page size too large".to_owned(),
        });
    }
    Ok(usize::try_from(max_page_size).expect("bounds checked already"))
}

pub fn extract_u8_from_api_i32(input: i32) -> Result<u8, ExtractionError> {
    if input < 0 {
        return Err(ExtractionError::InvalidInteger {
            message: "Is negative".to_owned(),
        });
    }
    if input > (u8::MAX as i32) {
        return Err(ExtractionError::InvalidInteger {
            message: "Is larger than the max value allowed".to_owned(),
        });
    }
    Ok(input.try_into().expect("Number invalid somehow"))
}
