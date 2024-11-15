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
pub(crate) fn read_optional_collection_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase<impl ReadableRocks>,
    node_id: &NodeId,
    collection_index: CollectionIndex,
    substate_key: &SubstateKey,
) -> Option<KeyValueEntrySubstate<D>> {
    // Note - the field partition (if it exists) takes the first partition number,
    // the collections go after - so start at offset 1
    // (assuming there is a tuple partition on the node...)
    let partition_number = MAIN_BASE_PARTITION
        .at_offset(PartitionOffset(1 + collection_index))
        .unwrap();
    read_optional_substate::<KeyValueEntrySubstate<D>>(
        database,
        node_id,
        partition_number,
        substate_key,
    )
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
