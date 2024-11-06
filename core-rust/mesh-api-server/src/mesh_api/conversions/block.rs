use crate::prelude::*;

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

pub fn extract_state_version_from_mesh_api_partial_block_identifier(
    block_identifier: &models::PartialBlockIdentifier,
) -> Result<Option<StateVersion>, ExtractionError> {
    let state_version = if let Some(index) = block_identifier.index {
        Some(StateVersion::of(index as u64))
    } else if let Some(hash) = &block_identifier.hash {
        let index = hash
            .parse::<i64>()
            .map_err(|_| ExtractionError::InvalidInteger {
                message: "Error converting hash to integer".to_string(),
            })?;
        Some(StateVersion::of(index as u64))
    } else {
        None
    };

    Ok(state_version)
}
