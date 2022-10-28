use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::model::PersistedSubstate;
use scrypto::engine::types::{GlobalAddress, GlobalOffset, RENodeId, SubstateId, SubstateOffset};

use super::{CoreApiState, Extension, Json, MappingError, RequestHandlingError};
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) fn core_api_handler_empty_request<Response>(
    Extension(state): Extension<CoreApiState>,
    method: impl FnOnce(&mut ActualStateManager) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let mut state_manager = state.state_manager.write();

    method(&mut state_manager).map(Json)
}

#[tracing::instrument(skip_all)]
pub(crate) fn core_api_handler<Request, Response>(
    Extension(state): Extension<CoreApiState>,
    Json(request_body): Json<Request>,
    method: impl FnOnce(&mut ActualStateManager, Request) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let mut state_manager = state.state_manager.write();

    method(&mut state_manager, request_body).map(Json)
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
) -> Result<Option<PersistedSubstate>, MappingError> {
    let substate_id = SubstateId(
        RENodeId::Global(global_address),
        SubstateOffset::Global(GlobalOffset::Global),
    );
    if let Some(output_value) = state_manager.store.get_substate(&substate_id) {
        if let PersistedSubstate::Global(global) = output_value.substate {
            let substate_id = SubstateId(global.node_deref(), substate_offset);
            let substate = state_manager.store.get_substate(&substate_id);
            Ok(substate.map(|o| o.substate))
        } else {
            Err(MappingError::MismatchedSubstateId {
                message: "Global address substate was not of the right type".to_owned(),
            })
        }
    } else {
        Ok(None)
    }
}
