use radix_engine::blueprints::access_controller::AccessControllerSubstate;
use radix_engine::blueprints::account::AccountSubstate;
use radix_engine::blueprints::clock::CurrentTimeRoundedToMinutesSubstate;
use radix_engine::blueprints::epoch_manager::{
    EpochManagerSubstate, Validator, ValidatorSetSubstate, ValidatorSubstate,
};
use radix_engine::blueprints::resource::{
    NonFungible, NonFungibleSubstate, ResourceManagerSubstate, VaultInfoSubstate,
};
use radix_engine::system::node_modules::access_rules::{
    FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::system::type_info::PackageCodeTypeSubstate;
use std::collections::BTreeSet;

use super::*;
use crate::core_api::models;

use radix_engine::types::{
    scrypto_encode, Decimal, KeyValueStoreOffset, NonFungibleStoreOffset, RENodeId,
    ResourceAddress, RoyaltyConfig, SubstateId, SubstateOffset,
};
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate,
    KeyValueStoreEntrySubstate,
};
use radix_engine_interface::api::package::{
    PackageCodeSubstate, PackageInfoSubstate, PackageRoyaltyAccumulatorSubstate,
    PackageRoyaltyConfigSubstate,
};
use radix_engine_interface::api::types::{IndexedScryptoValue, NodeModuleId};
use radix_engine_interface::blueprints::resource::{
    AccessRule, AccessRuleEntry, AccessRuleNode, AccessRulesConfig, LiquidFungibleResource,
    LiquidNonFungibleResource, LockedFungibleResource, LockedNonFungibleResource, MethodKey,
    ProofRule, ResourceType, SoftCount, SoftDecimal, SoftResource, SoftResourceOrNonFungible,
    SoftResourceOrNonFungibleList,
};
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::data::scrypto::model::{
    Address, ComponentAddress, NonFungibleIdType, NonFungibleLocalId,
};
use radix_engine_interface::data::scrypto::{SchemaPath, SchemaSubPath};

use super::MappingError;

pub fn to_api_substate(
    context: &MappingContext,
    substate_id: &SubstateId,
    substate: &PersistedSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(match substate {
        // Shared
        PersistedSubstate::MethodAccessRules(substate) => {
            to_api_access_rules_chain_substate(context, substate)?
        }
        PersistedSubstate::FunctionAccessRules(substate) => {
            to_api_function_access_rules_substate(context, substate)?
        }
        PersistedSubstate::TypeInfo(type_info) => to_api_type_info_substate(context, type_info)?,
        // Specific
        PersistedSubstate::ComponentState(component_state) => {
            to_api_component_state_substate(context, component_state)?
        }
        PersistedSubstate::ComponentRoyaltyConfig(substate) => {
            to_api_component_royalty_config_substate(context, substate)?
        }
        PersistedSubstate::ComponentRoyaltyAccumulator(substate) => {
            to_api_component_royalty_accumulator_substate(substate)?
        }
        PersistedSubstate::ResourceManager(resource_manager) => {
            to_api_resource_manager_substate(context, resource_manager)?
        }
        PersistedSubstate::PackageInfo(package) => to_api_package_info_substate(context, package)?,
        PersistedSubstate::PackageCode(package_code) => {
            to_api_package_code_substate(context, package_code)?
        }
        PersistedSubstate::PackageCodeType(code_type) => {
            to_api_package_code_type_substate(context, code_type)?
        }
        PersistedSubstate::PackageRoyaltyConfig(substate) => {
            to_api_package_royalty_config_substate(context, substate)?
        }
        PersistedSubstate::PackageRoyaltyAccumulator(substate) => {
            to_api_package_royalty_accumulator_substate(substate)?
        }
        PersistedSubstate::EpochManager(epoch_manager) => {
            to_api_epoch_manager_substate(context, epoch_manager)?
        }
        PersistedSubstate::ValidatorSet(validator_set) => {
            to_api_validator_set_substate(context, validator_set)?
        }
        PersistedSubstate::Validator(validator) => to_api_validator_substate(context, validator)?,
        PersistedSubstate::CurrentTimeRoundedToMinutes(substate) => {
            to_api_clock_current_time_rounded_down_to_minutes_substate(substate)?
        }
        PersistedSubstate::VaultInfo(vault_info) => {
            to_api_vault_info_substate(context, vault_info)?
        }
        PersistedSubstate::VaultLiquidFungible(vault_fungible) => {
            to_api_fungible_vault_substate(context, vault_fungible)?
        }
        PersistedSubstate::VaultLiquidNonFungible(vault_non_fungible) => {
            to_api_non_fungible_vault_substate(context, vault_non_fungible)?
        }
        PersistedSubstate::VaultLockedFungible(locked_fungible) => {
            to_api_locked_fungible_vault_substate(context, locked_fungible)?
        }
        PersistedSubstate::VaultLockedNonFungible(locked_non_fungible) => {
            to_api_locked_non_fungible_vault_substate(context, locked_non_fungible)?
        }
        PersistedSubstate::KeyValueStoreEntry(kv_store_entry_wrapper) => {
            to_api_key_value_story_entry_substate(context, substate_id, kv_store_entry_wrapper)?
        }
        PersistedSubstate::NonFungible(non_fungible_wrapper) => {
            to_api_non_fungible_substate(context, substate_id, non_fungible_wrapper)?
        }
        PersistedSubstate::AccessController(access_controller) => {
            to_api_access_controller_substate(context, access_controller)?
        }
        PersistedSubstate::Account(account) => to_api_account_substate(context, account)?,
    })
}

pub fn to_api_access_rules_chain_substate(
    context: &MappingContext,
    substate: &MethodAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let MethodAccessRulesSubstate { access_rules } = substate;

    Ok(models::Substate::AccessRulesSubstate {
        access_rules: Box::new(to_api_access_rules(context, access_rules)?),
    })
}

pub fn to_api_function_access_rules_substate(
    _context: &MappingContext,
    _substate: &FunctionAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    //let FunctionAccessRulesSubstate { .. } = substate;

    Ok(models::Substate::FunctionAccessRulesSubstate {})
}

pub fn to_api_resource_manager_substate(
    _context: &MappingContext,
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
        NonFungibleIdType::Integer => models::NonFungibleIdType::Integer,
        NonFungibleIdType::Bytes => models::NonFungibleIdType::Bytes,
        NonFungibleIdType::UUID => models::NonFungibleIdType::UUID,
    }
}

pub fn to_api_type_info_substate(
    context: &MappingContext,
    substate: &TypeInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let TypeInfoSubstate {
        package_address,
        blueprint_name,
        global,
    } = substate;

    Ok(models::Substate::TypeInfoSubstate {
        package_address: to_api_package_address(context, package_address),
        blueprint_name: blueprint_name.to_string(),
        global: *global,
    })
}

pub fn to_api_access_rules(
    context: &MappingContext,
    access_rules: &AccessRulesConfig,
) -> Result<models::AccessRules, MappingError> {
    Ok(models::AccessRules {
        method_auth: access_rules
            .get_all_method_auth()
            .iter()
            .map(|(key, entry)| to_api_method_auth_entry(context, key, entry))
            .collect::<Result<_, _>>()?,
        grouped_auth: access_rules
            .get_all_grouped_auth()
            .iter()
            .map(|(key, rule)| to_api_grouped_auth_entry(context, key, rule))
            .collect::<Result<_, _>>()?,
        default_auth: Some(to_api_dynamic_access_rule(
            context,
            access_rules.get_default_auth(),
        )?),
        method_auth_mutability: access_rules
            .get_all_method_auth_mutability()
            .iter()
            .map(|(key, access_rule)| {
                to_api_method_auth_mutability_entry(context, key, access_rule)
            })
            .collect::<Result<_, _>>()?,
        grouped_auth_mutability: access_rules
            .get_all_grouped_auth_mutability()
            .iter()
            .map(|(key, rule)| to_api_grouped_auth_entry(context, key, rule))
            .collect::<Result<_, _>>()?,
        default_auth_mutability: Some(to_api_dynamic_access_rule(
            context,
            access_rules.get_default_auth_mutability(),
        )?),
    })
}

pub fn to_api_method_auth_entry(
    context: &MappingContext,
    key: &MethodKey,
    entry: &AccessRuleEntry,
) -> Result<models::MethodAuthEntry, MappingError> {
    let access_rule_reference = match entry {
        AccessRuleEntry::AccessRule(access_rule) => {
            models::AccessRuleReference::RuleAccessRuleReference {
                access_rule: Box::new(to_api_dynamic_access_rule(context, access_rule)?),
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
    context: &MappingContext,
    key: &MethodKey,
    access_rule: &AccessRule,
) -> Result<models::MethodAuthMutabilityEntry, MappingError> {
    Ok(models::MethodAuthMutabilityEntry {
        method: Some(to_api_local_method_reference(key)),
        access_rule: Some(to_api_dynamic_access_rule(context, access_rule)?),
    })
}

pub fn to_api_local_method_reference(key: &MethodKey) -> models::LocalMethodReference {
    models::LocalMethodReference {
        name: key.ident.to_string(),
    }
}

pub fn to_api_grouped_auth_entry(
    context: &MappingContext,
    group_name: &str,
    access_rule: &AccessRule,
) -> Result<models::GroupedAuthEntry, MappingError> {
    Ok(models::GroupedAuthEntry {
        group_name: group_name.to_string(),
        access_rule: Some(to_api_dynamic_access_rule(context, access_rule)?),
    })
}

pub fn to_api_dynamic_access_rule(
    context: &MappingContext,
    access_rule: &AccessRule,
) -> Result<models::AccessRule, MappingError> {
    Ok(match access_rule {
        AccessRule::Protected(access_rule_node) => models::AccessRule::ProtectedAccessRule {
            access_rule: Box::new(to_api_dynamic_access_rule_node(context, access_rule_node)?),
        },
        AccessRule::AllowAll => models::AccessRule::AllowAllAccessRule {},
        AccessRule::DenyAll => models::AccessRule::DenyAllAccessRule {},
    })
}

pub fn to_api_dynamic_access_rule_node(
    context: &MappingContext,
    access_rule: &AccessRuleNode,
) -> Result<models::AccessRuleNode, MappingError> {
    Ok(match access_rule {
        AccessRuleNode::ProofRule(proof_rule) => models::AccessRuleNode::ProofAccessRuleNode {
            proof_rule: Box::new(to_api_dynamic_proof_rule(context, proof_rule)?),
        },
        AccessRuleNode::AnyOf(access_rules) => models::AccessRuleNode::AnyOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_dynamic_access_rule_node(context, ar))
                .collect::<Result<_, _>>()?,
        },
        AccessRuleNode::AllOf(access_rules) => models::AccessRuleNode::AllOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_dynamic_access_rule_node(context, ar))
                .collect::<Result<_, _>>()?,
        },
    })
}

pub fn to_api_dynamic_proof_rule(
    context: &MappingContext,
    proof_rule: &ProofRule,
) -> Result<models::ProofRule, MappingError> {
    Ok(match proof_rule {
        ProofRule::Require(resource) => models::ProofRule::RequireProofRule {
            resource: Box::new(to_api_dynamic_resource_descriptor(context, resource)?),
        },
        ProofRule::AmountOf(amount, resource) => models::ProofRule::AmountOfProofRule {
            amount: Box::new(to_api_dynamic_amount_from_soft_decimal(amount)?),
            resource: Box::new(to_api_dynamic_resource_descriptor_from_resource(
                context, resource,
            )?),
        },
        ProofRule::AllOf(resources) => models::ProofRule::AllOfProofRule {
            list: Box::new(to_api_dynamic_resource_descriptor_list(context, resources)?),
        },
        ProofRule::AnyOf(resources) => models::ProofRule::AnyOfProofRule {
            list: Box::new(to_api_dynamic_resource_descriptor_list(context, resources)?),
        },
        ProofRule::CountOf(count, resources) => models::ProofRule::CountOfProofRule {
            count: Box::new(to_api_dynamic_count_from_soft_count(count)?),
            list: Box::new(to_api_dynamic_resource_descriptor_list(context, resources)?),
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
                    message: format!("Could not translate count into i32: {err:?}"),
                })?,
        },
        SoftCount::Dynamic(schema_path) => models::DynamicCount::SchemaPathDynamicCount {
            schema_path: to_api_schema_path(schema_path)?,
        },
    })
}

pub fn to_api_dynamic_resource_descriptor_list(
    context: &MappingContext,
    resource_list: &SoftResourceOrNonFungibleList,
) -> Result<models::DynamicResourceDescriptorList, MappingError> {
    Ok(match resource_list {
        SoftResourceOrNonFungibleList::Static(resources) => {
            models::DynamicResourceDescriptorList::ListDynamicResourceDescriptorList {
                resources: resources
                    .iter()
                    .map(|r| to_api_dynamic_resource_descriptor(context, r))
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
    context: &MappingContext,
    resource: &SoftResource,
) -> Result<models::DynamicResourceDescriptor, MappingError> {
    Ok(match resource {
        SoftResource::Static(resource) => {
            models::DynamicResourceDescriptor::ResourceDynamicResourceDescriptor {
                resource_address: to_api_resource_address(context, resource),
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
    context: &MappingContext,
    resource: &SoftResourceOrNonFungible,
) -> Result<models::DynamicResourceDescriptor, MappingError> {
    Ok(match resource {
        SoftResourceOrNonFungible::StaticNonFungible(nf) => {
            models::DynamicResourceDescriptor::NonFungibleDynamicResourceDescriptor {
                resource_address: to_api_resource_address(context, &nf.resource_address()),
                non_fungible_id: Box::new(to_api_non_fungible_id(nf.local_id())),
            }
        }
        SoftResourceOrNonFungible::StaticResource(resource) => {
            models::DynamicResourceDescriptor::ResourceDynamicResourceDescriptor {
                resource_address: to_api_resource_address(context, resource),
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

pub fn to_api_active_validator(
    context: &MappingContext,
    address: &ComponentAddress,
    validator: &Validator,
) -> models::ActiveValidator {
    models::ActiveValidator {
        address: to_api_component_address(context, address),
        key: Box::new(to_api_ecdsa_secp256k1_public_key(&validator.key)),
        stake: to_api_decimal(&validator.stake),
    }
}

pub fn to_api_non_fungible_id(non_fungible_id: &NonFungibleLocalId) -> models::NonFungibleId {
    models::NonFungibleId {
        simple_rep: non_fungible_id.to_string(),
        id_type: to_api_fungible_id_type(&non_fungible_id.id_type()),
        sbor_hex: to_hex(scrypto_encode(non_fungible_id).unwrap()),
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
                    message: format!("Couldn't map usize to u64: {err:?}"),
                }
            })?),
        },
    })
}

pub fn to_api_component_state_substate(
    context: &MappingContext,
    component_state: &ComponentStateSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::ComponentStateSubstate {
        data_struct: Box::new(to_api_data_struct(context, &component_state.raw)?),
    })
}

fn scrypto_value_to_api_data_struct(
    context: &MappingContext,
    scrypto_value: IndexedScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let entities = extract_entities(context, &scrypto_value)?;
    Ok(models::DataStruct {
        struct_data: Box::new(scrypto_value_to_api_sbor_data(
            context,
            scrypto_value.as_slice(),
            scrypto_value.as_value(),
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
    context: &MappingContext,
    struct_scrypto_value: &IndexedScryptoValue,
) -> Result<Entities, MappingError> {
    let owned_entities = struct_scrypto_value
        .owned_node_ids()
        .iter()
        .map(|node_id| -> Result<models::EntityReference, MappingError> {
            Ok(MappedEntityId::try_from(*node_id)?.into())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let referenced_entities: Result<Vec<_>, _> = struct_scrypto_value
        .global_references()
        .iter()
        .map(|addr| {
            let address: Address = (*addr).into();
            let reference = to_global_entity_reference(context, &address)?;
            Ok(reference)
        })
        .collect();

    Ok(Entities {
        owned_entities,
        referenced_entities: referenced_entities?,
    })
}

pub fn to_api_component_royalty_config_substate(
    _context: &MappingContext,
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
    substate: &ComponentRoyaltyAccumulatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentRoyaltyAccumulatorSubstate { royalty } = substate;

    Ok(models::Substate::ComponentRoyaltyAccumulatorSubstate {
        vault_entity: Box::new(to_api_entity_reference(RENodeId::Object(
            royalty.vault_id(),
        ))?),
    })
}

pub fn to_api_package_info_substate(
    context: &MappingContext,
    substate: &PackageInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageInfoSubstate {
        dependent_resources,
        dependent_components,
        schema,
    } = substate;

    Ok(models::Substate::PackageInfoSubstate {
        dependent_resources: dependent_resources
            .iter()
            .map(|address| to_api_resource_address(context, address))
            .collect(),
        dependent_components: dependent_components
            .iter()
            .map(|address| to_api_component_address(context, address))
            .collect(),
        blueprints: schema
            .blueprints
            .iter()
            .map(|(blueprint_name, abi)| {
                let blueprint_data = models::BlueprintData {
                    // TODO: Whilst an SBOR-encoded ABI is probably most useful for consumers using the ABI,
                    //       we should probably at some point also map this to something more human-intelligible.
                    //       But let's wait till SBOR schema changes have finalized first.
                    abi: Box::new(scrypto_bytes_to_api_sbor_data(
                        context,
                        &scrypto_encode(abi).unwrap(),
                    )?),
                };
                Ok((blueprint_name.to_owned(), blueprint_data))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_package_code_substate(
    _context: &MappingContext,
    substate: &PackageCodeSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageCodeSubstate { code } = substate;

    Ok(models::Substate::PackageCodeSubstate {
        code_hex: to_hex(code),
    })
}

pub fn to_api_package_code_type_substate(
    _context: &MappingContext,
    substate: &PackageCodeTypeSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::PackageCodeTypeSubstate {
        code_type: match substate {
            PackageCodeTypeSubstate::Native => "native".to_string(),
            PackageCodeTypeSubstate::Wasm => "wasm".to_string(),
        },
    })
}

pub fn to_api_package_royalty_config_substate(
    _context: &MappingContext,
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
    substate: &PackageRoyaltyAccumulatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageRoyaltyAccumulatorSubstate { royalty } = substate;

    Ok(models::Substate::PackageRoyaltyAccumulatorSubstate {
        vault_entity: Box::new(to_api_entity_reference(RENodeId::Object(
            royalty.vault_id(),
        ))?),
    })
}

pub fn to_api_validator_set_substate(
    context: &MappingContext,
    substate: &ValidatorSetSubstate,
) -> Result<models::Substate, MappingError> {
    let ValidatorSetSubstate {
        validator_set,
        epoch,
    } = substate;

    let validator_set = validator_set
        .iter()
        .map(|(address, validator)| to_api_active_validator(context, address, validator))
        .collect();
    Ok(models::Substate::ValidatorSetSubstate {
        validator_set,
        epoch: to_api_epoch(context, *epoch)?,
    })
}

pub fn to_api_validator_substate(
    context: &MappingContext,
    substate: &ValidatorSubstate,
) -> Result<models::Substate, MappingError> {
    let ValidatorSubstate {
        manager,
        address,
        key,
        is_registered,

        unstake_nft,
        liquidity_token,
        stake_xrd_vault_id,
        pending_xrd_withdraw_vault_id,
    } = substate;

    let owned_stake_vault_id = MappedEntityId::try_from(RENodeId::Object(*stake_xrd_vault_id))?;
    let owned_unstake_vault_id =
        MappedEntityId::try_from(RENodeId::Object(*pending_xrd_withdraw_vault_id))?;

    Ok(models::Substate::ValidatorSubstate {
        epoch_manager_address: to_api_component_address(context, manager),
        validator_address: to_api_component_address(context, address),
        public_key: Box::new(to_api_ecdsa_secp256k1_public_key(key)),
        stake_vault: Box::new(owned_stake_vault_id.into()),
        unstake_vault: Box::new(owned_unstake_vault_id.into()),
        unstake_claim_token_resource_address: to_api_resource_address(context, unstake_nft),
        liquid_stake_unit_resource_address: to_api_resource_address(context, liquidity_token),
        is_registered: *is_registered,
    })
}

pub fn to_api_epoch_manager_substate(
    context: &MappingContext,
    substate: &EpochManagerSubstate,
) -> Result<models::Substate, MappingError> {
    let EpochManagerSubstate {
        address,
        epoch,
        round,
        rounds_per_epoch,
        num_unstake_epochs,
    } = substate;

    Ok(models::Substate::EpochManagerSubstate {
        address: to_api_component_address(context, address),
        epoch: to_api_epoch(context, *epoch)?,
        round: to_api_round(*round)?,
        rounds_per_epoch: to_api_round(*rounds_per_epoch)?,
        num_unstake_epochs: to_api_epoch(context, *num_unstake_epochs)?,
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
        timestamp_rounded_down_to_minute: Box::new(to_api_instant_from_safe_timestamp(
            *current_time_rounded_to_minutes_ms,
        )?),
    })
}

pub fn to_api_vault_info_substate(
    context: &MappingContext,
    vault: &VaultInfoSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::VaultInfoSubstate {
        resource_address: to_api_resource_address(context, &vault.resource_address),
    })
}

pub fn to_api_fungible_vault_substate(
    _context: &MappingContext,
    vault: &LiquidFungibleResource,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::VaultFungibleSubstate {
        amount: to_api_decimal(&vault.amount()),
    })
}

pub fn to_api_non_fungible_vault_substate(
    _context: &MappingContext,
    vault: &LiquidNonFungibleResource,
) -> Result<models::Substate, MappingError> {
    let non_fungible_ids = vault.ids().iter().map(to_api_non_fungible_id).collect();

    Ok(models::Substate::VaultNonFungibleSubstate { non_fungible_ids })
}

pub fn to_api_locked_fungible_vault_substate(
    _context: &MappingContext,
    vault: &LockedFungibleResource,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::VaultLockedFungibleSubstate {
        amount: to_api_decimal(&vault.amount()),
    })
}

pub fn to_api_locked_non_fungible_vault_substate(
    _context: &MappingContext,
    vault: &LockedNonFungibleResource,
) -> Result<models::Substate, MappingError> {
    let non_fungible_ids = vault.ids.keys().map(to_api_non_fungible_id).collect();
    Ok(models::Substate::VaultLockedNonFungibleSubstate { non_fungible_ids })
}

pub fn to_api_fungible_resource_amount(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    amount: &Decimal,
) -> Result<models::ResourceAmount, MappingError> {
    Ok(models::ResourceAmount::FungibleResourceAmount {
        resource_address: to_api_resource_address(context, resource_address),
        amount: to_api_decimal(amount),
    })
}

pub fn to_api_non_fungible_resource_amount(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    ids: &BTreeSet<NonFungibleLocalId>,
) -> Result<models::ResourceAmount, MappingError> {
    let non_fungible_ids = ids.iter().map(to_api_non_fungible_id).collect();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: to_api_resource_address(context, resource_address),
        non_fungible_ids,
    })
}

pub fn to_api_non_fungible_substate(
    context: &MappingContext,
    substate_id: &SubstateId,
    substate: &NonFungibleSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let NonFungibleSubstate(non_fungible_option) = substate;

    let nf_id = match substate_id {
        SubstateId(
            RENodeId::NonFungibleStore(..),
            NodeModuleId::SELF,
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
            non_fungible_data: Some(Box::new(to_api_non_fungible_data(context, non_fungible)?)),
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
    context: &MappingContext,
    non_fungible: &NonFungible,
) -> Result<models::NonFungibleData, MappingError> {
    // NOTE - There's currently no schema / validation of non-fungible id data at the moment
    // It's not even guaranteed to be valid SBOR
    // Therefore we attempt to decode it as valid SBOR (becuase it will be in the 99% case), but
    // also provide the raw hex bytes for the cases where it's not
    let immutable_data = non_fungible.immutable_data();
    let mutable_data = non_fungible.mutable_data();
    let immutable_data_sbor_option = to_api_data_struct(context, &immutable_data).ok();
    let mutable_data_sbor_option = to_api_data_struct(context, &mutable_data).ok();
    Ok(models::NonFungibleData {
        immutable_data: immutable_data_sbor_option.map(Box::new),
        immutable_data_raw_hex: to_hex(&immutable_data),
        mutable_data: mutable_data_sbor_option.map(Box::new),
        mutable_data_raw_hex: to_hex(&mutable_data),
    })
}

pub fn to_api_access_controller_substate(
    context: &MappingContext,
    substate: &AccessControllerSubstate,
) -> Result<models::Substate, MappingError> {
    let data = scrypto_encode(substate).unwrap();
    let substate = models::Substate::AccessControllerSubstate {
        data_struct: Box::new(to_api_data_struct(context, &data)?),
    };

    Ok(substate)
}

pub fn to_api_account_substate(
    context: &MappingContext,
    substate: &AccountSubstate,
) -> Result<models::Substate, MappingError> {
    let data = scrypto_encode(substate).unwrap();
    let substate = models::Substate::AccountSubstate {
        data_struct: Box::new(to_api_data_struct(context, &data)?),
    };
    Ok(substate)
}

fn to_api_key_value_story_entry_substate(
    context: &MappingContext,
    substate_id: &SubstateId,
    key_value_store_entry: &KeyValueStoreEntrySubstate,
) -> Result<models::Substate, MappingError> {
    let substate = match substate_id {
        SubstateId(
            RENodeId::KeyValueStore(..),
            NodeModuleId::SELF,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
        ) => match key_value_store_entry {
            KeyValueStoreEntrySubstate::Some(value) => {
                models::Substate::KeyValueStoreEntrySubstate {
                    key_hex: to_hex(key),
                    is_deleted: false,
                    data_struct: Some(Box::new(to_api_data_struct(
                        context,
                        &scrypto_encode(&value).unwrap(),
                    )?)),
                }
            }
            KeyValueStoreEntrySubstate::None => models::Substate::KeyValueStoreEntrySubstate {
                key_hex: to_hex(key),
                is_deleted: true,
                data_struct: None,
            },
        },
        SubstateId(
            _,
            NodeModuleId::Metadata,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
        ) => match key_value_store_entry {
            KeyValueStoreEntrySubstate::Some(value) => models::Substate::MetadataEntrySubstate {
                key_hex: to_hex(key),
                data_struct: Some(Box::new(to_api_data_struct(
                    context,
                    &scrypto_encode(&value).unwrap(),
                )?)),
            },
            KeyValueStoreEntrySubstate::None => models::Substate::MetadataEntrySubstate {
                key_hex: to_hex(key),
                data_struct: None,
            },
        },
        _ => {
            return Err(MappingError::MismatchedSubstateId {
                message: "KVStoreEntry substate was matched with a different substate id"
                    .to_owned(),
            })
        }
    };

    Ok(substate)
}

fn to_api_data_struct(
    context: &MappingContext,
    data: &[u8],
) -> Result<models::DataStruct, MappingError> {
    let scrypto_value =
        IndexedScryptoValue::from_slice(data).map_err(|err| MappingError::ScryptoValueDecode {
            decode_error: err,
            bytes: data.to_vec(),
        })?;
    scrypto_value_to_api_data_struct(context, scrypto_value)
}
