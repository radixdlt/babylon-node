use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{GlobalAddress, SubstateId, SubstateOffset};
use state_manager::{jni::state_manager::ActualStateManager, query::StateManagerSubstateQueries};

use super::{CoreApiState, Extension, Json, RequestHandlingError};

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
pub(crate) fn read_derefed_global_substate(
    state_manager: &ActualStateManager,
    global_address: GlobalAddress,
    substate_offset: SubstateOffset,
) -> Option<PersistedSubstate> {
    let node = state_manager.store.global_deref(global_address)?;
    let substate_id = SubstateId(node, substate_offset);
    let substate = state_manager.store.get_substate(&substate_id)?.substate;
    Some(substate)
}
