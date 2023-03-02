use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{RENodeId, SubstateId, SubstateOffset};
use radix_engine_interface::api::types::NodeModuleId;
use state_manager::{jni::state_manager::ActualStateManager};

use super::{
    CoreApiState, Extension, Json,
    MappingError, ResponseError,
};

pub(crate) fn core_api_handler_empty_request<Response>(
    Extension(state): Extension<CoreApiState>,
    method: impl FnOnce(&mut ActualStateManager) -> Result<Response, ResponseError<()>>,
) -> Result<Json<Response>, ResponseError<()>> {
    let mut state_manager = state.state_manager.write();

    method(&mut state_manager).map(Json)
}

#[tracing::instrument(skip_all)]
pub(crate) fn core_api_read_handler<Request, Response>(
    Extension(state): Extension<CoreApiState>,
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
    let substate_id = SubstateId(renode_id, node_module_id, substate_offset.clone());
    let output_value = state_manager
        .store()
        .get_substate(&substate_id)
        .ok_or_else(|| MappingError::MismatchedSubstateId {
            message: format!(
                "Substate {:?} not found under {:?}",
                substate_offset, renode_id
            ),
        })?;
    Ok(output_value.substate)
}

#[tracing::instrument(skip_all)]
pub(crate) fn wrong_substate_type(substate_offset: SubstateOffset) -> ResponseError<()> {
    MappingError::MismatchedSubstateId {
        message: format!("{:?} not of expected type", substate_offset),
    }
    .into()
}
