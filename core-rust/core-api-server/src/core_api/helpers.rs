use std::ops::DerefMut;

use super::{Extension, Json};
use state_manager::jni::state_manager::ActualStateManager;

use super::*;

pub(crate) fn core_api_handler_empty_request<Response>(
    state: Extension<CoreApiState>,
    method: impl FnOnce(&mut ActualStateManager) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let state_manager_arc = state.0.state_manager;
    let mut state_manager = state_manager_arc.write();

    let response = method(state_manager.deref_mut())?;
    Ok(Json(response))
}

#[tracing::instrument(skip_all)]
pub(crate) fn core_api_handler<Request, Response>(
    state: Extension<CoreApiState>,
    Json(request_body): Json<Request>,
    method: impl FnOnce(&mut ActualStateManager, Request) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let state_manager_arc = state.0.state_manager;
    let mut state_manager = state_manager_arc.write();

    let response = method(state_manager.deref_mut(), request_body)?;
    Ok(Json(response))
}

#[tracing::instrument(skip_all)]
pub(crate) fn core_api_read_handler<Request, Response>(
    state: Extension<CoreApiState>,
    Json(request_body): Json<Request>,
    method: impl FnOnce(&ActualStateManager, Request) -> Result<Response, RequestHandlingError>,
) -> Result<Json<Response>, RequestHandlingError> {
    let state_manager_arc = state.0.state_manager;
    let state_manager = state_manager_arc.read();

    let response = method(&*state_manager, request_body)?;
    Ok(Json(response))
}
