use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{RENodeId, SubstateId, SubstateOffset};
use radix_engine_interface::api::types::NodeModuleId;

use state_manager::store::StateManagerDatabase;

use super::{MappingError, ResponseError};

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_substate(
    database: &StateManagerDatabase,
    renode_id: RENodeId,
    node_module_id: NodeModuleId,
    substate_offset: &SubstateOffset,
) -> Result<PersistedSubstate, ResponseError<()>> {
    let substate_id = SubstateId(renode_id, node_module_id, substate_offset.clone());
    read_mandatory_substate_from_id(database, &substate_id)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_substate_from_id(
    database: &StateManagerDatabase,
    substate_id: &SubstateId,
) -> Result<PersistedSubstate, ResponseError<()>> {
    read_optional_substate_from_id(database, substate_id).ok_or_else(
        || {
            let SubstateId(renode_id, node_module_id, substate_offset) = substate_id;
            MappingError::MismatchedSubstateId {
                message: format!(
                    "Substate {substate_offset:?} not found under RE node {renode_id:?} and module {node_module_id:?}"
                ),
            }
            .into()
        },
    )
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_substate(
    database: &StateManagerDatabase,
    renode_id: RENodeId,
    node_module_id: NodeModuleId,
    substate_offset: &SubstateOffset,
) -> Option<PersistedSubstate> {
    let substate_id = SubstateId(renode_id, node_module_id, substate_offset.clone());
    read_optional_substate_from_id(database, &substate_id)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_substate_from_id(
    database: &StateManagerDatabase,
    substate_id: &SubstateId,
) -> Option<PersistedSubstate> {
    database.get_substate(substate_id).map(|o| o.substate)
}

#[tracing::instrument(skip_all)]
pub(crate) fn wrong_substate_type(substate_offset: SubstateOffset) -> ResponseError<()> {
    MappingError::MismatchedSubstateId {
        message: format!("{substate_offset:?} not of expected type"),
    }
    .into()
}
