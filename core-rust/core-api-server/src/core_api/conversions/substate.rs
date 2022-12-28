use std::collections::BTreeSet;

use super::*;
use crate::core_api::models;
use radix_engine_interface::data::{IndexedScryptoValue, SchemaPath, SchemaSubPath};

use radix_engine::model::{AccessRulesChainSubstate, ComponentInfoSubstate, ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate, CurrentTimeRoundedToMinutesSubstate, EpochManagerSubstate, GlobalAddressSubstate, KeyValueStoreEntrySubstate, MetadataSubstate, NonFungible, NonFungibleSubstate, PackageInfoSubstate, PackageRoyaltyAccumulatorSubstate, PackageRoyaltyConfigSubstate, PersistedSubstate, Resource, ResourceManagerSubstate, Validator, ValidatorSetSubstate, ValidatorSubstate, VaultSubstate};
use radix_engine::types::{
    scrypto_encode, AccessRule, AccessRuleEntry, AccessRuleKey, AccessRuleNode, AccessRules,
    Bech32Encoder, Decimal, GlobalOffset, KeyValueStoreOffset, NonFungibleId,
    NonFungibleIdType, NonFungibleStoreOffset, ProofRule, RENodeId, ResourceAddress, ResourceType,
    RoyaltyConfig, SoftCount, SoftDecimal, SoftResource, SoftResourceOrNonFungible,
    SoftResourceOrNonFungibleList, SubstateId, SubstateOffset, RADIX_TOKEN,
};
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use utils::ContextualDisplay;

use super::MappingError;

pub fn to_api_substate(
    substate_id: &SubstateId,
    substate: &PersistedSubstate,
    bech32_encoder: &Bech32Encoder,
) -> Result<models::Substate, MappingError> {
    Ok(match substate {
        // Shared
        PersistedSubstate::AccessRulesChain(substate) => {
            to_api_access_rules_chain_substate(bech32_encoder, substate)?
        }
        PersistedSubstate::Metadata(substate) => {
            to_api_metadata_substate(bech32_encoder, substate)?
        }
        // Specific
        PersistedSubstate::Global(global) => {
            to_api_global_address_substate(bech32_encoder, substate_id, global)?
        }
        PersistedSubstate::ComponentInfo(component_info) => {
            to_api_component_info_substate(bech32_encoder, component_info)?
        }
        PersistedSubstate::ComponentState(component_state) => {
            to_api_component_state_substate(bech32_encoder, component_state)?
        }
        PersistedSubstate::ComponentRoyaltyConfig(substate) => {
            to_api_component_royalty_config_substate(bech32_encoder, substate)?
        }
        PersistedSubstate::ComponentRoyaltyAccumulator(substate) => {
            to_api_component_royalty_accumulator_substate(bech32_encoder, substate)?
        }
        PersistedSubstate::ResourceManager(resource_manager) => {
            to_api_resource_manager_substate(bech32_encoder, resource_manager)?
        }
        PersistedSubstate::PackageInfo(package) => {
            to_api_package_info_substate(bech32_encoder, package)?
        }
        PersistedSubstate::PackageRoyaltyConfig(substate) => {
            to_api_package_royalty_config_substate(bech32_encoder, substate)?
        }
        PersistedSubstate::PackageRoyaltyAccumulator(substate) => {
            to_api_package_royalty_accumulator_substate(bech32_encoder, substate)?
        }
        PersistedSubstate::EpochManager(epoch_manager) => {
            to_api_epoch_manager_substate(bech32_encoder, epoch_manager)?
        }
        PersistedSubstate::ValidatorSet(validator_set) => {
            to_api_validator_set_substate(bech32_encoder, validator_set)?
        }
        PersistedSubstate::Validator(validator) => {
            to_api_validator_substate(bech32_encoder, validator)?
        }
        PersistedSubstate::CurrentTimeRoundedToMinutes(substate) => {
            to_api_clock_current_time_rounded_down_to_minutes_substate(substate)?
        }
        PersistedSubstate::Vault(vault) => to_api_vault_substate(bech32_encoder, vault)?,
        PersistedSubstate::KeyValueStoreEntry(kv_store_entry_wrapper) => {
            to_api_key_value_story_entry_substate(
                bech32_encoder,
                substate_id,
                kv_store_entry_wrapper,
            )?
        }
        PersistedSubstate::NonFungible(non_fungible_wrapper) => {
            to_api_non_fungible_substate(bech32_encoder, substate_id, non_fungible_wrapper)?
        }
    })
}

pub fn to_api_access_rules_chain_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &AccessRulesChainSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let AccessRulesChainSubstate { access_rules_chain } = substate;

    Ok(models::Substate::AccessRulesChainSubstate {
        chain: access_rules_chain
            .iter()
            .map(|access_rules| to_api_access_rules(bech32_encoder, access_rules))
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_metadata_substate(
    _bech32_encoder: &Bech32Encoder,
    substate: &MetadataSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let MetadataSubstate { metadata } = substate;

    Ok(models::Substate::MetadataSubstate {
        metadata: metadata
            .iter()
            .map(|(key, value)| models::MetadataSubstateAllOfMetadata {
                key: key.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    })
}

fn to_api_global_address_substate(
    bech32_encoder: &Bech32Encoder,
    substate_id: &SubstateId,
    substate: &GlobalAddressSubstate,
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

    Ok(models::Substate::GlobalAddressSubstate {
        target_entity: Box::new(to_api_global_entity_assignment(
            bech32_encoder,
            substate_id,
            global_address,
            substate,
        )?),
    })
}

pub fn to_api_resource_manager_substate(
    _bech32_encoder: &Bech32Encoder,
    resource_manager: &ResourceManagerSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ResourceManagerSubstate {
        resource_type,
        resource_address: _,
        total_supply,
        nf_store_id,
    } = resource_manager;

    let (resource_type, fungible_divisibility, non_fungible_id_type) = match resource_type {
        ResourceType::Fungible { divisibility } => {
            (models::ResourceType::Fungible, Some(*divisibility), None)
        }
        ResourceType::NonFungible { id_type } => {
            (models::ResourceType::NonFungible, None, Some(id_type))
        }
    };
    let owned_nf_store = nf_store_id
        .map(|node_id| MappedEntityId::try_from(RENodeId::NonFungibleStore(node_id)))
        .transpose()?;

    Ok(models::Substate::ResourceManagerSubstate {
        resource_type,
        fungible_divisibility: fungible_divisibility.map(to_api_u8_as_i32),
        non_fungible_id_type: non_fungible_id_type.map(to_api_fungible_id_type),
        total_supply: to_api_decimal(total_supply),
        owned_non_fungible_store: owned_nf_store.map(|entity_id| Box::new(entity_id.into())),
    })
}

pub fn to_api_fungible_id_type(id_type: &NonFungibleIdType) -> models::NonFungibleIdType {
    match id_type {
        NonFungibleIdType::String => models::NonFungibleIdType::String,
        NonFungibleIdType::U32 => models::NonFungibleIdType::U32,
        NonFungibleIdType::U64 => models::NonFungibleIdType::U64,
        NonFungibleIdType::Bytes => models::NonFungibleIdType::Bytes,
        NonFungibleIdType::UUID => models::NonFungibleIdType::UUID,
    }
}

pub fn to_api_component_info_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &ComponentInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentInfoSubstate {
        package_address,
        blueprint_name,
    } = substate;

    Ok(models::Substate::ComponentInfoSubstate {
        package_address: bech32_encoder.encode_package_address_to_string(package_address),
        blueprint_name: blueprint_name.to_string(),
    })
}

pub fn to_api_access_rules(
    bech32_encoder: &Bech32Encoder,
    access_rules: &AccessRules,
) -> Result<models::AccessRules, MappingError> {
    Ok(models::AccessRules {
        method_auth: access_rules
            .get_all_method_auth()
            .iter()
            .map(|(key, entry)| to_api_method_auth_entry(bech32_encoder, key, entry))
            .collect::<Result<_, _>>()?,
        grouped_auth: access_rules
            .get_all_grouped_auth()
            .iter()
            .map(|(key, rule)| to_api_grouped_auth_entry(bech32_encoder, key, rule))
            .collect::<Result<_, _>>()?,
        default_auth: Some(to_api_dynamic_access_rule(
            bech32_encoder,
            access_rules.get_default_auth(),
        )?),
        method_auth_mutability: access_rules
            .get_all_method_auth_mutability()
            .iter()
            .map(|(key, access_rule)| {
                to_api_method_auth_mutability_entry(bech32_encoder, key, access_rule)
            })
            .collect::<Result<_, _>>()?,
        grouped_auth_mutability: access_rules
            .get_all_grouped_auth_mutability()
            .iter()
            .map(|(key, rule)| to_api_grouped_auth_entry(bech32_encoder, key, rule))
            .collect::<Result<_, _>>()?,
        default_auth_mutability: Some(to_api_dynamic_access_rule(
            bech32_encoder,
            access_rules.get_default_auth_mutability(),
        )?),
    })
}

pub fn to_api_method_auth_entry(
    bech32_encoder: &Bech32Encoder,
    key: &AccessRuleKey,
    entry: &AccessRuleEntry,
) -> Result<models::MethodAuthEntry, MappingError> {
    let access_rule_reference = match entry {
        AccessRuleEntry::AccessRule(access_rule) => {
            models::AccessRuleReference::RuleAccessRuleReference {
                access_rule: Box::new(to_api_dynamic_access_rule(bech32_encoder, access_rule)?),
            }
        }
        AccessRuleEntry::Group(group_name) => {
            models::AccessRuleReference::GroupAccessRuleReference {
                group_name: group_name.to_string(),
            }
        }
    };
    Ok(models::MethodAuthEntry {
        method: Some(to_api_local_method_reference(key)),
        access_rule_reference: Some(access_rule_reference),
    })
}

pub fn to_api_method_auth_mutability_entry(
    bech32_encoder: &Bech32Encoder,
    key: &AccessRuleKey,
    access_rule: &AccessRule,
) -> Result<models::MethodAuthMutabilityEntry, MappingError> {
    Ok(models::MethodAuthMutabilityEntry {
        method: Some(to_api_local_method_reference(key)),
        access_rule: Some(to_api_dynamic_access_rule(bech32_encoder, access_rule)?),
    })
}

pub fn to_api_local_method_reference(key: &AccessRuleKey) -> models::LocalMethodReference {
    match key {
        AccessRuleKey::ScryptoMethod(method_name) => {
            models::LocalMethodReference::LocalScryptoMethodReference {
                name: method_name.to_string(),
            }
        }
        AccessRuleKey::Native(native_fn) => {
            models::LocalMethodReference::LocalNativeFunctionReference {
                name: format!("{:?}", native_fn),
            }
        }
        // TODO: Cleanup
        /*
        AccessRuleKey::Native(NativeFn::Method(method)) => {
            models::LocalMethodReference::LocalNativeMethodReference {
                name: format!("{:?}", method),
            }
        }
         */
    }
}

pub fn to_api_grouped_auth_entry(
    bech32_encoder: &Bech32Encoder,
    group_name: &str,
    access_rule: &AccessRule,
) -> Result<models::GroupedAuthEntry, MappingError> {
    Ok(models::GroupedAuthEntry {
        group_name: group_name.to_string(),
        access_rule: Some(to_api_dynamic_access_rule(bech32_encoder, access_rule)?),
    })
}

pub fn to_api_dynamic_access_rule(
    bech32_encoder: &Bech32Encoder,
    access_rule: &AccessRule,
) -> Result<models::AccessRule, MappingError> {
    Ok(match access_rule {
        AccessRule::Protected(access_rule_node) => models::AccessRule::ProtectedAccessRule {
            access_rule: Box::new(to_api_dynamic_access_rule_node(
                bech32_encoder,
                access_rule_node,
            )?),
        },
        AccessRule::AllowAll => models::AccessRule::AllowAllAccessRule {},
        AccessRule::DenyAll => models::AccessRule::DenyAllAccessRule {},
    })
}

pub fn to_api_dynamic_access_rule_node(
    bech32_encoder: &Bech32Encoder,
    access_rule: &AccessRuleNode,
) -> Result<models::AccessRuleNode, MappingError> {
    Ok(match access_rule {
        AccessRuleNode::ProofRule(proof_rule) => models::AccessRuleNode::ProofAccessRuleNode {
            proof_rule: Box::new(to_api_dynamic_proof_rule(bech32_encoder, proof_rule)?),
        },
        AccessRuleNode::AnyOf(access_rules) => models::AccessRuleNode::AnyOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_dynamic_access_rule_node(bech32_encoder, ar))
                .collect::<Result<_, _>>()?,
        },
        AccessRuleNode::AllOf(access_rules) => models::AccessRuleNode::AllOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_dynamic_access_rule_node(bech32_encoder, ar))
                .collect::<Result<_, _>>()?,
        },
    })
}

pub fn to_api_dynamic_proof_rule(
    bech32_encoder: &Bech32Encoder,
    proof_rule: &ProofRule,
) -> Result<models::ProofRule, MappingError> {
    Ok(match proof_rule {
        ProofRule::Require(resource) => models::ProofRule::RequireProofRule {
            resource: Box::new(to_api_dynamic_resource_descriptor(
                bech32_encoder,
                resource,
            )?),
        },
        ProofRule::AmountOf(amount, resource) => models::ProofRule::AmountOfProofRule {
            amount: Box::new(to_api_dynamic_amount_from_soft_decimal(amount)?),
            resource: Box::new(to_api_dynamic_resource_descriptor_from_resource(
                bech32_encoder,
                resource,
            )?),
        },
        ProofRule::AllOf(resources) => models::ProofRule::AllOfProofRule {
            list: Box::new(to_api_dynamic_resource_descriptor_list(
                bech32_encoder,
                resources,
            )?),
        },
        ProofRule::AnyOf(resources) => models::ProofRule::AnyOfProofRule {
            list: Box::new(to_api_dynamic_resource_descriptor_list(
                bech32_encoder,
                resources,
            )?),
        },
        ProofRule::CountOf(count, resources) => models::ProofRule::CountOfProofRule {
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
                non_fungible_id: Box::new(to_api_non_fungible_id(nf.non_fungible_id())),
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

pub fn to_api_ecdsa_secp256k1_public_key(
    key: &EcdsaSecp256k1PublicKey,
) -> models::EcdsaSecp256k1PublicKey {
    models::EcdsaSecp256k1PublicKey {
        key_type: models::PublicKeyType::EcdsaSecp256k1,
        key_hex: to_hex(key.0),
    }
}

pub fn to_api_validator(
    bech32_encoder: &Bech32Encoder,
    validator: &Validator,
) -> models::Validator {
    models::Validator {
        key: Box::new(to_api_ecdsa_secp256k1_public_key(&validator.key)),
        address: bech32_encoder.encode_system_address_to_string(&validator.address),
    }
}

pub fn to_api_non_fungible_id(non_fungible_id: &NonFungibleId) -> models::NonFungibleId {
    models::NonFungibleId {
        simple_rep: non_fungible_id.to_simple_string(),
        id_type: to_api_fungible_id_type(&non_fungible_id.id_type()),
        sbor_hex: to_hex(non_fungible_id.to_vec()),
    }
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
        data_struct: Box::new(to_api_data_struct(bech32_encoder, &component_state.raw)?),
    })
}

fn scrypto_value_to_api_data_struct(
    bech32_encoder: &Bech32Encoder,
    scrypto_value: IndexedScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let entities = extract_entities(bech32_encoder, &scrypto_value)?;
    Ok(models::DataStruct {
        struct_data: Box::new(scrypto_value_to_api_sbor_data(
            bech32_encoder,
            &scrypto_value.raw,
            &scrypto_value.dom,
        )?),
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
    struct_scrypto_value: &IndexedScryptoValue,
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

pub fn to_api_component_royalty_config_substate(
    _bech32_encoder: &Bech32Encoder,
    substate: &ComponentRoyaltyConfigSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentRoyaltyConfigSubstate { royalty_config } = substate;

    Ok(models::Substate::ComponentRoyaltyConfigSubstate {
        royalty_config: Box::new(to_api_royalty_config(royalty_config)),
    })
}

pub fn to_api_royalty_config(royalty_config: &RoyaltyConfig) -> models::RoyaltyConfig {
    models::RoyaltyConfig {
        method_rules: royalty_config
            .rules
            .iter()
            .map(|(method_name, rule)| models::MethodRoyaltyRule {
                method_name: method_name.to_owned(),
                royalty_rule: to_api_royalty_rule(rule),
            })
            .collect(),
        default_rule: to_api_royalty_rule(&royalty_config.default_rule),
    }
}

pub fn to_api_royalty_rule(royalty_rule: &u32) -> i64 {
    to_api_u32_as_i64(*royalty_rule)
}

pub fn to_api_component_royalty_accumulator_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &ComponentRoyaltyAccumulatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentRoyaltyAccumulatorSubstate { royalty } = substate;

    Ok(models::Substate::ComponentRoyaltyAccumulatorSubstate {
        xrd_amount: to_api_xrd_amount(bech32_encoder, royalty)?,
    })
}

pub fn to_api_xrd_amount(
    bech32_encoder: &Bech32Encoder,
    resource: &Resource,
) -> Result<String, MappingError> {
    let Resource::Fungible {
        resource_address,
        divisibility: _,
        amount,
    } = resource else {
        return Err(MappingError::NotXrdError { message: "Resource was not fungible".to_string() });
    };
    if *resource_address != RADIX_TOKEN {
        return Err(MappingError::NotXrdError {
            message: format!(
                "Resource address was {}",
                resource_address.display(bech32_encoder)
            ),
        });
    }
    Ok(to_api_decimal(amount))
}

pub fn to_api_package_info_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &PackageInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageInfoSubstate {
        code,
        blueprint_abis,
    } = substate;

    Ok(models::Substate::PackageInfoSubstate {
        code_hex: to_hex(code),
        blueprints: blueprint_abis
            .iter()
            .map(|(blueprint_name, abi)| {
                let blueprint_data = models::BlueprintData {
                    // TODO: Whilst an SBOR-encoded ABI is probably most useful for consumers using the ABI,
                    //       we should probably at some point also map this to something more human-intelligible.
                    //       But let's wait till SBOR schema changes have finalized first.
                    abi: Box::new(scrypto_bytes_to_api_sbor_data(
                        bech32_encoder,
                        &scrypto_encode(abi).unwrap(),
                    )?),
                };
                Ok((blueprint_name.to_owned(), blueprint_data))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_package_royalty_config_substate(
    _bech32_encoder: &Bech32Encoder,
    substate: &PackageRoyaltyConfigSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageRoyaltyConfigSubstate { royalty_config } = substate;

    Ok(models::Substate::PackageRoyaltyConfigSubstate {
        blueprint_royalties: royalty_config
            .iter()
            .map(
                |(blueprint_name, royalty_config)| models::BlueprintRoyaltyConfig {
                    blueprint_name: blueprint_name.to_string(),
                    royalty_config: Box::new(to_api_royalty_config(royalty_config)),
                },
            )
            .collect(),
    })
}

pub fn to_api_package_royalty_accumulator_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &PackageRoyaltyAccumulatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageRoyaltyAccumulatorSubstate { royalty } = substate;

    Ok(models::Substate::PackageRoyaltyAccumulatorSubstate {
        xrd_amount: to_api_xrd_amount(bech32_encoder, royalty)?,
    })
}

pub fn to_api_validator_set_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &ValidatorSetSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    // TODO: convert validator_set
    let ValidatorSetSubstate {
        validator_set,
        epoch,
    } = substate;

    let validator_set = validator_set
        .iter()
        .map(|v| to_api_validator(bech32_encoder, v))
        .collect();
    Ok(models::Substate::ValidatorSetSubstate {
        validator_set,
        epoch: to_api_epoch(*epoch)?,
    })
}

pub fn to_api_validator_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &ValidatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    // TODO: convert validator_set
    let ValidatorSubstate {
        manager,
        address,
        key,
    } = substate;

    Ok(models::Substate::ValidatorSubstate {
        manager: bech32_encoder.encode_system_address_to_string(manager),
        address: bech32_encoder.encode_system_address_to_string(address),
        key: Box::new(to_api_ecdsa_secp256k1_public_key(key)),
    })
}


pub fn to_api_epoch_manager_substate(
    bech32_encoder: &Bech32Encoder,
    substate: &EpochManagerSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    // TODO: convert validator_set
    let EpochManagerSubstate {
        address,
        epoch,
        round,
        rounds_per_epoch,
    } = substate;

    Ok(models::Substate::EpochManagerSubstate {
        address: bech32_encoder.encode_system_address_to_string(address),
        epoch: to_api_epoch(*epoch)?,
        round: to_api_round(*round)?,
        rounds_per_epoch: to_api_round(*rounds_per_epoch)?,
    })
}

pub fn to_api_clock_current_time_rounded_down_to_minutes_substate(
    substate: &CurrentTimeRoundedToMinutesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let CurrentTimeRoundedToMinutesSubstate {
        current_time_rounded_to_minutes_ms,
    } = substate;

    Ok(models::Substate::ClockCurrentMinuteSubstate {
        timestamp_ms_rounded_down_to_minute: *current_time_rounded_to_minutes_ms,
    })
}

pub fn to_api_vault_substate(
    bech32_encoder: &Bech32Encoder,
    vault: &VaultSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::VaultSubstate {
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
            id_type,
            ids,
        } => to_api_non_fungible_resource_amount(bech32_encoder, resource_address, id_type, ids)?,
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
    _id_type: &NonFungibleIdType,
    ids: &BTreeSet<NonFungibleId>,
) -> Result<models::ResourceAmount, MappingError> {
    let non_fungible_ids = ids.iter().map(to_api_non_fungible_id).collect();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: bech32_encoder.encode_resource_address_to_string(resource_address),
        non_fungible_ids,
    })
}

pub fn to_api_non_fungible_substate(
    bech32_encoder: &Bech32Encoder,
    substate_id: &SubstateId,
    substate: &NonFungibleSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let NonFungibleSubstate(non_fungible_option) = substate;

    let nf_id = match substate_id {
        SubstateId(
            RENodeId::NonFungibleStore(..),
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(nf_id)),
        ) => nf_id,
        _ => {
            return Err(MappingError::MismatchedSubstateId {
                message: "NonFungibleStore substate was matched with a different substate id"
                    .to_owned(),
            })
        }
    };

    Ok(match non_fungible_option {
        Some(non_fungible) => models::Substate::NonFungibleStoreEntrySubstate {
            non_fungible_id: Box::new(to_api_non_fungible_id(nf_id)),
            non_fungible_data: Some(Box::new(to_api_non_fungible_data(
                bech32_encoder,
                non_fungible,
            )?)),
            is_deleted: false,
        },
        None => models::Substate::NonFungibleStoreEntrySubstate {
            non_fungible_id: Box::new(to_api_non_fungible_id(nf_id)),
            non_fungible_data: None,
            is_deleted: true,
        },
    })
}

fn to_api_non_fungible_data(
    bech32_encoder: &Bech32Encoder,
    non_fungible: &NonFungible,
) -> Result<models::NonFungibleData, MappingError> {
    // NOTE - There's currently no schema / validation of non-fungible id data at the moment
    // It's not even guaranteed to be valid SBOR
    // Therefore we attempt to decode it as valid SBOR (becuase it will be in the 99% case), but
    // also provide the raw hex bytes for the cases where it's not
    let immutable_data = non_fungible.immutable_data();
    let mutable_data = non_fungible.mutable_data();
    let immutable_data_sbor_option = to_api_data_struct(bech32_encoder, &immutable_data).ok();
    let mutable_data_sbor_option = to_api_data_struct(bech32_encoder, &mutable_data).ok();
    Ok(models::NonFungibleData {
        immutable_data: immutable_data_sbor_option.map(Box::new),
        immutable_data_raw_hex: to_hex(&immutable_data),
        mutable_data: mutable_data_sbor_option.map(Box::new),
        mutable_data_raw_hex: to_hex(&mutable_data),
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
            key_hex: to_hex(key),
            is_deleted: false,
            data_struct: Some(Box::new(to_api_data_struct(bech32_encoder, data)?)),
        },
        None => models::Substate::KeyValueStoreEntrySubstate {
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
        IndexedScryptoValue::from_slice(data).map_err(|err| MappingError::ScryptoValueDecode {
            decode_error: err,
            bytes: data.to_vec(),
        })?;
    scrypto_value_to_api_data_struct(bech32_encoder, scrypto_value)
}
