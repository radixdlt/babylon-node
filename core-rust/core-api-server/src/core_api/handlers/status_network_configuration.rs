use crate::core_api::*;
use radix_engine::types::*;
use radix_engine_common::types::EntityType;
use radix_engine_interface::address::HrpSet;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_status_network_configuration(
    state: State<CoreApiState>,
) -> Result<Json<models::NetworkConfigurationResponse>, ResponseError<()>> {
    let network = state.network.clone();

    let bech32_encoder = AddressBech32Encoder::new(&network);
    let hrp_set: HrpSet = (&network).into();

    let address_types = ALL_ENTITY_TYPES
        .into_iter()
        .map(|entity_type| to_api_address_type(&hrp_set, entity_type))
        .collect::<Vec<_>>();

    Ok(Json(models::NetworkConfigurationResponse {
        version: Box::new(models::NetworkConfigurationResponseVersion {
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: models::SCHEMA_VERSION.to_string(),
        }),
        network: network.logical_name,
        network_id: to_api_u8_as_i32(network.id),
        network_hrp_suffix: network.hrp_suffix,
        usd_price_in_xrd: to_api_decimal(&Decimal::try_from(USD_PRICE_IN_XRD).unwrap()),
        address_types,
        well_known_addresses: Box::new(models::NetworkConfigurationResponseWellKnownAddresses {
            xrd: bech32_encoder.encode(XRD.as_ref()).unwrap(),
            secp256k1_signature_virtual_badge: bech32_encoder
                .encode(SECP256K1_SIGNATURE_VIRTUAL_BADGE.as_ref())
                .unwrap(),
            ed25519_signature_virtual_badge: bech32_encoder
                .encode(ED25519_SIGNATURE_VIRTUAL_BADGE.as_ref())
                .unwrap(),
            package_of_direct_caller_virtual_badge: bech32_encoder
                .encode(PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.as_ref())
                .unwrap(),
            global_caller_virtual_badge: bech32_encoder
                .encode(GLOBAL_CALLER_VIRTUAL_BADGE.as_ref())
                .unwrap(),
            system_transaction_badge: bech32_encoder
                .encode(SYSTEM_TRANSACTION_BADGE.as_ref())
                .unwrap(),
            package_owner_badge: bech32_encoder.encode(PACKAGE_OWNER_BADGE.as_ref()).unwrap(),
            validator_owner_badge: bech32_encoder
                .encode(VALIDATOR_OWNER_BADGE.as_ref())
                .unwrap(),
            account_owner_badge: bech32_encoder.encode(ACCOUNT_OWNER_BADGE.as_ref()).unwrap(),
            identity_owner_badge: bech32_encoder
                .encode(IDENTITY_OWNER_BADGE.as_ref())
                .unwrap(),
            package_package: bech32_encoder.encode(PACKAGE_PACKAGE.as_ref()).unwrap(),
            resource_package: bech32_encoder.encode(RESOURCE_PACKAGE.as_ref()).unwrap(),
            account_package: bech32_encoder.encode(ACCOUNT_PACKAGE.as_ref()).unwrap(),
            identity_package: bech32_encoder.encode(IDENTITY_PACKAGE.as_ref()).unwrap(),
            consensus_manager_package: bech32_encoder
                .encode(CONSENSUS_MANAGER_PACKAGE.as_ref())
                .unwrap(),
            access_controller_package: bech32_encoder
                .encode(ACCESS_CONTROLLER_PACKAGE.as_ref())
                .unwrap(),
            transaction_processor_package: bech32_encoder
                .encode(TRANSACTION_PROCESSOR_PACKAGE.as_ref())
                .unwrap(),
            metadata_module_package: bech32_encoder
                .encode(METADATA_MODULE_PACKAGE.as_ref())
                .unwrap(),
            royalty_module_package: bech32_encoder
                .encode(ROYALTY_MODULE_PACKAGE.as_ref())
                .unwrap(),
            role_assignment_module_package: bech32_encoder
                .encode(ROLE_ASSIGNMENT_MODULE_PACKAGE.as_ref())
                .unwrap(),
            genesis_helper_package: bech32_encoder
                .encode(GENESIS_HELPER_PACKAGE.as_ref())
                .unwrap(),
            faucet_package: bech32_encoder.encode(FAUCET_PACKAGE.as_ref()).unwrap(),
            pool_package: bech32_encoder.encode(POOL_PACKAGE.as_ref()).unwrap(),
            consensus_manager: bech32_encoder.encode(CONSENSUS_MANAGER.as_ref()).unwrap(),
            genesis_helper: bech32_encoder.encode(GENESIS_HELPER.as_ref()).unwrap(),
            faucet: bech32_encoder.encode(FAUCET.as_ref()).unwrap(),
        }),
    }))
}

const ALL_ENTITY_TYPES: [EntityType; 17] = [
    // Package
    EntityType::GlobalPackage,
    // System
    EntityType::GlobalConsensusManager,
    EntityType::GlobalValidator,
    // Standard global
    EntityType::GlobalGenericComponent,
    EntityType::GlobalAccount,
    EntityType::GlobalIdentity,
    EntityType::GlobalAccessController,
    // Secp256k1 Virtual Global
    EntityType::GlobalVirtualSecp256k1Account,
    EntityType::GlobalVirtualSecp256k1Identity,
    // Ed25519 Virtual Global Components
    EntityType::GlobalVirtualEd25519Account,
    EntityType::GlobalVirtualEd25519Identity,
    // Fungible-related
    EntityType::GlobalFungibleResourceManager,
    EntityType::InternalFungibleVault,
    // Non-fungible related
    EntityType::GlobalNonFungibleResourceManager,
    EntityType::InternalNonFungibleVault,
    // Internal misc
    EntityType::InternalGenericComponent,
    // Internal key-value-store-like
    EntityType::InternalKeyValueStore,
];

fn to_api_address_type(hrp_set: &HrpSet, entity_type: EntityType) -> models::AddressType {
    // If you add another entity type here, add it to the ALL_ENTITY_TYPES list above.
    // We do it like this in a match statement so that we catch a compile error if a new entity type is added :)
    let api_entity_type = to_api_entity_type(entity_type);

    models::AddressType {
        hrp_prefix: hrp_set.get_entity_hrp(&entity_type).to_string(),
        entity_type: api_entity_type,
        address_byte_prefix: entity_type as i32,
        address_byte_length: NodeId::LENGTH.try_into().unwrap(),
    }
}
