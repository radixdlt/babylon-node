use crate::prelude::*;

const MAX_API_STATE_VERSION: u64 = 100000000000000;

/// We assume that Block is a single transaction.
/// Block index => State version
/// Block hash  => 32 bytes of: transaction_tree_hash[0..12] | receipt_tree_hash[0..12] | state_version
pub fn extract_state_version_from_block_hash(
    database: &ActualStateManagerDatabase,
    block_hash: &str,
) -> Result<StateVersion, ExtractionError> {
    if block_hash.len() == 32 {
        let hash_bytes =
            hex::decode(block_hash).map_err(|_| ExtractionError::InvalidBlockIdentifier {
                message: format!("Error decoding block hash {}", block_hash),
            })?;

        let mut index_bytes: [u8; 8] = [0; 8];
        index_bytes.copy_from_slice(&hash_bytes[24..]);
        let index = u64::from_be_bytes(index_bytes);

        let state_version = StateVersion::of(index);
        let transaction_identifiers = database
            .get_committed_transaction_identifiers(state_version)
            .ok_or_else(|| ExtractionError::NotFound)?;

        let transaction_tree_hash = transaction_identifiers
            .resultant_ledger_hashes
            .transaction_root;
        let receipt_tree_hash = transaction_identifiers.resultant_ledger_hashes.receipt_root;

        if hash_bytes[..12] != transaction_tree_hash.as_slice()[0..12] {
            return Err(ExtractionError::InvalidBlockIdentifier {
                message: format!(
                    "Block hash {} does not match transaction tree hash",
                    block_hash
                ),
            });
        }
        if hash_bytes[12..24] != receipt_tree_hash.as_slice()[0..12] {
            return Err(ExtractionError::InvalidBlockIdentifier {
                message: format!("Block hash {} does not match receipt tree hash", block_hash),
            });
        }

        Ok(state_version)
    } else {
        Err(ExtractionError::InvalidBlockIdentifier {
            message: format!("hash length {} not equal 32", block_hash.len()),
        })
    }
}

pub fn extract_state_version_from_mesh_api_partial_block_identifier(
    database: &ActualStateManagerDatabase,
    block_identifier: &models::PartialBlockIdentifier,
) -> Result<Option<StateVersion>, ExtractionError> {
    let state_version = match (&block_identifier.hash, &block_identifier.index) {
        (None, None) => None,
        (Some(hash), None) => Some(extract_state_version_from_block_hash(database, hash)?),
        (None, Some(index)) => Some(StateVersion::of(*index as u64)),
        (Some(hash), Some(index)) => {
            let state_version = extract_state_version_from_block_hash(database, hash)?;
            if *index as u64 == state_version.number() {
                Some(state_version)
            } else {
                return Err(ExtractionError::InvalidBlockIdentifier {
                    message: format!("Hash {} does not match index {}", hash, index),
                });
            }
        }
    };

    Ok(state_version)
}

pub fn extract_state_version_from_mesh_api_block_identifier(
    database: &ActualStateManagerDatabase,
    block_identifier: &models::BlockIdentifier,
) -> Result<StateVersion, ExtractionError> {
    let state_version_from_hash =
        extract_state_version_from_block_hash(database, &block_identifier.hash)?;
    if block_identifier.index as u64 != state_version_from_hash.number() {
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

pub fn to_mesh_api_block_identifier_from_state_version(
    state_version: StateVersion,
    transaction_tree_hash: &TransactionTreeHash,
    receipt_tree_hash: &ReceiptTreeHash,
) -> Result<models::BlockIdentifier, MappingError> {
    let index = to_mesh_api_block_index_from_state_version(state_version)?;

    let mut hash_bytes = [0u8; 32];

    hash_bytes[..12].copy_from_slice(&transaction_tree_hash.as_slice()[..12]);
    hash_bytes[12..24].copy_from_slice(&receipt_tree_hash.as_slice()[..12]);
    hash_bytes[24..].copy_from_slice((index as u64).to_be_bytes().as_slice());

    Ok(models::BlockIdentifier {
        index,
        hash: hex::encode(hash_bytes),
    })
}

pub fn to_mesh_api_block_identifier_from_ledger_header(
    ledger_header: &LedgerStateSummary,
) -> Result<models::BlockIdentifier, MappingError> {
    to_mesh_api_block_identifier_from_state_version(
        ledger_header.state_version,
        &ledger_header.hashes.transaction_root,
        &ledger_header.hashes.receipt_root,
    )
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
