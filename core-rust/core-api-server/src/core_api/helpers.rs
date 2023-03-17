use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{RENodeId, SubstateId, SubstateOffset};
use radix_engine_interface::api::types::NodeModuleId;
use state_manager::jni::state_manager::ActualStateManager;

use super::{CoreApiState, Json, MappingError, ResponseError, State};

pub(crate) fn core_api_handler_empty_request<Response>(
    State(state): State<CoreApiState>,
    method: impl FnOnce(&mut ActualStateManager) -> Result<Response, ResponseError<()>>,
) -> Result<Json<Response>, ResponseError<()>> {
    let mut state_manager = state.state_manager.write();

    method(&mut state_manager).map(Json)
}

#[tracing::instrument(skip_all)]
pub(crate) fn core_api_read_handler<Request, Response>(
    State(state): State<CoreApiState>,
    Json(request_body): Json<Request>,
    method: impl FnOnce(&ActualStateManager, Request) -> Result<Response, ResponseError<()>>,
) -> Result<Json<Response>, ResponseError<()>> {
    let state_manager = state.state_manager.read();

    method(&state_manager, request_body).map(Json)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_known_substate(
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
pub(crate) fn wrong_substate_type(substate_offset: SubstateOffset) -> ResponseError<()> {
    MappingError::MismatchedSubstateId {
        message: format!("{substate_offset:?} not of expected type"),
    }
    .into()
}
