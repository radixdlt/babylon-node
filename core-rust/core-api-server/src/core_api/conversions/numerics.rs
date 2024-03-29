use std::any::type_name;

use radix_engine_common::math::*;
use radix_engine_interface::blueprints::package::BlueprintVersion;
use radix_engine_interface::prelude::*;
use sbor::WellKnownTypeId;
use state_manager::store::traits::scenario::ScenarioSequenceNumber;
use state_manager::StateVersion;

use crate::core_api::models;

use super::*;

// See the guidance in the top of the core-api-schema.yaml
// These should be duplicated in the Open API schema along with the actual types
// If mapped to i32, these need to be below 4294967295 = 2^32 - 1 to ensure they fit into an i64 on the OAS.
// If mapped to i64, these need to be below 9223372036854775807 = 2^63 - 1 to ensure they fit into an i64 on the OAS.

const MAX_API_EPOCH: u64 = 10000000000;
const MAX_API_ROUND: u64 = 10000000000;
const MAX_API_STATE_VERSION: u64 = 100000000000000;
const MIN_API_TIMESTAMP_MS: i64 = 0;
const MAX_API_TIMESTAMP_MS: i64 = 100000000000000; // For comparison, current timestamp is 1673822843000 (about 1/100th the size)
const MAX_API_GENESIS_SCENARIO_NUMBER: i32 = 1000000;
const TEN_TRILLION: u64 = 10000000000;

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

pub fn to_api_active_validator_index(index: ValidatorIndex) -> models::ActiveValidatorIndex {
    models::ActiveValidatorIndex {
        index: index as i32,
    }
}

pub fn to_api_well_known_type_id(
    well_known_type_id: &WellKnownTypeId,
) -> Result<i64, MappingError> {
    well_known_type_id
        .as_index()
        .try_into()
        .map_err(|_| MappingError::IntegerError {
            message: "Well-known type index too large".to_string(),
        })
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

#[tracing::instrument(skip_all)]
pub fn to_api_ten_trillion_capped_u64(
    num: u64,
    descriptor: &'static str,
) -> Result<i64, MappingError> {
    if num > TEN_TRILLION {
        return Err(MappingError::IntegerError {
            message: format!("{descriptor} larger than {TEN_TRILLION}"),
        });
    }
    Ok(num.try_into().expect("Too large somehow"))
}

pub fn to_api_decimal(value: &Decimal) -> String {
    value.to_string()
}

pub fn to_api_u8_as_i32(input: u8) -> i32 {
    input.into()
}

pub fn to_api_u16_as_i32(input: u16) -> i32 {
    input.into()
}

pub fn to_api_u32_as_i64(input: u32) -> i64 {
    input.into()
}

pub fn to_api_index_as_i64(index: usize) -> Result<i64, MappingError> {
    index.try_into().map_err(|_| MappingError::IntegerError {
        message: "Index number too large".to_string(),
    })
}

pub fn to_api_scenario_number(number: ScenarioSequenceNumber) -> Result<i32, MappingError> {
    if number > MAX_API_GENESIS_SCENARIO_NUMBER as u32 {
        return Err(MappingError::IntegerError {
            message: format!(
                "Genesis scenario sequence number must be <= {MAX_API_GENESIS_SCENARIO_NUMBER}"
            ),
        });
    }
    Ok(number as i32)
}

#[allow(dead_code)]
pub fn to_api_u64_as_string(input: u64) -> String {
    input.to_string()
}

pub fn to_unix_timestamp_ms(time: std::time::SystemTime) -> Result<i64, MappingError> {
    let millis = time
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after Unix epoch")
        .as_millis();

    millis.try_into().map_err(|_| MappingError::IntegerError {
        message: format!("Timestamp ms must be <= {MAX_API_TIMESTAMP_MS}"),
    })
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
            DateTime::<Utc>::from_utc(d, Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        })
        .ok_or_else(|| MappingError::IntegerError {
            message: "Timestamp invalid when converted to DateTime".to_string(),
        })?;

    Ok(models::Instant {
        unix_timestamp_ms: timestamp_millis,
        date_time,
    })
}

pub fn extract_api_state_version(
    state_version_number: i64,
) -> Result<StateVersion, ExtractionError> {
    if state_version_number < 1 {
        return Err(ExtractionError::InvalidInteger {
            message: "State version must be >= 1".to_owned(),
        });
    }
    if state_version_number > MAX_API_STATE_VERSION as i64 {
        return Err(ExtractionError::InvalidInteger {
            message: format!(
                "State version is larger than the max allowed: {MAX_API_STATE_VERSION}"
            ),
        });
    }
    Ok(StateVersion::of(
        state_version_number
            .try_into()
            .expect("State version invalid somehow"),
    ))
}

pub fn extract_api_epoch(epoch: i64) -> Result<Epoch, ExtractionError> {
    if epoch < 0 {
        return Err(ExtractionError::InvalidInteger {
            message: "Epoch too low".to_owned(),
        });
    }
    if epoch > MAX_API_EPOCH as i64 {
        return Err(ExtractionError::InvalidInteger {
            message: "Epoch larger than max api epoch".to_owned(),
        });
    }
    Ok(Epoch::of(epoch.try_into().expect("Epoch invalid somehow")))
}

#[allow(dead_code)]
pub fn extract_api_u64_as_string(input: String) -> Result<u64, ExtractionError> {
    input
        .parse::<u64>()
        .map_err(|_| ExtractionError::InvalidInteger {
            message: "Is not valid u64 string".to_owned(),
        })
}

pub fn extract_api_u32_as_i64(input: i64) -> Result<u32, ExtractionError> {
    if input < 0 {
        return Err(ExtractionError::InvalidInteger {
            message: "Is negative".to_owned(),
        });
    }
    if input > (u32::MAX as i64) {
        return Err(ExtractionError::InvalidInteger {
            message: "Is larger than the max value allowed".to_owned(),
        });
    }
    Ok(input.try_into().expect("Number invalid somehow"))
}

pub fn extract_api_u16_as_i32(input: i32) -> Result<u16, ExtractionError> {
    if input < 0 {
        return Err(ExtractionError::InvalidInteger {
            message: "Is negative".to_owned(),
        });
    }
    if input > (u16::MAX as i32) {
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
