use crate::core_api::generated::models::*;
use crate::core_api::generated::{StatusNetworkConfigurationPostResponse, API_VERSION};
use scrypto::address::get_network_hrp_set;
use state_manager::StateManager;
use std::sync::{Arc, Mutex};
use swagger::ApiError;

pub(crate) fn handle_network_configuration(
    state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>,
) -> Result<StatusNetworkConfigurationPostResponse, ApiError> {
    handle_network_configuration_internal(state_manager)
        .map(StatusNetworkConfigurationPostResponse::NetworkConfiguration)
        .or_else(Ok)
}

fn handle_network_configuration_internal(
    state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>,
) -> Result<NetworkConfigurationResponse, StatusNetworkConfigurationPostResponse> {
    let locked_state_manager = state_manager
        .lock()
        .map_err(|_| server_error("Internal server error (state manager lock)"))?;
    let network = locked_state_manager.network();

    let hrp_set = get_network_hrp_set(network);

    Ok(NetworkConfigurationResponse {
        version: NetworkConfigurationResponseVersion {
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: API_VERSION.to_string(),
        },
        network_identifier: NetworkIdentifier {
            network: format!("{:?}", network),
        },
        bech32_human_readable_parts: Bech32Hrps {
            account_hrp: hrp_set.account_component.to_string(),
            validator_hrp: "TODO".to_string(),
            node_hrp: "TODO".to_string(),
            resource_hrp_suffix: hrp_set.resource.to_string(),
        },
    })
}

fn server_error(message: &str) -> StatusNetworkConfigurationPostResponse {
    StatusNetworkConfigurationPostResponse::ServerError(ErrorResponse::new(
        500,
        message.to_string(),
    ))
}
