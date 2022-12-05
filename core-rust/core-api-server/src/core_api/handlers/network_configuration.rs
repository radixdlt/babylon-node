use crate::core_api::*;
use radix_engine::types::*;
use radix_engine_interface::address::{EntityType, HrpSet};
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
    let hrp_set: HrpSet = (&network).into();

    let address_types = ALL_ENTITY_TYPES
        .into_iter()
        .map(|entity_type| to_api_address_type(&hrp_set, entity_type))
        .collect::<Vec<_>>();

    Ok(models::NetworkConfigurationResponse {
        version: Box::new(models::NetworkConfigurationResponseVersion {
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: models::SCHEMA_VERSION.to_string(),
        }),
        network: network.logical_name,
        network_hrp_suffix: network.hrp_suffix,
        address_types,
        well_known_addresses: Box::new(models::NetworkConfigurationResponseWellKnownAddresses {
            account_package: bech32_encoder.encode_package_address_to_string(&ACCOUNT_PACKAGE),
            faucet: bech32_encoder.encode_component_address_to_string(&FAUCET_COMPONENT),
            epoch_manager: bech32_encoder.encode_system_address_to_string(&EPOCH_MANAGER),
            clock: bech32_encoder.encode_system_address_to_string(&CLOCK),
            ecdsa_secp256k1: bech32_encoder
                .encode_resource_address_to_string(&ECDSA_SECP256K1_TOKEN),
            eddsa_ed25519: bech32_encoder.encode_resource_address_to_string(&EDDSA_ED25519_TOKEN),
            xrd: bech32_encoder.encode_resource_address_to_string(&RADIX_TOKEN),
        }),
    })
}

const ALL_ENTITY_TYPES: [EntityType; 8] = [
    EntityType::Resource,
    EntityType::Package,
    EntityType::NormalComponent,
    EntityType::AccountComponent,
    EntityType::EcdsaSecp256k1VirtualAccountComponent,
    EntityType::EddsaEd25519VirtualAccountComponent,
    EntityType::EpochManager,
    EntityType::Clock,
];

fn to_api_address_type(hrp_set: &HrpSet, entity_type: EntityType) -> models::AddressType {
    // If you add another entity type here, add it to the ALL_ENTITY_TYPES list above.
    // We do it like this in a match statement so that we catch a compile error if a new entity type is added :)
    let (subtype, api_entity_type, address_length) = match entity_type {
        EntityType::Resource => (
            models::address_type::Subtype::Resource,
            models::EntityType::ResourceManager,
            extract_length(ResourceAddress::Normal),
        ),
        EntityType::Package => (
            models::address_type::Subtype::Package,
            models::EntityType::Package,
            extract_length(PackageAddress::Normal),
        ),
        EntityType::NormalComponent => (
            models::address_type::Subtype::NormalComponent,
            models::EntityType::Component,
            extract_length(ComponentAddress::Normal),
        ),
        EntityType::AccountComponent => (
            models::address_type::Subtype::AccountComponent,
            models::EntityType::Component,
            extract_length(ComponentAddress::Account),
        ),
        EntityType::EcdsaSecp256k1VirtualAccountComponent => (
            models::address_type::Subtype::EcdsaSecp256k1VirtualAccountComponent,
            models::EntityType::Component,
            extract_length(ComponentAddress::EcdsaSecp256k1VirtualAccount),
        ),
        EntityType::EddsaEd25519VirtualAccountComponent => (
            models::address_type::Subtype::EddsaEd25519VirtualAccountComponent,
            models::EntityType::Component,
            extract_length(ComponentAddress::EddsaEd25519VirtualAccount),
        ),
        EntityType::EpochManager => (
            models::address_type::Subtype::EpochManager,
            models::EntityType::EpochManager,
            extract_length(SystemAddress::EpochManager),
        ),
        EntityType::Clock => (
            models::address_type::Subtype::Clock,
            models::EntityType::Clock,
            extract_length(SystemAddress::Clock),
        ),
    };
    models::AddressType {
        hrp_prefix: hrp_set.get_entity_hrp(&entity_type).to_string(),
        subtype,
        entity_type: api_entity_type,
        address_byte_prefix: entity_type.id().into(),
        address_byte_length: address_length
            .try_into()
            .expect("address was longer than expected"),
    }
}

fn extract_length<T: FnOnce([u8; N]) -> X, const N: usize, X>(_: T) -> usize {
    N
}
