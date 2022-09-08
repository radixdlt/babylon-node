use crate::core_api::models::*;
use crate::core_api::*;
use axum::{Extension, Json};
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_network_configuration(
    state: Extension<CoreApiState>,
) -> Result<Json<NetworkConfigurationResponse>, RequestHandlingError> {
    core_api_handler_empty_request(state, handle_network_configuration_internal)
}

pub(crate) fn handle_network_configuration_internal(
    state_manager: &mut ActualStateManager,
) -> Result<NetworkConfigurationResponse, RequestHandlingError> {
    let network = state_manager.network.clone();

    Ok(NetworkConfigurationResponse {
        version: Box::new(NetworkConfigurationResponseVersion {
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: "API_VERSION".to_string(), // TODO - fix this to come from something auto-generated
        }),
        network_identifier: Box::new(NetworkIdentifier {
            network: network.logical_name,
        }),
        network_hrp_suffix: network.hrp_suffix,
    })
}
