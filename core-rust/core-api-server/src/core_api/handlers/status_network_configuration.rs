use crate::core_api::*;
use radix_engine::types::*;
use radix_engine_interface::address::{EntityType, HrpSet};
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(err(Debug), skip(state))]
pub(crate) async fn handle_status_network_configuration(
    state: State<CoreApiState>,
) -> Result<Json<models::NetworkConfigurationResponse>, ResponseError<()>> {
    core_api_handler_empty_request(state, handle_status_network_configuration_internal)
}

pub(crate) fn handle_status_network_configuration_internal(
    state_manager: &mut ActualStateManager,
) -> Result<models::NetworkConfigurationResponse, ResponseError<()>> {
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
        network_id: to_api_u8_as_i32(network.id),
        network_hrp_suffix: network.hrp_suffix,
        address_types,
        well_known_addresses: Box::new(models::NetworkConfigurationResponseWellKnownAddresses {
            faucet: bech32_encoder.encode_component_address_to_string(&FAUCET_COMPONENT),
            epoch_manager: bech32_encoder.encode_component_address_to_string(&EPOCH_MANAGER),
            clock: bech32_encoder.encode_component_address_to_string(&CLOCK),
            ecdsa_secp256k1: bech32_encoder
                .encode_resource_address_to_string(&ECDSA_SECP256K1_TOKEN),
            eddsa_ed25519: bech32_encoder.encode_resource_address_to_string(&EDDSA_ED25519_TOKEN),
            xrd: bech32_encoder.encode_resource_address_to_string(&RADIX_TOKEN),
        }),
    })
}

const ALL_ENTITY_TYPES: [EntityType; 14] = [
    EntityType::Package,
    EntityType::FungibleResource,
    EntityType::NonFungibleResource,
    EntityType::NormalComponent,
    EntityType::AccountComponent,
    EntityType::EcdsaSecp256k1VirtualAccountComponent,
    EntityType::EddsaEd25519VirtualAccountComponent,
    EntityType::IdentityComponent,
    EntityType::EcdsaSecp256k1VirtualIdentityComponent,
    EntityType::EddsaEd25519VirtualIdentityComponent,
    EntityType::EpochManager,
    EntityType::Validator,
    EntityType::Clock,
    EntityType::AccessControllerComponent,
];

fn to_api_address_type(hrp_set: &HrpSet, entity_type: EntityType) -> models::AddressType {
    // If you add another entity type here, add it to the ALL_ENTITY_TYPES list above.
    // We do it like this in a match statement so that we catch a compile error if a new entity type is added :)
    let (subtype, api_entity_type) = match entity_type {
        EntityType::FungibleResource => (
            models::address_type::Subtype::FungibleResource,
            models::EntityType::FungibleResource,
        ),
        EntityType::NonFungibleResource => (
            models::address_type::Subtype::NonFungibleResource,
            models::EntityType::NonFungibleResource,
        ),
        EntityType::Package => (
            models::address_type::Subtype::Package,
            models::EntityType::Package,
        ),
        EntityType::NormalComponent => (
            models::address_type::Subtype::NormalComponent,
            models::EntityType::NormalComponent,
        ),
        EntityType::AccountComponent => (
            models::address_type::Subtype::AccountComponent,
            models::EntityType::Account,
        ),
        EntityType::EcdsaSecp256k1VirtualAccountComponent => (
            models::address_type::Subtype::EcdsaSecp256k1VirtualAccountComponent,
            models::EntityType::Account,
        ),
        EntityType::EddsaEd25519VirtualAccountComponent => (
            models::address_type::Subtype::EddsaEd25519VirtualAccountComponent,
            models::EntityType::Account,
        ),
        EntityType::IdentityComponent => (
            models::address_type::Subtype::IdentityComponent,
            models::EntityType::Identity,
        ),
        EntityType::EcdsaSecp256k1VirtualIdentityComponent => (
            models::address_type::Subtype::EcdsaSecp256k1VirtualIdentityComponent,
            models::EntityType::Identity,
        ),
        EntityType::EddsaEd25519VirtualIdentityComponent => (
            models::address_type::Subtype::EddsaEd25519VirtualIdentityComponent,
            models::EntityType::Identity,
        ),
        EntityType::EpochManager => (
            models::address_type::Subtype::EpochManager,
            models::EntityType::EpochManager,
        ),
        EntityType::Validator => (
            models::address_type::Subtype::Validator,
            models::EntityType::Validator,
        ),
        EntityType::Clock => (
            models::address_type::Subtype::Clock,
            models::EntityType::Clock,
        ),
        EntityType::AccessControllerComponent => (
            models::address_type::Subtype::AccessController,
            models::EntityType::AccessController,
        ),
    };
    models::AddressType {
        hrp_prefix: hrp_set.get_entity_hrp(&entity_type).to_string(),
        entity_type: api_entity_type,
        subtype,
        address_byte_prefix: entity_type.id().into(),
        address_byte_length: ADDRESS_LENGTH.try_into().unwrap(),
    }
}
