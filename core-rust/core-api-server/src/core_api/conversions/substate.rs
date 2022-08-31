use crate::core_api::generated::models;
use radix_engine::engine::Substate;
use radix_engine::model::{ComponentInfo, ComponentState, ResourceManager, ValidatedPackage};
use scrypto::address::Bech32Encoder;
use scrypto::prelude::ResourceType;
use serde::Serialize;

pub fn to_api_substate(substate: &Substate, bech32_encoder: &Bech32Encoder) -> String {
    match substate {
        Substate::System(_system) => to_json(&models::EmptySubstate::new()),
        Substate::Resource(resource_manager) => {
            to_json(&to_api_resource_substate(resource_manager))
        }
        Substate::ComponentInfo(component_info) => to_json(&to_api_component_info_substate(
            component_info,
            bech32_encoder,
        )),
        Substate::ComponentState(component_state) => {
            to_json(&to_api_component_state_substate(component_state))
        }
        Substate::Package(validated_package) => {
            to_json(&to_api_package_substate(validated_package))
        }
        Substate::Vault(_vault) => to_json(&models::EmptySubstate::new()),
        Substate::NonFungible(_non_fungible_wrapper) => to_json(&models::EmptySubstate::new()),
        Substate::KeyValueStoreEntry(_kv_store_entry_wrapper) => {
            to_json(&models::EmptySubstate::new())
        }
    }
}

fn to_api_resource_substate(resource_manager: &ResourceManager) -> models::ResourceSubstate {
    let (resource_type, fungible_divisibility) = match resource_manager.resource_type() {
        ResourceType::Fungible { divisibility } => ("fungible", Some(divisibility as isize)),
        ResourceType::NonFungible => ("non_fungible", None),
    };
    models::ResourceSubstate {
        resource_type: resource_type.to_string(),
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

fn to_api_package_substate(validated_package: &ValidatedPackage) -> models::PackageSubstate {
    models::PackageSubstate {
        code: hex::encode(validated_package.code()),
    }
}

fn to_json<T: ?Sized + Serialize>(value: &T) -> String {
    // TODO: should return 500 error instead of unwrap (but can this even fail?)
    serde_json::to_string(&value).unwrap()
}
