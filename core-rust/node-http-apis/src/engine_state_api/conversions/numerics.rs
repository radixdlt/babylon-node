use crate::engine_prelude::*;
use std::any::type_name;

use regex::Regex;
use state_manager::StateVersion;

use crate::engine_state_api::models;

use super::*;

const MAX_API_EPOCH: u64 = 10000000000;
const MAX_API_ROUND: u64 = 10000000000;
const MAX_API_STATE_VERSION: u64 = 100000000000000;
const MIN_API_TIMESTAMP_MS: i64 = 0;
const MAX_API_TIMESTAMP_MS: i64 = 100000000000000; // For comparison, current timestamp is 1673822843000 (about 1/100th the size)
const DEFAULT_MAX_PAGE_SIZE: usize = 100; // Must match the OpenAPI's `MaxPageSize.maximum`

#[tracing::instrument(skip_all)]
pub fn to_api_epoch(_mapping_context: &MappingContext, epoch: Epoch) -> Result<i64, MappingError> {
    if epoch.number() > MAX_API_EPOCH {
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

pub fn to_api_blueprint_version(
    _context: &MappingContext,
    version: &BlueprintVersion,
) -> Result<String, MappingError> {
    Ok(format!(
        "{}.{}.{}",
        version.major, version.minor, version.patch
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

pub fn to_api_instant(instant: &Instant) -> Result<models::Instant, MappingError> {
    to_api_instant_from_safe_timestamp(instant.seconds_since_unix_epoch.checked_mul(1000).ok_or(
        MappingError::IntegerError {
            message: "Timestamp must be representable as millis in i64".to_owned(),
        },
    )?)
}

pub fn to_api_instant_from_safe_timestamp(
    timestamp_millis: i64,
) -> Result<models::Instant, MappingError> {
    if !(MIN_API_TIMESTAMP_MS..=MAX_API_TIMESTAMP_MS).contains(&timestamp_millis) {
        return Err(MappingError::IntegerError {
            message: format!("Timestamp ms must be >= 0 and <= {MAX_API_TIMESTAMP_MS}"),
        });
    }
    use chrono::prelude::*;
    let date_time = NaiveDateTime::from_timestamp_millis(timestamp_millis)
        .map(|d| {
            DateTime::<Utc>::from_naive_utc_and_offset(d, Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        })
        .ok_or_else(|| MappingError::IntegerError {
            message: "Timestamp invalid when converted to DateTime".to_string(),
        })?;

    Ok(models::Instant {
        unix_timestamp_ms: timestamp_millis,
        date_time,
    })
}

pub fn extract_api_max_page_size(max_page_size: Option<i32>) -> Result<usize, ExtractionError> {
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

pub fn extract_api_u8_as_i32(input: i32) -> Result<u8, ExtractionError> {
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

pub trait PanickingOps: Sized {
    fn add_or_panic(self, rhs: Self) -> Self;
    fn sub_or_panic(self, rhs: Self) -> Self;
    fn mul_or_panic(self, rhs: Self) -> Self;
    fn div_or_panic(self, rhs: Self) -> Self;
    fn neg_or_panic(self) -> Self;
}

impl<T> PanickingOps for T
where
    T: CheckedAdd<Output = T>
        + CheckedSub<Output = T>
        + CheckedDiv<Output = T>
        + CheckedMul<Output = T>
        + CheckedNeg<Output = T>
        + Copy
        + Display,
{
    fn add_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "+", rhs, self.checked_add(rhs))
    }

    fn sub_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "-", rhs, self.checked_sub(rhs))
    }

    fn mul_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "*", rhs, self.checked_mul(rhs))
    }

    fn div_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "/", rhs, self.checked_div(rhs))
    }

    fn neg_or_panic(self) -> Self {
        if let Some(result) = self.checked_neg() {
            result
        } else {
            panic!("result of -{} does not fit in {}", self, type_name::<T>());
        }
    }
}

pub trait PanickingSumIterator<E> {
    fn sum_or_panic(self) -> E;
}

impl<T, E> PanickingSumIterator<E> for T
where
    T: Iterator<Item = E>,
    E: Default + CheckedAdd<Output = E> + Copy + Display,
{
    fn sum_or_panic(self) -> E {
        let mut result = E::default();
        for (index, element) in self.enumerate() {
            let sum = result.checked_add(element);
            if let Some(sum) = sum {
                result = sum;
            } else {
                panic!(
                    "result of accumulating {}. element ({} + {}) does not fit in {}",
                    index,
                    result,
                    element,
                    type_name::<T>()
                );
            }
        }
        result
    }
}

fn op_or_panic<T: Display>(left: T, op: &str, right: T, result: Option<T>) -> T {
    if let Some(result) = result {
        result
    } else {
        panic!(
            "result of {} {} {} does not fit in {}",
            left,
            op,
            right,
            type_name::<T>()
        );
    }
}
