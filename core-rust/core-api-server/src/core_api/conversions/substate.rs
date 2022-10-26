use std::collections::BTreeSet;

use super::*;
use crate::core_api::models;
use models::EntityType;

use crate::core_api::models::SborData;
use radix_engine::model::{
    ComponentInfoSubstate, ComponentStateSubstate, GlobalAddressSubstate,
    KeyValueStoreEntrySubstate, NonFungible, NonFungibleSubstate, PackageSubstate,
    PersistedSubstate, Resource, ResourceManagerSubstate, SystemSubstate, VaultSubstate,
};
use radix_engine::types::{
    Decimal, GlobalOffset, NonFungibleId, ResourceAddress, ScryptoValue, SubstateId,
};
use scrypto::address::Bech32Encoder;
use scrypto::engine::types::{
    KeyValueStoreOffset, NonFungibleStoreOffset, RENodeId, SubstateOffset,
};
use scrypto::prelude::{scrypto_encode, ResourceType};

use super::MappingError;

#[tracing::instrument(skip_all)]
pub fn to_api_substate(
    substate_id: &SubstateId,
    substate: &PersistedSubstate,
    bech32_encoder: &Bech32Encoder,
) -> Result<models::Substate, MappingError> {
    Ok(match substate {
        PersistedSubstate::Global(global) => {
            to_api_global_substate(bech32_encoder, substate_id, global)?
        }
        PersistedSubstate::System(system) => to_api_system_substate(system)?,
        PersistedSubstate::ResourceManager(resource_manager) => {
            to_api_resource_substate(resource_manager)
        }
        PersistedSubstate::ComponentInfo(component_info) => {
            to_api_component_info_substate(component_info, bech32_encoder)
        }
        PersistedSubstate::ComponentState(component_state) => {
            to_api_component_state_substate(bech32_encoder, component_state)?
        }
        PersistedSubstate::Package(package) => to_api_package_substate(package),
        PersistedSubstate::Vault(vault) => to_api_vault_substate(bech32_encoder, vault)?,
        PersistedSubstate::NonFungible(non_fungible_wrapper) => {
            to_api_non_fungible_substate(bech32_encoder, substate_id, non_fungible_wrapper)?
        }
        PersistedSubstate::KeyValueStoreEntry(kv_store_entry_wrapper) => {
            to_api_key_value_story_entry_substate(
                bech32_encoder,
                substate_id,
                kv_store_entry_wrapper,
            )?
        }
    })
}

fn to_api_global_substate(
    bech32_encoder: &Bech32Encoder,
    substate_id: &SubstateId,
    global_substate: &GlobalAddressSubstate,
) -> Result<models::Substate, MappingError> {
    let global_address = match substate_id {
        SubstateId(
            RENodeId::Global(global_address),
            SubstateOffset::Global(GlobalOffset::Global),
        ) => global_address,
        _ => {
            return Err(MappingError::MismatchedSubstateId {
                message: "Global substate was matched with a different substate id".to_owned(),
            })
        }
    };
    Ok(models::Substate::GlobalSubstate {
        entity_type: EntityType::Global,
        target_entity: Box::new(to_api_global_entity_id(
            bech32_encoder,
            global_address,
            global_substate,
        )?),
    })
}

fn to_api_system_substate(system: &SystemSubstate) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::SystemSubstate {
        entity_type: EntityType::System,
        epoch: to_api_epoch(system.epoch)?,
    })
}

pub fn to_api_resource_substate(resource_manager: &ResourceManagerSubstate) -> models::Substate {
    let (resource_type, fungible_divisibility) = match resource_manager.resource_type {
        ResourceType::Fungible { divisibility } => {
            (models::ResourceType::Fungible, Some(divisibility as i32))
        }
        ResourceType::NonFungible => (models::ResourceType::NonFungible, None),
    };
    // TODO: map method_table, vault_method_table, bucket_method_table, authorization
    models::Substate::ResourceManagerSubstate {
        entity_type: EntityType::ResourceManager,
        resource_type,
        fungible_divisibility,
        metadata: resource_manager
            .metadata
            .iter()
            .map(|(k, v)| models::ResourceManagerSubstateAllOfMetadata {
                key: k.clone(),
                value: v.clone(),
            })
            .collect(),
        total_supply_attos: to_api_decimal_attos(&resource_manager.total_supply),
    }
}

pub fn to_api_component_info_substate(
    component_info: &ComponentInfoSubstate,
    bech32_encoder: &Bech32Encoder,
) -> models::Substate {
    // TODO: map access_rules
    models::Substate::ComponentInfoSubstate {
        entity_type: EntityType::Component,
        package_address: bech32_encoder
            .encode_package_address_to_string(&component_info.package_address),
        blueprint_name: component_info.blueprint_name.to_string(),
    }
}

pub fn to_api_component_state_substate(
    bech32_encoder: &Bech32Encoder,
    component_state: &ComponentStateSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::ComponentStateSubstate {
        entity_type: EntityType::Component,
        data_struct: Box::new(to_api_data_struct(bech32_encoder, &component_state.raw)?),
    })
}

fn scrypto_value_to_api_data_struct(
    bech32_encoder: &Bech32Encoder,
    scrypto_value: ScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let entities = extract_entities(&scrypto_value)?;
    Ok(models::DataStruct {
        struct_data: Box::new(SborData {
            data_hex: to_hex(scrypto_value.raw),
            data_json: Some(convert_scrypto_sbor_value_to_json(
                bech32_encoder,
                &scrypto_value.dom,
            )),
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
        return Err(MappingError::InvalidComponentStateEntities {
            message: "Bucket/s in state".to_owned(),
        });
    }
    if !struct_scrypto_value.proof_ids.is_empty() {
        return Err(MappingError::InvalidComponentStateEntities {
            message: "Proof/s in state".to_owned(),
        });
    }

    let mut owned_entities = Vec::<models::EntityId>::new();
    owned_entities.extend(
        struct_scrypto_value
            .component_ids
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

    // TODO - need to fix
    let mut referenced_entities = Vec::<models::EntityId>::new();
    referenced_entities.extend(
        struct_scrypto_value
            .refed_component_addresses
            .iter()
            .map(|x| to_global_component_entity_id(x).into()),
    );
    referenced_entities.extend(
        struct_scrypto_value
            .resource_addresses
            .iter()
            .map(|x| to_global_resource_entity_id(x).into()),
    );

    Ok(Entities {
        owned_entities,
        referenced_entities,
    })
}

pub fn to_api_package_substate(package: &PackageSubstate) -> models::Substate {
    // TODO: map blueprint_abis
    models::Substate::PackageSubstate {
        entity_type: EntityType::Package,
        code_hex: to_hex(package.code()),
    }
}

pub fn to_api_vault_substate(
    bech32_encoder: &Bech32Encoder,
    vault: &VaultSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::VaultSubstate {
        entity_type: EntityType::Vault,
        resource_amount: Box::new(to_api_resource_amount(bech32_encoder, &vault.0)?),
    })
}

fn to_api_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource: &Resource,
) -> Result<models::ResourceAmount, MappingError> {
    Ok(match resource {
        Resource::Fungible {
            ref resource_address,
            divisibility: _,
            amount,
        } => to_api_fungible_resource_amount(bech32_encoder, resource_address, amount)?,
        Resource::NonFungible {
            ref resource_address,
            ids,
        } => to_api_non_fungible_resource_amount(bech32_encoder, resource_address, ids)?,
    })
}

fn to_api_fungible_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource_address: &ResourceAddress,
    amount: &Decimal,
) -> Result<models::ResourceAmount, MappingError> {
    Ok(models::ResourceAmount::FungibleResourceAmount {
        resource_address: bech32_encoder.encode_resource_address_to_string(resource_address),
        amount_attos: to_api_decimal_attos(amount),
    })
}

fn to_api_non_fungible_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource_address: &ResourceAddress,
    ids: &BTreeSet<NonFungibleId>,
) -> Result<models::ResourceAmount, MappingError> {
    let nf_ids_hex = ids.iter().map(|nf_id| to_hex(&nf_id.0)).collect::<Vec<_>>();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: bech32_encoder.encode_resource_address_to_string(resource_address),
        nf_ids_hex,
    })
}

pub fn to_api_non_fungible_substate(
    bech32_encoder: &Bech32Encoder,
    substate_id: &SubstateId,
    non_fungible: &NonFungibleSubstate,
) -> Result<models::Substate, MappingError> {
    let nf_id_bytes = match substate_id {
        SubstateId(
            RENodeId::NonFungibleStore(..),
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(nf_id)),
        ) => &nf_id.0,
        _ => {
            return Err(MappingError::MismatchedSubstateId {
                message: "NonFungibleStore substate was matched with a different substate id"
                    .to_owned(),
            })
        }
    };

    Ok(match &non_fungible.0 {
        Some(non_fungible) => models::Substate::NonFungibleSubstate {
            entity_type: EntityType::KeyValueStore,
            nf_id_hex: to_hex(nf_id_bytes),
            is_deleted: false,
            non_fungible_data: Some(Box::new(to_api_non_fungible_data(
                bech32_encoder,
                non_fungible,
            )?)),
        },
        None => models::Substate::NonFungibleSubstate {
            entity_type: EntityType::KeyValueStore,
            nf_id_hex: to_hex(nf_id_bytes),
            is_deleted: true,
            non_fungible_data: None,
        },
    })
}

fn to_api_non_fungible_data(
    bech32_encoder: &Bech32Encoder,
    non_fungible: &NonFungible,
) -> Result<models::NonFungibleData, MappingError> {
    // TODO: NFT data is no longer a ScryptoValue (but it should be again at some point)
    // remove scrypto_encode call once that happens
    Ok(models::NonFungibleData {
        immutable_data: Box::new(to_api_data_struct(
            bech32_encoder,
            &scrypto_encode(&non_fungible.immutable_data()),
        )?),
        mutable_data: Box::new(to_api_data_struct(
            bech32_encoder,
            &scrypto_encode(&non_fungible.mutable_data()),
        )?),
    })
}

fn to_api_key_value_story_entry_substate(
    bech32_encoder: &Bech32Encoder,
    substate_id: &SubstateId,
    key_value_store_entry: &KeyValueStoreEntrySubstate,
) -> Result<models::Substate, MappingError> {
    let key = match substate_id {
        SubstateId(
            RENodeId::KeyValueStore(..),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
        ) => key,
        _ => {
            return Err(MappingError::MismatchedSubstateId {
                message: "KVStoreEntry substate was matched with a different substate id"
                    .to_owned(),
            })
        }
    };

    Ok(match &key_value_store_entry.0 {
        Some(data) => models::Substate::KeyValueStoreEntrySubstate {
            entity_type: EntityType::KeyValueStore,
            key_hex: to_hex(key),
            is_deleted: false,
            data_struct: Some(Box::new(to_api_data_struct(bech32_encoder, data)?)),
        },
        None => models::Substate::KeyValueStoreEntrySubstate {
            entity_type: EntityType::KeyValueStore,
            key_hex: to_hex(key),
            is_deleted: true,
            data_struct: None,
        },
    })
}

fn to_api_data_struct(
    bech32_encoder: &Bech32Encoder,
    data: &[u8],
) -> Result<models::DataStruct, MappingError> {
    let scrypto_value =
        ScryptoValue::from_slice(data).map_err(|err| MappingError::InvalidSbor {
            decode_error: err,
            bytes: data.to_vec(),
        })?;
    scrypto_value_to_api_data_struct(bech32_encoder, scrypto_value)
}
