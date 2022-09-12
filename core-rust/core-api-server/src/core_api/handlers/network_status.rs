use crate::core_api::models::*;
use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::QueryableProofStore;

pub(crate) async fn handle_network_status(
    state: Extension<CoreApiState>,
    request: Json<NetworkStatusRequest>,
) -> Result<Json<NetworkStatusResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_network_status_internal)
}

pub(crate) fn handle_network_status_internal(
    state_manager: &mut ActualStateManager,
    request: NetworkStatusRequest,
) -> Result<NetworkStatusResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    Ok(NetworkStatusResponse {
        pre_genesis_state_identifier: Box::new(CommittedStateIdentifier { state_version: 0 }),
        post_genesis_state_identifier: Box::new(CommittedStateIdentifier { state_version: 1 }),
        current_state_identifier: Box::new(CommittedStateIdentifier {
            state_version: to_api_state_version(state_manager.store.max_state_version())
                .map_err(|err| {
                    common_server_errors::mapping_error(err, "Unable to map state_version")
                })?,
        }),
    })
}
