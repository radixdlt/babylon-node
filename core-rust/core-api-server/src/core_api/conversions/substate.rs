use std::cell::Ref;
use std::collections::BTreeSet;

use super::{addressing::*, to_hex};
use crate::core_api::models;
use models::EntityType;

use radix_engine::engine::Substate;
use radix_engine::model::{
    ComponentInfo, ComponentState, KeyValueStoreEntryWrapper, NonFungible, NonFungibleWrapper,
    Package, ResourceContainer, ResourceManager, System, Vault,
};
use radix_engine::types::{Decimal, NonFungibleId, ResourceAddress, ScryptoValue, SubstateId};
use scrypto::address::Bech32Encoder;
use scrypto::prelude::ResourceType;

use super::MappingError;

pub fn to_api_substate(
    substate_id: &SubstateId,
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
            to_api_component_state_substate(component_state)?
        }
        Substate::Package(validated_package) => to_api_package_substate(validated_package),
        Substate::Vault(vault) => to_api_vault_substate(bech32_encoder, vault)?,
        Substate::NonFungible(non_fungible_wrapper) => {
            to_api_non_fungible_substate(substate_id, non_fungible_wrapper)?
        }
        Substate::KeyValueStoreEntry(kv_store_entry_wrapper) => {
            to_api_key_value_story_entry_substate(substate_id, kv_store_entry_wrapper)?
        }
    })
}

fn to_api_system_substate(system: &System) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::SystemSubstate {
        entity_type: EntityType::System,
        epoch: system.epoch,
    })
}

fn to_api_resource_substate(resource_manager: &ResourceManager) -> models::Substate {
    let (resource_type, fungible_divisibility) = match resource_manager.resource_type() {
        ResourceType::Fungible { divisibility } => {
            (models::ResourceType::Fungible, Some(divisibility as u32))
        }
        ResourceType::NonFungible => (models::ResourceType::NonFungible, None),
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

fn to_api_component_state_substate(
    component_state: &ComponentState,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::ComponentStateSubstate {
        entity_type: EntityType::Component,
        data_struct: Box::new(to_api_data_struct(component_state.state())?),
    })
}

fn scrypto_value_to_api_data_struct(
    scrypto_value: ScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let entities = extract_entities(&scrypto_value)?;
    Ok(models::DataStruct {
        struct_data: Box::new(models::SborData {
            data_bytes: to_hex(scrypto_value.raw),
            data_json: serde_json::to_string(&scrypto_value.dom).expect("JSON serialize error"),
        }),
        owned_entities: entities.owned_entities,
        referenced_entities: entities.referenced_entities,
    })
}

struct Entities {
    pub owned_entities: Vec<models::EntityId>,
    pub referenced_entities: Vec<models::EntityId>,
}

fn extract_entities(struct_scrypto_value: &ScryptoValue) -> Result<Entities, MappingError> {
    if !struct_scrypto_value.bucket_ids.is_empty() {
        Err(MappingError::InvalidComponentStateEntities {
            message: "Bucket/s in state".to_owned(),
        })?;
    }
    if !struct_scrypto_value.proof_ids.is_empty() {
        Err(MappingError::InvalidComponentStateEntities {
            message: "Proof/s in state".to_owned(),
        })?;
    }

    let mut owned_entities = Vec::<models::EntityId>::new();
    owned_entities.extend(
        struct_scrypto_value
            .owned_component_addresses
            .iter()
            .map(|x| to_component_entity_id(x).into()),
    );
    owned_entities.extend(
        struct_scrypto_value
            .vault_ids
            .iter()
            .map(|x| to_vault_entity_id(x).into()),
    );
    owned_entities.extend(
        struct_scrypto_value
            .kv_store_ids
            .iter()
            .map(|x| to_key_value_store_entity_id(x).into()),
    );

    let mut referenced_entities = Vec::<models::EntityId>::new();
    referenced_entities.extend(
        struct_scrypto_value
            .refed_component_addresses
            .iter()
            .map(|x| to_component_entity_id(x).into()),
    );
    referenced_entities.extend(
        struct_scrypto_value
            .resource_addresses
            .iter()
            .map(|x| to_resource_entity_id(x).into()),
    );

    Ok(Entities {
        owned_entities,
        referenced_entities,
    })
}

fn to_api_package_substate(package: &Package) -> models::Substate {
    // TODO: map blueprint_abis
    models::Substate::PackageSubstate {
        entity_type: EntityType::Package,
        code: hex::encode(package.code()),
    }
}

fn to_api_vault_substate(
    bech32_encoder: &Bech32Encoder,
    vault: &Vault,
) -> Result<models::Substate, MappingError> {
    let _resource_container = vault.borrow_container();
    Ok(models::Substate::VaultSubstate {
        entity_type: EntityType::Vault,
        resource_amount: Box::new(to_api_resource_amount(
            bech32_encoder,
            vault.borrow_container(),
        )?),
    })
}

fn to_api_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource_container: Ref<ResourceContainer>,
) -> Result<models::ResourceAmount, MappingError> {
    Ok(match *resource_container {
        ResourceContainer::Fungible {
            ref resource_address,
            divisibility: _,
            locked_amounts: _,
            ref liquid_amount,
        } => to_api_fungible_resource_amount(bech32_encoder, resource_address, liquid_amount)?,
        ResourceContainer::NonFungible {
            ref resource_address,
            locked_ids: _,
            ref liquid_ids,
        } => to_api_non_fungible_resource_amount(bech32_encoder, resource_address, liquid_ids)?,
    })
}

fn to_api_fungible_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource_address: &ResourceAddress,
    amount: &Decimal,
) -> Result<models::ResourceAmount, MappingError> {
    let resource_entity =
        to_api_global_entity_id(bech32_encoder, to_resource_entity_id(resource_address))?;
    Ok(models::ResourceAmount::FungibleResourceAmount {
        resource_address: resource_entity.global_address_str,
        amount_subunits: amount.0.to_string(),
    })
}

fn to_api_non_fungible_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource_address: &ResourceAddress,
    ids: &BTreeSet<NonFungibleId>,
) -> Result<models::ResourceAmount, MappingError> {
    let resource_entity =
        to_api_global_entity_id(bech32_encoder, to_resource_entity_id(resource_address))?;
    let nf_ids = ids.iter().map(|nf_id| to_hex(&nf_id.0)).collect::<Vec<_>>();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: resource_entity.global_address_str,
        nf_ids,
    })
}

fn to_api_non_fungible_substate(
    substate_id: &SubstateId,
    non_fungible: &NonFungibleWrapper,
) -> Result<models::Substate, MappingError> {
    let nf_id_bytes = match substate_id {
        SubstateId::NonFungible(_, nf_id) => &nf_id.0,
        _ => Err(MappingError::MismatchedSubstateId {
            message: "KVStoreEntry Substate was matched with a different substate id".to_owned(),
        })?,
    };

    Ok(match &non_fungible.0 {
        Some(non_fungible) => models::Substate::NonFungibleSubstate {
            entity_type: EntityType::KeyValueStore,
            nf_id: to_hex(nf_id_bytes),
            is_deleted: false,
            non_fungible_data: Option::Some(Box::new(to_api_non_fungible_data(non_fungible)?)),
        },
        None => models::Substate::NonFungibleSubstate {
            entity_type: EntityType::KeyValueStore,
            nf_id: to_hex(nf_id_bytes),
            is_deleted: true,
            non_fungible_data: Option::None,
        },
    })
}

fn to_api_non_fungible_data(
    non_fungible: &NonFungible,
) -> Result<models::NonFungibleData, MappingError> {
    Ok(models::NonFungibleData {
        immutable_data: Box::new(to_api_data_struct(&non_fungible.immutable_data())?),
        mutable_data: Box::new(to_api_data_struct(&non_fungible.mutable_data())?),
    })
}

fn to_api_key_value_story_entry_substate(
    substate_id: &SubstateId,
    key_value_store_entry: &KeyValueStoreEntryWrapper,
) -> Result<models::Substate, MappingError> {
    let key = match substate_id {
        SubstateId::KeyValueStoreEntry(_, key) => key,
        _ => Err(MappingError::MismatchedSubstateId {
            message: "KVStoreEntry Substate was matched with a different substate id".to_owned(),
        })?,
    };

    Ok(match &key_value_store_entry.0 {
        Some(data) => models::Substate::KeyValueStoreEntrySubstate {
            entity_type: EntityType::KeyValueStore,
            key: to_hex(key),
            is_deleted: false,
            data_struct: Option::Some(Box::new(to_api_data_struct(data)?)),
        },
        None => models::Substate::KeyValueStoreEntrySubstate {
            entity_type: EntityType::KeyValueStore,
            key: to_hex(key),
            is_deleted: true,
            data_struct: Option::None,
        },
    })
}

fn to_api_data_struct(data: &[u8]) -> Result<models::DataStruct, MappingError> {
    let scrypto_value =
        ScryptoValue::from_slice(data).map_err(|err| MappingError::InvalidSbor {
            decode_error: err,
            bytes: data.to_vec(),
        })?;
    scrypto_value_to_api_data_struct(scrypto_value)
}
