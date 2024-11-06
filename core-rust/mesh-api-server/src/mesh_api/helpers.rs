use crate::engine_prelude::*;
use crate::prelude::*;

#[tracing::instrument(skip_all)]
pub(crate) fn read_current_ledger_header(
    database: &StateManagerDatabase<impl ReadableRocks>,
) -> LedgerHeader {
    database
        .get_latest_proof()
        .expect("proof for outputted state must exist")
        .ledger_header
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_main_field_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase<impl ReadableRocks>,
    node_id: &NodeId,
    substate_key: &SubstateKey,
) -> Result<FieldSubstate<D>, ResponseError> {
    read_mandatory_substate::<FieldSubstate<D>>(
        database,
        node_id,
        MAIN_BASE_PARTITION,
        substate_key,
    )
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase<impl ReadableRocks>,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<D, ResponseError> {
    read_optional_substate(
        database,
        node_id,
        partition_number,
        substate_key
    ).ok_or_else(
        || {
            MappingError::MismatchedSubstateId {
                message: format!(
                    "Substate key {substate_key:?} not found under NodeId {node_id:?} and partition number {partition_number:?}"
                ),
            }
            .into()
        },
    )
}
#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_main_field_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase<impl ReadableRocks>,
    node_id: &NodeId,
    substate_key: &SubstateKey,
) -> Option<FieldSubstate<D>> {
    read_optional_substate::<FieldSubstate<D>>(database, node_id, MAIN_BASE_PARTITION, substate_key)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase<impl ReadableRocks>,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
) -> Option<D> {
    database.get_substate::<D>(node_id, partition_number, substate_key)
}

/// We assume that Block is a single transaction.
/// Block index => State version
/// Block hash  => State version printed to string and prefixed with zeros
pub(crate) fn to_mesh_api_block_identifier_from_state_version(
    state_version: StateVersion,
) -> Result<models::BlockIdentifier, MappingError> {
    let index = to_mesh_api_block_index_from_state_version(state_version)?;
    Ok(models::BlockIdentifier {
        index,
        hash: format!("{:0>32}", index),
    })
}
pub(crate) fn to_mesh_api_block_identifier_from_ledger_header(
    ledger_header: &LedgerStateSummary,
) -> Result<models::BlockIdentifier, MappingError> {
    to_mesh_api_block_identifier_from_state_version(ledger_header.state_version)
}

pub(crate) fn extract_state_version_from_mesh_api_partial_block_identifier(
    block_identifier: &models::PartialBlockIdentifier,
) -> Result<Option<StateVersion>, ExtractionError> {
    let state_version = if let Some(index) = block_identifier.index {
        Some(index)
    } else if let Some(hash) = &block_identifier.hash {
        Some(
            hash.parse::<i64>()
                .map_err(|_| ExtractionError::InvalidBlockIdentifier {
                    message: "Hash parsing error".to_string(),
                })?,
        )
    } else {
        None
    }
    .map(|index| StateVersion::of(index as u64));

    Ok(state_version)
}

pub(crate) fn resource_address_to_currency(
    database: &StateManagerDatabase<impl ReadableRocks>,
    symbol: &str,
    resource_address: ResourceAddress,
) -> Result<models::Currency, MappingError> {
    let resource_node_id = resource_address.as_node_id();
    if resource_node_id.entity_type() != Some(EntityType::GlobalFungibleResourceManager) {
        return Err(MappingError::InvalidResource {
            message: format!("currency {} is not fungible type", symbol),
        });
    }

    let divisibility: FungibleResourceManagerDivisibilityFieldSubstate =
        read_optional_main_field_substate(
            database,
            resource_node_id,
            &FungibleResourceManagerField::Divisibility.into(),
        )
        .ok_or_else(|| MappingError::InvalidResource {
            message: format!("currency {} not found", symbol),
        })?;
    let divisibility = *divisibility.payload().as_unique_version() as i32;

    Ok(models::Currency {
        symbol: symbol.to_string(),
        decimals: divisibility,
        metadata: None,
    })
}
