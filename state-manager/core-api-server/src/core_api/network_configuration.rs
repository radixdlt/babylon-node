use crate::core_api::errors::{common_server_errors, RequestHandlingError};
use crate::core_api::generated::models::*;
use crate::core_api::generated::{StatusNetworkConfigurationPostResponse, API_VERSION};
use state_manager::jni::state_manager::ActualStateManager;
use std::sync::{Arc, Mutex};

pub(crate) fn handle_network_configuration(
    state_manager: Arc<Mutex<ActualStateManager>>,
) -> StatusNetworkConfigurationPostResponse {
    match handle_network_configuration_internal(state_manager) {
        Ok(response) => StatusNetworkConfigurationPostResponse::NetworkConfiguration(response),
        Err(RequestHandlingError::ServerError(error_response)) => {
            StatusNetworkConfigurationPostResponse::ServerError(error_response)
        }
        Err(RequestHandlingError::ClientError(error_response)) =>
        // No client errors are expected; returning a server error
        {
            StatusNetworkConfigurationPostResponse::ServerError(error_response)
        }
    }
}

fn handle_network_configuration_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
) -> Result<NetworkConfigurationResponse, RequestHandlingError> {
    let locked_state_manager = state_manager
        .lock()
        .map_err(|_| common_server_errors::state_manager_lock_error())?;

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
