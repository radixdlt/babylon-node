use std::ops::DerefMut;

use axum::{Extension, Json};
use state_manager::jni::state_manager::ActualStateManager;

use super::*;

pub(crate) fn core_api_handler_empty_request<Response>(
    state: Extension<CoreApiState>,
    method: impl FnOnce(&mut ActualStateManager) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let state_manager_arc = state.0.state_manager;
    let mut state_manager = state_manager_arc
        .lock()
        .map_err(|_| common_server_errors::state_manager_lock_error())?;

    let response = method(state_manager.deref_mut())?;
    Ok(Json(response))
}

pub(crate) fn core_api_handler<Request, Response>(
    state: Extension<CoreApiState>,
    Json(request_body): Json<Request>,
    method: impl FnOnce(&mut ActualStateManager, Request) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let state_manager_arc = state.0.state_manager;
    let mut state_manager = state_manager_arc
        .lock()
        .map_err(|_| common_server_errors::state_manager_lock_error())?;

    let response = method(state_manager.deref_mut(), request_body)?;
    Ok(Json(response))
}
