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

#[derive(Debug, Clone, EnumIter, Display, FromRepr)]
#[repr(i64)]
pub(crate) enum OperationTypes {
    Withdraw,
    Deposit,
    // LockFee,
    // Mint,
    // Burn,
}
