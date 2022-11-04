use std::collections::BTreeSet;

use super::*;
use crate::core_api::models;
use models::EntityType;
use scrypto::resource::{SchemaPath, SchemaSubPath};

use crate::core_api::models::SborData;
use radix_engine::model::{
    ComponentInfoSubstate, ComponentStateSubstate, EpochManagerSubstate, GlobalAddressSubstate,
    HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible, KeyValueStoreEntrySubstate, MethodAuthorization, NonFungible,
    NonFungibleSubstate, PackageSubstate, PersistedSubstate, Resource, ResourceManagerSubstate,
    VaultSubstate,
};
use radix_engine::types::{
    AccessRule, AccessRuleNode, AccessRules, Decimal, GlobalOffset, NonFungibleId, ProofRule,
    ResourceAddress, ResourceMethodAuthKey, ScryptoValue, SoftCount, SoftDecimal, SoftResource,
    SoftResourceOrNonFungible, SoftResourceOrNonFungibleList, SubstateId,
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
        PersistedSubstate::EpochManager(epoch_manager) => {
            to_api_epoch_manager_substate(epoch_manager)?
        }
        PersistedSubstate::ResourceManager(resource_manager) => {
            to_api_resource_manager_substate(bech32_encoder, resource_manager)?
        }
        PersistedSubstate::ComponentInfo(component_info) => {
            to_api_component_info_substate(component_info, bech32_encoder)?
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
        target_entity: Box::new(to_api_global_entity_assignment(
            bech32_encoder,
            global_address,
            global_substate,
        )?),
    })
}

fn to_api_epoch_manager_substate(
    epoch_manager: &EpochManagerSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::EpochManagerSubstate {
        entity_type: EntityType::EpochManager,
        epoch: to_api_epoch(epoch_manager.epoch)?,
    })
}

pub fn to_api_resource_manager_substate(
    bech32_encoder: &Bech32Encoder,
    resource_manager: &ResourceManagerSubstate,
) -> Result<models::Substate, MappingError> {
    let (resource_type, fungible_divisibility) = match resource_manager.resource_type {
        ResourceType::Fungible { divisibility } => {
            (models::ResourceType::Fungible, Some(divisibility as i32))
        }
        ResourceType::NonFungible => (models::ResourceType::NonFungible, None),
    };
    let owned_nf_store = resource_manager
        .nf_store_id
        .map(|node_id| MappedEntityId::try_from(RENodeId::NonFungibleStore(node_id)))
        .transpose()?;

    Ok(models::Substate::ResourceManagerSubstate {
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
        total_supply: to_api_decimal(&resource_manager.total_supply),
        owned_non_fungible_store: owned_nf_store.map(|entity_id| Box::new(entity_id.into())),
        auth_rules: Box::new(models::ResourceManagerSubstateAllOfAuthRules {
            mint: to_api_resource_manager_auth_rule(
                bech32_encoder,
                resource_manager,
                ResourceMethodAuthKey::Mint,
            )?,
            burn: to_api_resource_manager_auth_rule(
                bech32_encoder,
                resource_manager,
                ResourceMethodAuthKey::Burn,
            )?,
            withdraw: to_api_resource_manager_auth_rule(
                bech32_encoder,
                resource_manager,
                ResourceMethodAuthKey::Withdraw,
            )?,
            deposit: to_api_resource_manager_auth_rule(
                bech32_encoder,
                resource_manager,
                ResourceMethodAuthKey::Deposit,
            )?,
            update_metadata: to_api_resource_manager_auth_rule(
                bech32_encoder,
                resource_manager,
                ResourceMethodAuthKey::UpdateMetadata,
            )?,
            update_non_fungible_data: to_api_resource_manager_auth_rule(
                bech32_encoder,
                resource_manager,
                ResourceMethodAuthKey::UpdateNonFungibleData,
            )?,
        }),
    })
}

pub fn to_api_resource_manager_auth_rule(
    bech32_encoder: &Bech32Encoder,
    resource_manager: &ResourceManagerSubstate,
    resource_method: ResourceMethodAuthKey,
) -> Result<Box<models::FixedActionAuthRules>, MappingError> {
    let auth_rule = resource_manager
        .authorization
        .get(&resource_method)
        .ok_or_else(|| MappingError::AuthRuleNotFound {
            message: format!("{:?}", &resource_method),
        })?;

    Ok(Box::new(models::FixedActionAuthRules {
        perform_action: Some(to_api_fixed_authorization(bech32_encoder, &auth_rule.auth)?),
        update_rules: Some(to_api_fixed_authorization(
            bech32_encoder,
            &auth_rule.update_auth,
        )?),
    }))
}

pub fn to_api_fixed_authorization(
    bech32_encoder: &Bech32Encoder,
    authorization: &MethodAuthorization,
) -> Result<models::FixedAuthorization, MappingError> {
    Ok(match authorization {
        MethodAuthorization::Protected(auth_rule) => {
            models::FixedAuthorization::ProtectedFixedAuthorization {
                auth_rule: Box::new(to_api_fixed_auth_rule(bech32_encoder, auth_rule)?),
            }
        }
        MethodAuthorization::AllowAll => models::FixedAuthorization::AllowAllFixedAuthorization {},
        MethodAuthorization::DenyAll => models::FixedAuthorization::DenyAllFixedAuthorization {},
        MethodAuthorization::Unsupported => Err(MappingError::UnsupportedAuthRulePartPersisted {
            message: "MethodAuthorization::Unsupported was persisted".to_string(),
        })?,
    })
}

pub fn to_api_fixed_auth_rule(
    bech32_encoder: &Bech32Encoder,
    auth_rule: &HardAuthRule,
) -> Result<models::FixedAuthRule, MappingError> {
    Ok(match auth_rule {
        HardAuthRule::ProofRule(proof_rule) => models::FixedAuthRule::ProofFixedAuthRule {
            proof_rule: Box::new(to_api_fixed_proof_rule(bech32_encoder, proof_rule)?),
        },
        HardAuthRule::AnyOf(auth_rules) => models::FixedAuthRule::AnyOfFixedAuthRule {
            auth_rules: auth_rules
                .iter()
                .map(|ar| to_api_fixed_auth_rule(bech32_encoder, ar))
                .collect::<Result<_, _>>()?,
        },
        HardAuthRule::AllOf(auth_rules) => models::FixedAuthRule::AllOfFixedAuthRule {
            auth_rules: auth_rules
                .iter()
                .map(|ar| to_api_fixed_auth_rule(bech32_encoder, ar))
                .collect::<Result<_, _>>()?,
        },
    })
}

pub fn to_api_fixed_proof_rule(
    bech32_encoder: &Bech32Encoder,
    proof_rule: &HardProofRule,
) -> Result<models::FixedProofRule, MappingError> {
    Ok(match proof_rule {
        HardProofRule::Require(resource) => models::FixedProofRule::RequireFixedProofRule {
            resource: Box::new(to_api_fixed_resource_descriptor(bech32_encoder, resource)?),
        },
        HardProofRule::AmountOf(amount, resource) => {
            models::FixedProofRule::AmountOfFixedProofRule {
                amount: to_api_decimal_from_hard_decimal(amount)?,
                resource: Box::new(to_api_fixed_resource_descriptor(bech32_encoder, resource)?),
            }
        }
        HardProofRule::AllOf(resources) => models::FixedProofRule::AllOfFixedProofRule {
            resources: to_api_fixed_resource_descriptor_list(bech32_encoder, resources)?,
        },
        HardProofRule::AnyOf(resources) => models::FixedProofRule::AnyOfFixedProofRule {
            resources: to_api_fixed_resource_descriptor_list(bech32_encoder, resources)?,
        },
        HardProofRule::CountOf(count, resources) => models::FixedProofRule::CountOfFixedProofRule {
            count: to_api_count(count)?,
            resources: to_api_fixed_resource_descriptor_list(bech32_encoder, resources)?,
        },
    })
}

pub fn to_api_decimal_from_hard_decimal(
    hard_decimal: &HardDecimal,
) -> Result<String, MappingError> {
    Ok(match hard_decimal {
        HardDecimal::Amount(amount) => to_api_decimal(amount),
        HardDecimal::SoftDecimalNotFound => Err(MappingError::UnsupportedAuthRulePartPersisted {
            message: "HardDecimal::SoftDecimalNotFound was persisted".to_string(),
        })?,
    })
}

pub fn to_api_count(hard_count: &HardCount) -> Result<i32, MappingError> {
    Ok(match hard_count {
        HardCount::Count(count) => {
            (*count)
                .try_into()
                .map_err(|err| MappingError::IntegerError {
                    message: format!("Could not translate count into i32: {:?}", err),
                })?
        }
        HardCount::SoftCountNotFound => Err(MappingError::UnsupportedAuthRulePartPersisted {
            message: "HardCount::SoftCountNotFound was persisted".to_string(),
        })?,
    })
}

pub fn to_api_fixed_resource_descriptor_list(
    bech32_encoder: &Bech32Encoder,
    resource_list: &HardProofRuleResourceList,
) -> Result<Vec<models::FixedResourceDescriptor>, MappingError> {
    Ok(match resource_list {
        HardProofRuleResourceList::List(resources) => resources
            .iter()
            .map(|r| to_api_fixed_resource_descriptor(bech32_encoder, r))
            .collect::<Result<_, _>>()?,
        HardProofRuleResourceList::SoftResourceListNotFound => {
            Err(MappingError::UnsupportedAuthRulePartPersisted {
                message: "HardProofRuleResourceList::SoftResourceListNotFound was persisted"
                    .to_string(),
            })?
        }
    })
}

pub fn to_api_fixed_resource_descriptor(
    bech32_encoder: &Bech32Encoder,
    resource: &HardResourceOrNonFungible,
) -> Result<models::FixedResourceDescriptor, MappingError> {
    Ok(match resource {
        HardResourceOrNonFungible::NonFungible(nf) => {
            models::FixedResourceDescriptor::NonFungibleFixedResourceDescriptor {
                resource_address: bech32_encoder
                    .encode_resource_address_to_string(&nf.resource_address()),
                non_fungible_id_hex: to_hex(nf.non_fungible_id().0),
            }
        }
        HardResourceOrNonFungible::Resource(resource) => {
            models::FixedResourceDescriptor::ResourceFixedResourceDescriptor {
                resource_address: bech32_encoder.encode_resource_address_to_string(resource),
            }
        }
        HardResourceOrNonFungible::SoftResourceNotFound => {
            Err(MappingError::UnsupportedAuthRulePartPersisted {
                message: "HardResourceOrNonFungible::SoftResourceNotFound was persisted"
                    .to_string(),
            })?
        }
    })
}

pub fn to_api_component_info_substate(
    component_info: &ComponentInfoSubstate,
    bech32_encoder: &Bech32Encoder,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::ComponentInfoSubstate {
        entity_type: EntityType::Component,
        package_address: bech32_encoder
            .encode_package_address_to_string(&component_info.package_address),
        blueprint_name: component_info.blueprint_name.to_string(),
        access_rules_layers: component_info
            .access_rules
            .iter()
            .map(|ar| to_api_component_access_rules_layer(bech32_encoder, ar))
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_component_access_rules_layer(
    bech32_encoder: &Bech32Encoder,
    access_rules_layer: &AccessRules,
) -> Result<models::ComponentAccessRulesLayer, MappingError> {
    Ok(models::ComponentAccessRulesLayer {
        method_auth: access_rules_layer
            .iter()
            .map(|(method_name, auth)| {
                Ok((
                    method_name.to_owned(),
                    to_api_dynamic_authorization(bech32_encoder, auth)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        default_auth: Some(to_api_dynamic_authorization(
            bech32_encoder,
            access_rules_layer.get_default(),
        )?),
    })
}

pub fn to_api_dynamic_authorization(
    bech32_encoder: &Bech32Encoder,
    authorization: &AccessRule,
) -> Result<models::DynamicAuthorization, MappingError> {
    Ok(match authorization {
        AccessRule::Protected(auth_rule) => {
            models::DynamicAuthorization::ProtectedDynamicAuthorization {
                auth_rule: Box::new(to_api_dynamic_auth_rule(bech32_encoder, auth_rule)?),
            }
        }
        AccessRule::AllowAll => models::DynamicAuthorization::AllowAllDynamicAuthorization {},
        AccessRule::DenyAll => models::DynamicAuthorization::DenyAllDynamicAuthorization {},
    })
}

pub fn to_api_dynamic_auth_rule(
    bech32_encoder: &Bech32Encoder,
    auth_rule: &AccessRuleNode,
) -> Result<models::DynamicAuthRule, MappingError> {
    Ok(match auth_rule {
        AccessRuleNode::ProofRule(proof_rule) => models::DynamicAuthRule::ProofDynamicAuthRule {
            proof_rule: Box::new(to_api_dynamic_proof_rule(bech32_encoder, proof_rule)?),
        },
        AccessRuleNode::AnyOf(auth_rules) => models::DynamicAuthRule::AnyOfDynamicAuthRule {
            auth_rules: auth_rules
                .iter()
                .map(|ar| to_api_dynamic_auth_rule(bech32_encoder, ar))
                .collect::<Result<_, _>>()?,
        },
        AccessRuleNode::AllOf(auth_rules) => models::DynamicAuthRule::AllOfDynamicAuthRule {
            auth_rules: auth_rules
                .iter()
                .map(|ar| to_api_dynamic_auth_rule(bech32_encoder, ar))
                .collect::<Result<_, _>>()?,
        },
    })
}

pub fn to_api_dynamic_proof_rule(
    bech32_encoder: &Bech32Encoder,
    proof_rule: &ProofRule,
) -> Result<models::DynamicProofRule, MappingError> {
    Ok(match proof_rule {
        ProofRule::Require(resource) => models::DynamicProofRule::RequireDynamicProofRule {
            resource: Box::new(to_api_dynamic_resource_descriptor(
                bech32_encoder,
                resource,
            )?),
        },
        ProofRule::AmountOf(amount, resource) => {
            models::DynamicProofRule::AmountOfDynamicProofRule {
                amount: Box::new(to_api_dynamic_amount_from_soft_decimal(amount)?),
                resource: Box::new(to_api_dynamic_resource_descriptor_from_resource(
                    bech32_encoder,
                    resource,
                )?),
            }
        }
        ProofRule::AllOf(resources) => models::DynamicProofRule::AllOfDynamicProofRule {
            list: Box::new(to_api_dynamic_resource_descriptor_list(
                bech32_encoder,
                resources,
            )?),
        },
        ProofRule::AnyOf(resources) => models::DynamicProofRule::AnyOfDynamicProofRule {
            list: Box::new(to_api_dynamic_resource_descriptor_list(
                bech32_encoder,
                resources,
            )?),
        },
        ProofRule::CountOf(count, resources) => models::DynamicProofRule::CountOfDynamicProofRule {
            count: Box::new(to_api_dynamic_count_from_soft_count(count)?),
            list: Box::new(to_api_dynamic_resource_descriptor_list(
                bech32_encoder,
                resources,
            )?),
        },
    })
}

pub fn to_api_dynamic_amount_from_soft_decimal(
    soft_decimal: &SoftDecimal,
) -> Result<models::DynamicAmount, MappingError> {
    Ok(match soft_decimal {
        SoftDecimal::Static(amount) => models::DynamicAmount::AmountDynamicAmount {
            amount: to_api_decimal(amount),
        },
        SoftDecimal::Dynamic(schema_path) => models::DynamicAmount::SchemaPathDynamicAmount {
            schema_path: to_api_schema_path(schema_path)?,
        },
    })
}

pub fn to_api_dynamic_count_from_soft_count(
    soft_count: &SoftCount,
) -> Result<models::DynamicCount, MappingError> {
    Ok(match soft_count {
        SoftCount::Static(count) => models::DynamicCount::CountDynamicCount {
            count: (*count)
                .try_into()
                .map_err(|err| MappingError::IntegerError {
                    message: format!("Could not translate count into i32: {:?}", err),
                })?,
        },
        SoftCount::Dynamic(schema_path) => models::DynamicCount::SchemaPathDynamicCount {
            schema_path: to_api_schema_path(schema_path)?,
        },
    })
}

pub fn to_api_dynamic_resource_descriptor_list(
    bech32_encoder: &Bech32Encoder,
    resource_list: &SoftResourceOrNonFungibleList,
) -> Result<models::DynamicResourceDescriptorList, MappingError> {
    Ok(match resource_list {
        SoftResourceOrNonFungibleList::Static(resources) => {
            models::DynamicResourceDescriptorList::ListDynamicResourceDescriptorList {
                resources: resources
                    .iter()
                    .map(|r| to_api_dynamic_resource_descriptor(bech32_encoder, r))
                    .collect::<Result<_, _>>()?,
            }
        }
        SoftResourceOrNonFungibleList::Dynamic(schema_path) => {
            models::DynamicResourceDescriptorList::SchemaPathDynamicResourceDescriptorList {
                schema_path: to_api_schema_path(schema_path)?,
            }
        }
    })
}

pub fn to_api_dynamic_resource_descriptor_from_resource(
    bech32_encoder: &Bech32Encoder,
    resource: &SoftResource,
) -> Result<models::DynamicResourceDescriptor, MappingError> {
    Ok(match resource {
        SoftResource::Static(resource) => {
            models::DynamicResourceDescriptor::ResourceDynamicResourceDescriptor {
                resource_address: bech32_encoder.encode_resource_address_to_string(resource),
            }
        }
        SoftResource::Dynamic(schema_path) => {
            models::DynamicResourceDescriptor::SchemaPathDynamicResourceDescriptor {
                schema_path: to_api_schema_path(schema_path)?,
            }
        }
    })
}

pub fn to_api_dynamic_resource_descriptor(
    bech32_encoder: &Bech32Encoder,
    resource: &SoftResourceOrNonFungible,
) -> Result<models::DynamicResourceDescriptor, MappingError> {
    Ok(match resource {
        SoftResourceOrNonFungible::StaticNonFungible(nf) => {
            models::DynamicResourceDescriptor::NonFungibleDynamicResourceDescriptor {
                resource_address: bech32_encoder
                    .encode_resource_address_to_string(&nf.resource_address()),
                non_fungible_id_hex: to_hex(nf.non_fungible_id().0),
            }
        }
        SoftResourceOrNonFungible::StaticResource(resource) => {
            models::DynamicResourceDescriptor::ResourceDynamicResourceDescriptor {
                resource_address: bech32_encoder.encode_resource_address_to_string(resource),
            }
        }
        SoftResourceOrNonFungible::Dynamic(schema_path) => {
            models::DynamicResourceDescriptor::SchemaPathDynamicResourceDescriptor {
                schema_path: to_api_schema_path(schema_path)?,
            }
        }
    })
}

pub fn to_api_schema_path(
    schema_path: &SchemaPath,
) -> Result<Vec<models::SchemaSubpath>, MappingError> {
    let mapped = schema_path
        .0
        .iter()
        .map(to_api_schema_subpath)
        .collect::<Result<_, _>>()?;
    Ok(mapped)
}

pub fn to_api_schema_subpath(
    schema_subpath: &SchemaSubPath,
) -> Result<models::SchemaSubpath, MappingError> {
    Ok(match schema_subpath {
        SchemaSubPath::Field(field) => models::SchemaSubpath::FieldSchemaSubpath {
            field: field.to_owned(),
        },
        SchemaSubPath::Index(index) => models::SchemaSubpath::IndexSchemaSubpath {
            index: to_api_u64_as_string((*index).try_into().map_err(|err| {
                MappingError::IntegerError {
                    message: format!("Couldn't map usize to u64: {:?}", err),
                }
            })?),
        },
    })
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
    let entities = extract_entities(bech32_encoder, &scrypto_value)?;
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
    pub owned_entities: Vec<models::EntityReference>,
    pub referenced_entities: Vec<models::GlobalEntityReference>,
}

fn extract_entities(
    bech32_encoder: &Bech32Encoder,
    struct_scrypto_value: &ScryptoValue,
) -> Result<Entities, MappingError> {
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

    let owned_entities = struct_scrypto_value
        .node_ids()
        .into_iter()
        .map(|node_id| -> Result<models::EntityReference, MappingError> {
            Ok(MappedEntityId::try_from(node_id)?.into())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let referenced_entities = struct_scrypto_value
        .global_references()
        .into_iter()
        .map(|addr| to_global_entity_reference(bech32_encoder, &addr))
        .collect::<Vec<_>>();

    Ok(Entities {
        owned_entities,
        referenced_entities,
    })
}

pub fn to_api_package_substate(package: &PackageSubstate) -> models::Substate {
    models::Substate::PackageSubstate {
        entity_type: EntityType::Package,
        code_hex: to_hex(package.code()),
        blueprints: package
            .blueprint_abis
            .iter()
            .map(|(blueprint_name, abi)| {
                let blueprint_data = models::BlueprintData {
                    // TODO: Whilst an SBOR-encoded ABI is probably most useful for consumers using the ABI,
                    //       we should probably at some point also map this to something more human-intelligible.
                    //       But let's wait till SBOR schema changes have finalized first.
                    abi_hex: to_hex(scrypto_encode(abi)),
                };
                (blueprint_name.to_owned(), blueprint_data)
            })
            .collect(),
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
        amount: to_api_decimal(amount),
    })
}

fn to_api_non_fungible_resource_amount(
    bech32_encoder: &Bech32Encoder,
    resource_address: &ResourceAddress,
    ids: &BTreeSet<NonFungibleId>,
) -> Result<models::ResourceAmount, MappingError> {
    let non_fungible_ids_hex = ids.iter().map(|nf_id| to_hex(&nf_id.0)).collect::<Vec<_>>();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: bech32_encoder.encode_resource_address_to_string(resource_address),
        non_fungible_ids_hex,
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
            entity_type: EntityType::NonFungibleStore,
            non_fungible_id_hex: to_hex(nf_id_bytes),
            is_deleted: false,
            non_fungible_data: Some(Box::new(to_api_non_fungible_data(
                bech32_encoder,
                non_fungible,
            )?)),
        },
        None => models::Substate::NonFungibleSubstate {
            entity_type: EntityType::NonFungibleStore,
            non_fungible_id_hex: to_hex(nf_id_bytes),
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
