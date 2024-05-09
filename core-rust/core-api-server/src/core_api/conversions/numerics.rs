use chrono::prelude::*;
use std::any::type_name;
use std::ops::RangeInclusive;

use crate::engine_prelude::*;
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
const MAX_API_EXECUTED_SCENARIO_NUMBER: i32 = 1000000;
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
    if number > MAX_API_EXECUTED_SCENARIO_NUMBER as u32 {
        return Err(MappingError::IntegerError {
            message: format!(
                "Executed scenario sequence number must be <= {MAX_API_EXECUTED_SCENARIO_NUMBER}"
            ),
        });
    }
    Ok(number as i32)
}

pub fn to_api_i64_as_string(input: i64) -> String {
    input.to_string()
}

pub fn to_api_usize_as_string(input: usize) -> String {
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

/// A range of valid years accepted by the basic ISO 8601 (i.e. without extensions).
///
/// For those curious:
/// - The beginning of this range is the start of a Gregorian calendar.
/// - The end is simply the maximum fitting within 4 characters.
const ISO_8601_YEAR_RANGE: RangeInclusive<i32> = 1583..=9999;

pub fn to_api_scrypto_instant(instant: &Instant) -> Result<models::ScryptoInstant, MappingError> {
    let timestamp_seconds = instant.seconds_since_unix_epoch;
    let date_time = NaiveDateTime::from_timestamp_opt(timestamp_seconds, 0)
        .filter(|date_time| ISO_8601_YEAR_RANGE.contains(&date_time.year()));
    Ok(models::ScryptoInstant {
        unix_timestamp_seconds: to_api_i64_as_string(timestamp_seconds),
        date_time: date_time.map(to_second_precision_rfc3339_string),
    })
}

pub fn to_api_clamped_instant_from_epoch_milli(
    timestamp_millis: i64,
) -> Result<models::InstantMs, MappingError> {
    let clamped_timestamp_millis =
        timestamp_millis.clamp(MIN_API_TIMESTAMP_MS, MAX_API_TIMESTAMP_MS);
    let date_time = NaiveDateTime::from_timestamp_millis(clamped_timestamp_millis)
        .expect("it just got clamped to 100% supported range above");
    Ok(models::InstantMs {
        unix_timestamp_ms: clamped_timestamp_millis,
        date_time: to_canonical_rfc3339_string(date_time),
    })
}

fn to_canonical_rfc3339_string(date_time: NaiveDateTime) -> String {
    DateTime::<Utc>::from_utc(date_time, Utc).to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn to_second_precision_rfc3339_string(date_time: NaiveDateTime) -> String {
    DateTime::<Utc>::from_utc(date_time, Utc).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn extract_state_version(state_version_number: i64) -> Result<StateVersion, ExtractionError> {
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

pub fn extract_epoch(epoch: i64) -> Result<Epoch, ExtractionError> {
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
pub fn extract_u64_from_api_string(input: String) -> Result<u64, ExtractionError> {
    input
        .parse::<u64>()
        .map_err(|_| ExtractionError::InvalidInteger {
            message: "Is not valid u64 string".to_owned(),
        })
}

pub fn extract_u32_from_api_i64(input: i64) -> Result<u32, ExtractionError> {
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

pub fn extract_u16_from_api_i32(input: i32) -> Result<u16, ExtractionError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_scrypto_instant_supports_full_i64_range() {
        // Sanity check (epoch):
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(0)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "0".to_string(),
                date_time: Some("1970-01-01T00:00:00Z".to_string()),
            }
        );

        // Slightly positive and negative:
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(1)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "1".to_string(),
                date_time: Some("1970-01-01T00:00:01Z".to_string()),
            }
        );
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(-1)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "-1".to_string(),
                date_time: Some("1969-12-31T23:59:59Z".to_string()),
            }
        );

        // The extremes of ISO8601:
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(253402300799)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "253402300799".to_string(),
                date_time: Some("9999-12-31T23:59:59Z".to_string()),
            }
        );
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(-12212553600)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "-12212553600".to_string(),
                date_time: Some("1583-01-01T00:00:00Z".to_string()),
            }
        );

        // Slightly outside the extremes of ISO8601:
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(253402300800)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "253402300800".to_string(),
                date_time: None,
            }
        );
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(-12212553601)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "-12212553601".to_string(),
                date_time: None,
            }
        );

        // The extremes of i64:
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(i64::MAX)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "9223372036854775807".to_string(),
                date_time: None,
            }
        );
        assert_eq!(
            to_api_scrypto_instant(&Instant::new(i64::MIN)).unwrap(),
            models::ScryptoInstant {
                unix_timestamp_seconds: "-9223372036854775808".to_string(),
                date_time: None,
            }
        );
    }

    #[test]
    fn to_api_clamped_instant_from_epoch_milli_supports_full_i64_range() {
        // Sanity check (epoch):
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(0).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 0,
                date_time: "1970-01-01T00:00:00.000Z".to_string(),
            }
        );

        // Slightly positive and negative:
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(1).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 1,
                date_time: "1970-01-01T00:00:00.001Z".to_string(),
            }
        );
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(-1).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 0, // this is the reason for "clamped" in the name
                date_time: "1970-01-01T00:00:00.000Z".to_string(),
            }
        );

        // Our arbitrary 10^14 maximum, and what happens above it:
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(100000000000000).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 100000000000000,
                date_time: "5138-11-16T09:46:40.000Z".to_string(),
            }
        );
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(100000000000001).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 100000000000000, // clamped as expected
                date_time: "5138-11-16T09:46:40.000Z".to_string(),
            }
        );

        // The extremes of i64:
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(i64::MAX).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 100000000000000,
                date_time: "5138-11-16T09:46:40.000Z".to_string(),
            }
        );
        assert_eq!(
            to_api_clamped_instant_from_epoch_milli(i64::MIN).unwrap(),
            models::InstantMs {
                unix_timestamp_ms: 0,
                date_time: "1970-01-01T00:00:00.000Z".to_string(),
            }
        );
    }
}
