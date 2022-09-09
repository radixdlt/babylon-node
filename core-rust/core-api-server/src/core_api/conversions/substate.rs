use crate::core_api::models;
use models::EntityType;
use radix_engine::engine::Substate;
use radix_engine::model::{
    ComponentInfo, ComponentState, KeyValueStoreEntryWrapper, NonFungibleWrapper, Package,
    ResourceManager, System, Vault,
};
use scrypto::address::Bech32Encoder;
use scrypto::prelude::ResourceType;

use super::MappingError;

pub fn to_api_substate(
    substate: &Substate,
    bech32_encoder: &Bech32Encoder,
) -> Result<models::Substate, MappingError> {
    Ok(match substate {
        Substate::System(system) => to_api_system_substate(system)?,
        Substate::Resource(resource_manager) => to_api_resource_substate(resource_manager),
        Substate::ComponentInfo(component_info) => {
            to_api_component_info_substate(component_info, bech32_encoder)
        }
        Substate::ComponentState(component_state) => {
            to_api_component_state_substate(component_state)
        }
        Substate::Package(validated_package) => to_api_package_substate(validated_package),
        Substate::Vault(vault) => to_api_vault_substate(vault),
        Substate::NonFungible(non_fungible_wrapper) => {
            to_api_non_fungible_substate(non_fungible_wrapper)
        }
        Substate::KeyValueStoreEntry(kv_store_entry_wrapper) => {
            to_api_key_value_story_entry_substate(kv_store_entry_wrapper)
        }
    })
}

fn to_api_system_substate(system: &System) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::SystemSubstate {
        entity_type: EntityType::System,
        epoch: system
            .epoch
            .try_into()
            .map_err(|err| MappingError::Integer {
                message: "System Epoch could not be mapped to i64".to_owned(),
                error: err,
            })?,
    })
}

fn to_api_resource_substate(resource_manager: &ResourceManager) -> models::Substate {
    let (resource_type, fungible_divisibility) = match resource_manager.resource_type() {
        ResourceType::Fungible { divisibility } => (
            models::substate::ResourceType::Fungible,
            Some(divisibility as i32),
        ),
        ResourceType::NonFungible => (models::substate::ResourceType::NonFungible, None),
    };
    // TODO: map method_table, vault_method_table, bucket_method_table, authorization
    models::Substate::ResourceManagerSubstate {
        entity_type: EntityType::ResourceManager,
        resource_type,
        fungible_divisibility,
        metadata: resource_manager
            .metadata()
            .iter()
            .map(|(k, v)| models::ResourceManagerSubstateAllOfMetadata {
                key: k.clone(),
                value: v.clone(),
            })
            .collect(),
        total_supply: resource_manager.total_supply().to_string(),
    }
}

fn to_api_component_info_substate(
    component_info: &ComponentInfo,
    bech32_encoder: &Bech32Encoder,
) -> models::Substate {
    // TODO: map access_rules
    models::Substate::ComponentInfoSubstate {
        entity_type: EntityType::Component,
        package_address: bech32_encoder.encode_package_address(&component_info.package_address()),
        blueprint_name: component_info.blueprint_name().to_string(),
    }
}

fn to_api_component_state_substate(component_state: &ComponentState) -> models::Substate {
    models::Substate::ComponentStateSubstate {
        entity_type: EntityType::Component,
        state: hex::encode(component_state.state()),
    }
}

fn to_api_package_substate(package: &Package) -> models::Substate {
    // TODO: map blueprint_abis
    models::Substate::PackageSubstate {
        entity_type: EntityType::Package,
        code: hex::encode(package.code()),
    }
}

fn to_api_vault_substate(_vault: &Vault) -> models::Substate {
    models::Substate::VaultSubstate {
        entity_type: EntityType::Vault,
    }
}

fn to_api_non_fungible_substate(_non_fungible: &NonFungibleWrapper) -> models::Substate {
    models::Substate::NonFungibleSubstate {
        entity_type: EntityType::ResourceManager,
    }
}

fn to_api_key_value_story_entry_substate(
    _key_value_store_entry: &KeyValueStoreEntryWrapper,
) -> models::Substate {
    models::Substate::KeyValueStoreEntrySubstate {
        entity_type: EntityType::KeyValueStore,
    }
}
