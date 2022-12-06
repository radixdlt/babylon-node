use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{GlobalAddress, RENodeId, SubstateId, SubstateOffset};
use state_manager::{jni::state_manager::ActualStateManager, query::StateManagerSubstateQueries};

use super::{
    get_entity_type_from_global_address, not_found_error, CoreApiState, Extension, Json,
    MappingError, RequestHandlingError,
};

pub(crate) fn core_api_handler_empty_request<Response>(
    Extension(state): Extension<CoreApiState>,
    method: impl FnOnce(&mut ActualStateManager) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let mut state_manager = state.state_manager.write();

    method(&mut state_manager).map(Json)
}

#[tracing::instrument(skip_all)]
pub(crate) fn core_api_read_handler<Request, Response>(
    Extension(state): Extension<CoreApiState>,
    Json(request_body): Json<Request>,
    method: impl FnOnce(&ActualStateManager, Request) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let state_manager = state.state_manager.read();

    method(&state_manager, request_body).map(Json)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_derefed_global_node_id(
    state_manager: &ActualStateManager,
    global_address: GlobalAddress,
) -> Result<RENodeId, RequestHandlingError> {
    state_manager
        .store
        .global_deref(global_address)
        .ok_or_else(|| {
            not_found_error(format!(
                "{:?} not found",
                get_entity_type_from_global_address(&global_address)
            ))
        })
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_known_substate(
    state_manager: &ActualStateManager,
    renode_id: RENodeId,
    substate_offset: &SubstateOffset,
) -> Result<PersistedSubstate, RequestHandlingError> {
    let substate_id = SubstateId(renode_id, substate_offset.clone());
    let output_value = state_manager
        .store
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
pub(crate) fn wrong_substate_type(substate_offset: SubstateOffset) -> RequestHandlingError {
    MappingError::MismatchedSubstateId {
        message: format!("{:?} not of expected type", substate_offset),
    }
    .into()
}
