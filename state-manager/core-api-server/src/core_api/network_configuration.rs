use crate::core_api::generated::models::*;
use crate::core_api::generated::{StatusNetworkConfigurationPostResponse, API_VERSION};
use state_manager::jni::state_manager::ActualStateManager;
use std::sync::{Arc, Mutex};
use swagger::ApiError;

pub(crate) fn handle_network_configuration(
    state_manager: Arc<Mutex<ActualStateManager>>,
) -> Result<StatusNetworkConfigurationPostResponse, ApiError> {
    handle_network_configuration_internal(state_manager)
        .map(StatusNetworkConfigurationPostResponse::NetworkConfiguration)
        .or_else(Ok)
}

fn handle_network_configuration_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
) -> Result<NetworkConfigurationResponse, StatusNetworkConfigurationPostResponse> {
    let locked_state_manager = state_manager
        .lock()
        .map_err(|_| server_error("Internal server error (state manager lock)"))?;
    let network = locked_state_manager.network.clone();

    Ok(NetworkConfigurationResponse {
        version: NetworkConfigurationResponseVersion {
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: API_VERSION.to_string(),
        },
        network_identifier: NetworkIdentifier {
            network: network.logical_name,
        },
        network_hrp_suffix: network.hrp_suffix,
    })
}

fn server_error(message: &str) -> StatusNetworkConfigurationPostResponse {
    StatusNetworkConfigurationPostResponse::ServerError(ErrorResponse::new(
        500,
        message.to_string(),
    ))
}
