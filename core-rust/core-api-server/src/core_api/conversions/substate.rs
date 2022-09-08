use crate::core_api::models;
use radix_engine::engine::Substate;
use radix_engine::model::{ComponentInfo, ComponentState, Package, ResourceManager};
use scrypto::address::Bech32Encoder;
use scrypto::prelude::ResourceType;
use serde::Serialize;

pub fn to_api_substate(
    substate: &Substate,
    bech32_encoder: &Bech32Encoder,
) -> (models::TemporaryUpSubstateJsonPayloadType, String) {
    match substate {
        Substate::System(_system) => (
            models::TemporaryUpSubstateJsonPayloadType::System,
            to_json(&models::EmptySubstate::new()),
        ),
        Substate::Resource(resource_manager) => (
            models::TemporaryUpSubstateJsonPayloadType::Resource,
            to_json(&to_api_resource_substate(resource_manager)),
        ),
        Substate::ComponentInfo(component_info) => (
            models::TemporaryUpSubstateJsonPayloadType::ComponentInfo,
            to_json(&to_api_component_info_substate(
                component_info,
                bech32_encoder,
            )),
        ),
        Substate::ComponentState(component_state) => (
            models::TemporaryUpSubstateJsonPayloadType::ComponentState,
            to_json(&to_api_component_state_substate(component_state)),
        ),
        Substate::Package(validated_package) => (
            models::TemporaryUpSubstateJsonPayloadType::Package,
            to_json(&to_api_package_substate(validated_package)),
        ),
        Substate::Vault(_vault) => (
            models::TemporaryUpSubstateJsonPayloadType::Vault,
            to_json(&models::EmptySubstate::new()),
        ),
        Substate::NonFungible(_non_fungible_wrapper) => (
            models::TemporaryUpSubstateJsonPayloadType::NonFungible,
            to_json(&models::EmptySubstate::new()),
        ),
        Substate::KeyValueStoreEntry(_kv_store_entry_wrapper) => (
            models::TemporaryUpSubstateJsonPayloadType::KeyValueStoreEntry,
            to_json(&models::EmptySubstate::new()),
        ),
    }
}

fn to_api_resource_substate(resource_manager: &ResourceManager) -> models::ResourceSubstate {
    let (resource_type, fungible_divisibility) = match resource_manager.resource_type() {
        ResourceType::Fungible { divisibility } => (
            models::resource_substate::ResourceType::Fungible,
            Some(divisibility as i32),
        ),
        ResourceType::NonFungible => (models::resource_substate::ResourceType::NonFungible, None),
    };
    // TODO: map method_table, vault_method_table, bucket_method_table, authorization
    models::ResourceSubstate {
        resource_type,
        fungible_divisibility,
        metadata: resource_manager
            .metadata()
            .iter()
            .map(|(k, v)| models::ResourceSubstateMetadataInner {
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
) -> models::ComponentInfoSubstate {
    // TODO: map access_rules
    models::ComponentInfoSubstate {
        package_address: bech32_encoder.encode_package_address(&component_info.package_address()),
        blueprint_name: component_info.blueprint_name().to_string(),
    }
}

fn to_api_component_state_substate(
    component_state: &ComponentState,
) -> models::ComponentStateSubstate {
    models::ComponentStateSubstate {
        state: hex::encode(component_state.state()),
    }
}

fn to_api_package_substate(validated_package: &Package) -> models::PackageSubstate {
    // TODO: map blueprint_abis
    models::PackageSubstate {
        code: hex::encode(validated_package.code()),
    }
}

fn to_json<T: ?Sized + Serialize>(value: &T) -> String {
    // TODO: should return 500 error instead of unwrap (but can this even fail?)
    serde_json::to_string(&value).unwrap()
}
