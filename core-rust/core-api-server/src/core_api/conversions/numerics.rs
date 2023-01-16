use radix_engine::types::Decimal;

use super::*;

// See the guidance in the top of the core-api-schema.yaml
// These should be duplicated in the Open API schema along with the actual types
// If mapped to i32, these need to be below 4294967295 = 2^32 - 1 to ensure they fit into an i64 on the OAS.
// If mapped to i64, these need to be below 9223372036854775807 = 2^63 - 1 to ensure they fit into an i64 on the OAS.

const MAX_API_EPOCH: u64 = 10000000000;
const MAX_API_ROUND: u64 = 10000000000;
const MAX_API_STATE_VERSION: u64 = 100000000000000;
const MAX_API_SUBSTATE_VERSION: u64 = 100000000000000;
const MAX_API_TIMESTAMP_MS: u128 = 100000000000000; // For comparison, current timestamp is 1673822843000 (about 1/100th the size)

#[tracing::instrument(skip_all)]
pub fn to_api_epoch(epoch: u64) -> Result<i64, MappingError> {
    if epoch > MAX_API_EPOCH {
        return Err(MappingError::IntegerError {
            message: "Epoch larger than max api epoch".to_owned(),
        });
    }
    Ok(epoch.try_into().expect("Epoch too large somehow"))
}

#[tracing::instrument(skip_all)]
pub fn to_api_round(round: u64) -> Result<i64, MappingError> {
    if round > MAX_API_ROUND {
        return Err(MappingError::IntegerError {
            message: "Round larger than max api round".to_owned(),
        });
    }
    Ok(round.try_into().expect("Round too large somehow"))
}

#[tracing::instrument(skip_all)]
pub fn to_api_state_version(state_version: u64) -> Result<i64, MappingError> {
    if state_version > MAX_API_STATE_VERSION {
        return Err(MappingError::IntegerError {
            message: "State version larger than max api state version".to_owned(),
        });
    }
    Ok(state_version
        .try_into()
        .expect("State version too large somehow"))
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_version(substate_version: u32) -> Result<i64, MappingError> {
    let substate_version: u64 = substate_version.into();
    if substate_version > MAX_API_SUBSTATE_VERSION {
        return Err(MappingError::IntegerError {
            message: "Substate version larger than max api state version".to_owned(),
        });
    }
    Ok(substate_version
        .try_into()
        .expect("Substate version too large somehow"))
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

pub fn to_api_u64_as_string(input: u64) -> String {
    input.to_string()
}

pub fn to_api_unix_timestamp_ms(time: std::time::SystemTime) -> Result<i64, MappingError> {
    let millis = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    if millis > MAX_API_TIMESTAMP_MS {
        return Err(MappingError::IntegerError {
            message: format!("Timestamp ms must be <= {}", MAX_API_TIMESTAMP_MS),
        });
    }
    Ok(millis as i64)
}

pub fn extract_api_state_version(state_version: i64) -> Result<u64, ExtractionError> {
    if state_version < 1 {
        return Err(ExtractionError::InvalidInteger {
            message: "State version must be >= 1".to_owned(),
        });
    }
    let state_version: u64 = state_version
        .try_into()
        .expect("State version invalid somehow");
    if state_version > MAX_API_STATE_VERSION {
        return Err(ExtractionError::InvalidInteger {
            message: format!(
                "State version is larger than the max allowed: {}",
                MAX_API_STATE_VERSION
            ),
        });
    }
    Ok(state_version)
}

pub fn extract_api_epoch(epoch: i64) -> Result<u64, ExtractionError> {
    if epoch < 0 {
        return Err(ExtractionError::InvalidInteger {
            message: "Epoch too low".to_owned(),
        });
    }
    let epoch: u64 = epoch.try_into().expect("Epoch invalid somehow");
    if epoch > MAX_API_EPOCH {
        return Err(ExtractionError::InvalidInteger {
            message: "Epoch larger than max api epoch".to_owned(),
        });
    }
    Ok(epoch)
}

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
