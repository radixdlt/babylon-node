use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{RENodeId, SubstateId, SubstateOffset};
use radix_engine_interface::api::types::NodeModuleId;
use state_manager::jni::state_manager::ActualStateManager;

use super::{MappingError, ResponseError};

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_substate(
    state_manager: &ActualStateManager,
    renode_id: RENodeId,
    node_module_id: NodeModuleId,
    substate_offset: &SubstateOffset,
) -> Result<PersistedSubstate, ResponseError<()>> {
    read_optional_substate(state_manager, renode_id, node_module_id, substate_offset).ok_or_else(
        || {
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
pub(crate) fn read_mandatory_substate_from_id(
    state_manager: &ActualStateManager,
    substate_id: &SubstateId,
) -> Result<PersistedSubstate, ResponseError<()>> {
    read_optional_substate_from_id(state_manager, substate_id).ok_or_else(
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
    state_manager: &ActualStateManager,
    renode_id: RENodeId,
    node_module_id: NodeModuleId,
    substate_offset: &SubstateOffset,
) -> Option<PersistedSubstate> {
    let substate_id = SubstateId(renode_id, node_module_id, substate_offset.clone());
    state_manager
        .store()
        .get_substate(&substate_id)
        .map(|o| o.substate)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_substate_from_id(
    state_manager: &ActualStateManager,
    substate_id: &SubstateId,
) -> Option<PersistedSubstate> {
    state_manager
        .store()
        .get_substate(substate_id)
        .map(|o| o.substate)
}

#[tracing::instrument(skip_all)]
pub(crate) fn wrong_substate_type(substate_offset: SubstateOffset) -> ResponseError<()> {
    MappingError::MismatchedSubstateId {
        message: format!("{substate_offset:?} not of expected type"),
    }
    .into()
}
