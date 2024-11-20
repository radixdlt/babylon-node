use crate::prelude::*;

const MAX_API_STATE_VERSION: u64 = 100000000000000;

pub fn extract_state_version_from_mesh_api_partial_block_identifier(
    block_identifier: &models::PartialBlockIdentifier,
) -> Result<Option<StateVersion>, ExtractionError> {
    let state_version = match (&block_identifier.hash, &block_identifier.index) {
        (None, None) => None,
        (Some(hash), None) => {
            let index_from_hash =
                hash.parse::<i64>()
                    .map_err(|_| ExtractionError::InvalidBlockIdentifier {
                        message: "Error converting hash to integer".to_string(),
                    })?;

            Some(StateVersion::of(index_from_hash as u64))
        }
        (None, Some(index)) => Some(StateVersion::of(*index as u64)),
        (Some(hash), Some(index)) => {
            let index_from_hash =
                hash.parse::<i64>()
                    .map_err(|_| ExtractionError::InvalidBlockIdentifier {
                        message: "Error converting hash to integer".to_string(),
                    })?;
            if *index == index_from_hash {
                Some(StateVersion::of(index_from_hash as u64))
            } else {
                return Err(ExtractionError::InvalidBlockIdentifier {
                    message: format!("Hash {} does not match index {}", index_from_hash, index),
                });
            }
        }
    };

    Ok(state_version)
}

pub fn extract_state_version_from_mesh_api_block_identifier(
    block_identifier: &models::BlockIdentifier,
) -> Result<StateVersion, ExtractionError> {
    let index_from_hash = block_identifier.hash.parse::<i64>().map_err(|_| {
        ExtractionError::InvalidBlockIdentifier {
            message: "Error converting hash to integer".to_string(),
        }
    })?;
    if block_identifier.index != index_from_hash {
        Err(ExtractionError::InvalidBlockIdentifier {
            message: format!(
                "index {} and hash {} mismatch",
                block_identifier.index, block_identifier.hash
            ),
        })
    } else {
        Ok(StateVersion::of(block_identifier.index as u64))
    }
}

/// We assume that Block is a single transaction.
/// Block index => State version
/// Block hash  => State version printed to string and prefixed with zeros
pub fn to_mesh_api_block_identifier_from_state_version(
    state_version: StateVersion,
) -> Result<models::BlockIdentifier, MappingError> {
    let index = to_mesh_api_block_index_from_state_version(state_version)?;
    Ok(models::BlockIdentifier {
        index,
        hash: format!("{:0>32}", index),
    })
}

pub fn to_mesh_api_block_identifier_from_ledger_header(
    ledger_header: &LedgerStateSummary,
) -> Result<models::BlockIdentifier, MappingError> {
    to_mesh_api_block_identifier_from_state_version(ledger_header.state_version)
}

pub fn to_mesh_api_block_index_from_state_version(
    state_version: StateVersion,
) -> Result<i64, MappingError> {
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
