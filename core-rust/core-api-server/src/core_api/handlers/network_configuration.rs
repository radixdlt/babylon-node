use crate::core_api::*;
use radix_engine::types::*;
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(err(Debug), skip(state))]
pub(crate) async fn handle_network_configuration(
    state: Extension<CoreApiState>,
) -> Result<Json<models::NetworkConfigurationResponse>, RequestHandlingError> {
    core_api_handler_empty_request(state, handle_network_configuration_internal)
}

pub(crate) fn handle_network_configuration_internal(
    state_manager: &mut ActualStateManager,
) -> Result<models::NetworkConfigurationResponse, RequestHandlingError> {
    let network = state_manager.network.clone();

    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    Ok(models::NetworkConfigurationResponse {
        version: Box::new(models::NetworkConfigurationResponseVersion {
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: models::SCHEMA_VERSION.to_string(),
        }),
        network: network.logical_name,
        network_hrp_suffix: network.hrp_suffix,
        well_known_addresses: Box::new(models::NetworkConfigurationResponseWellKnownAddresses {
            account_package: bech32_encoder.encode_package_address_to_string(&ACCOUNT_PACKAGE),
            faucet: bech32_encoder.encode_component_address_to_string(&SYS_FAUCET_COMPONENT),
            ecdsa_secp256k1: bech32_encoder
                .encode_resource_address_to_string(&ECDSA_SECP256K1_TOKEN),
            eddsa_ed25519: bech32_encoder.encode_resource_address_to_string(&EDDSA_ED25519_TOKEN),
            xrd: bech32_encoder.encode_resource_address_to_string(&RADIX_TOKEN),
        }),
    })
}
