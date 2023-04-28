use radix_engine::blueprints::access_controller::AccessControllerSubstate;
use radix_engine::blueprints::account::AccountSubstate;
use radix_engine::blueprints::clock::ClockSubstate;
use radix_engine::blueprints::epoch_manager::{
    CurrentValidatorSetSubstate, EpochManagerSubstate, SecondaryIndexSubstate, Validator,
    ValidatorSubstate,
};
use radix_engine::blueprints::package::PackageCodeTypeSubstate;
use radix_engine::blueprints::resource::{
    FungibleResourceManagerSubstate, FungibleVaultBalanceSubstate,
    FungibleVaultDivisibilitySubstate, NonFungibleResourceManagerSubstate,
    NonFungibleVaultIdTypeSubstate,
};
use radix_engine::system::node_modules::access_rules::{
    FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use radix_engine::system::node_modules::metadata::MetadataValueSubstate;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine_interface::blueprints::package::{
    PackageCodeSubstate, PackageInfoSubstate, PackageRoyaltySubstate,
};
use radix_engine_interface::schema::{
    BlueprintSchema, FunctionSchema, KeyValueStoreSchema, PackageSchema, Receiver,
    VirtualLazyLoadSchema,
};
use sbor::LocalTypeIndex;
use std::collections::BTreeSet;

use super::*;
use crate::core_api::models;

use radix_engine::types::{
    scrypto_encode, Decimal, ResourceAddress, RoyaltyConfig, ScryptoSchema, ScryptoValue,
};
use radix_engine_common::types::{ComponentAddress, ModuleId, NodeId, SubstateKey};
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate,
};
use radix_engine_interface::blueprints::resource::{
    AccessRule, AccessRuleEntry, AccessRuleNode, AccessRulesConfig, LiquidFungibleResource,
    LiquidNonFungibleResource, MethodKey, ProofRule, ResourceType, SoftCount, SoftDecimal,
    SoftResource, SoftResourceOrNonFungible, SoftResourceOrNonFungibleList,
};
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::data::scrypto::model::{NonFungibleIdType, NonFungibleLocalId};
use radix_engine_interface::data::scrypto::{SchemaPath, SchemaSubPath};
use radix_engine_interface::types::{Blueprint, IndexedScryptoValue, ObjectInfo};
use radix_engine_queries::typed_substate_layout::{
    to_typed_substate_key, to_typed_substate_value, GenericScryptoComponentSubstateValue,
    GenericScryptoSborPayload, NonFungibleVaultBalanceSubstate, TypedAccessControllerSubstateValue,
    TypedAccessRulesModuleSubstateValue, TypedAccountSubstateValue, TypedClockSubstateValue,
    TypedEpochManagerSubstateValue, TypedFungibleResourceManagerSubstateValue,
    TypedFungibleVaultSubstateValue, TypedMetadataModuleSubstateValue,
    TypedNonFungibleResourceManagerSubstateValue, TypedNonFungibleVaultSubstateValue,
    TypedObjectModuleSubstateValue, TypedPackageSubstateValue, TypedRoyaltyModuleSubstateValue,
    TypedSubstateValue, TypedTypeInfoModuleSubstateValue, TypedValidatorSubstateValue,
};

use super::MappingError;

pub fn to_api_substate(
    context: &MappingContext,
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
    value: Vec<u8>,
) -> Result<models::Substate, MappingError> {
    let typed_substate_key = to_typed_substate_key(
        node_id.entity_type().unwrap(), /* TODO handle error */
        module_id,
        substate_key,
    )
    .unwrap(); // TODO: handle error

    // TODO: check value_is_mappable?

    let typed_substate_value =
        to_typed_substate_value(&typed_substate_key, value.as_ref()).unwrap(); // TODO: handle error

    let api_substate = match &typed_substate_value {
        TypedSubstateValue::TypeInfoModule(TypedTypeInfoModuleSubstateValue::TypeInfo(
            type_info_substate,
        )) => to_api_type_info_substate(context, type_info_substate)?,
        TypedSubstateValue::AccessRulesModule(
            TypedAccessRulesModuleSubstateValue::MethodAccessRules(method_access_rules_substate),
        ) => to_api_method_access_rules_substate(context, method_access_rules_substate)?,
        TypedSubstateValue::RoyaltyModule(
            TypedRoyaltyModuleSubstateValue::ComponentRoyaltyConfig(
                component_royalty_config_substate,
            ),
        ) => to_api_component_royalty_config_substate(context, component_royalty_config_substate)?,
        TypedSubstateValue::RoyaltyModule(
            TypedRoyaltyModuleSubstateValue::ComponentRoyaltyAccumulator(
                component_royalty_accumulator_substate,
            ),
        ) => to_api_component_royalty_accumulator_substate(component_royalty_accumulator_substate)?,
        TypedSubstateValue::MetadataModule(TypedMetadataModuleSubstateValue::Metadata(
            metadata_value_substate,
        )) => to_api_metadata_value_substate(context, metadata_value_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Package(
            TypedPackageSubstateValue::Info(package_info_substate),
        )) => to_api_package_info_substate(context, package_info_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Package(
            TypedPackageSubstateValue::CodeType(package_code_type_substate),
        )) => to_api_package_code_type_substate(context, package_code_type_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Package(
            TypedPackageSubstateValue::Code(package_code_substate),
        )) => to_api_package_code_substate(context, package_code_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Package(
            TypedPackageSubstateValue::Royalty(package_royalty_substate),
        )) => to_api_package_royalty_substate(context, package_royalty_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Package(
            TypedPackageSubstateValue::FunctionAccessRules(package_function_access_rules_substate),
        )) => to_api_package_function_access_rules_substate(
            context,
            package_function_access_rules_substate,
        )?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::FungibleResource(
            TypedFungibleResourceManagerSubstateValue::ResourceManager(
                fungible_resource_manager_substate,
            ),
        )) => {
            to_api_fungible_resource_manager_substate(context, fungible_resource_manager_substate)?
        }
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::NonFungibleResource(
            TypedNonFungibleResourceManagerSubstateValue::ResourceManager(
                non_fungible_resource_manager_substate,
            ),
        )) => to_api_non_fungible_resource_manager_substate(
            context,
            non_fungible_resource_manager_substate,
        )?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::FungibleVault(
            TypedFungibleVaultSubstateValue::Divisibility(fungible_vault_divisibility_substate),
        )) => to_api_fungible_vault_divisibility_substate(
            context,
            fungible_vault_divisibility_substate,
        )?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::FungibleVault(
            TypedFungibleVaultSubstateValue::Balance(fungible_vault_balance_substate),
        )) => to_api_fungible_vault_balance_substate(context, fungible_vault_balance_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::NonFungibleVault(
            TypedNonFungibleVaultSubstateValue::IdType(non_fungible_vault_id_type_substate),
        )) => to_api_non_fungible_vault_id_type_substate(
            context,
            non_fungible_vault_id_type_substate,
        )?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::NonFungibleVault(
            TypedNonFungibleVaultSubstateValue::Balance(non_fungible_vault_balance_substate),
        )) => to_api_non_fungible_vault_balance_substate(
            context,
            non_fungible_vault_balance_substate,
        )?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::EpochManager(
            TypedEpochManagerSubstateValue::EpochManager(epoch_manager_substate),
        )) => to_api_epoch_manager_substate(context, epoch_manager_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::EpochManager(
            TypedEpochManagerSubstateValue::CurrentValidatorSet(current_validator_set_substate),
        )) => to_api_current_validator_set_substate(context, current_validator_set_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::EpochManager(
            TypedEpochManagerSubstateValue::RegisteredValidatorSet(secondary_index_substate),
        )) => to_api_secondary_index_substate(context, secondary_index_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Clock(
            TypedClockSubstateValue::CurrentTimeRoundedToMinutes(clock_substate),
        )) => to_api_clock_substate(clock_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Validator(
            TypedValidatorSubstateValue::Validator(validator_substate),
        )) => to_api_validator_substate(context, validator_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::Account(
            TypedAccountSubstateValue::Account(account_substate),
        )) => to_api_account_substate(context, account_substate)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::AccessController(
            TypedAccessControllerSubstateValue::AccessController(access_controller_substate),
        )) => to_api_access_controller_substate(context, access_controller_substate)?,
        TypedSubstateValue::ObjectModule(
            TypedObjectModuleSubstateValue::GenericScryptoComponent(
                GenericScryptoComponentSubstateValue::State(generic_scrypto_sbor_payload),
            ),
        ) => to_api_generic_scrypto_sbor_payload(context, generic_scrypto_sbor_payload)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::GenericKeyValueStore(
            generic_scrypto_sbor_payload,
        )) => to_api_generic_scrypto_sbor_payload(context, generic_scrypto_sbor_payload)?,
        TypedSubstateValue::ObjectModule(TypedObjectModuleSubstateValue::GenericIndex(
            generic_scrypto_sbor_payload,
        )) => to_api_generic_scrypto_sbor_payload(context, generic_scrypto_sbor_payload)?,
        TypedSubstateValue::ObjectModule(
            TypedObjectModuleSubstateValue::GenericSortedU16Index(generic_scrypto_sbor_payload),
        ) => to_api_generic_scrypto_sbor_payload(context, generic_scrypto_sbor_payload)?,
    };

    Ok(api_substate)
}

pub fn to_api_generic_scrypto_sbor_payload(
    _context: &MappingContext,
    _payload: &GenericScryptoSborPayload,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_secondary_index_substate(
    _context: &MappingContext,
    substate: &SecondaryIndexSubstate,
) -> Result<models::Substate, MappingError> {
    let SecondaryIndexSubstate { validators: _ } = substate;

    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_current_validator_set_substate(
    context: &MappingContext,
    substate: &CurrentValidatorSetSubstate,
) -> Result<models::Substate, MappingError> {
    let CurrentValidatorSetSubstate { validator_set } = substate;

    let validator_set = validator_set
        .iter()
        .map(|(address, validator)| to_api_active_validator(context, address, validator))
        .collect();
    Ok(models::Substate::ValidatorSetSubstate {
        validator_set,
        epoch: 0, // to_api_epoch(context, *epoch)?, // TODO: remove
    })
}

pub fn to_api_non_fungible_vault_balance_substate(
    _context: &MappingContext,
    _substate: &NonFungibleVaultBalanceSubstate,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_non_fungible_vault_id_type_substate(
    _context: &MappingContext,
    _substate: &NonFungibleVaultIdTypeSubstate,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_fungible_vault_balance_substate(
    _context: &MappingContext,
    _substate: &FungibleVaultBalanceSubstate,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_fungible_vault_divisibility_substate(
    _context: &MappingContext,
    _substate: &FungibleVaultDivisibilitySubstate,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_metadata_value_substate(
    _context: &MappingContext,
    _substate: &MetadataValueSubstate,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_package_function_access_rules_substate(
    _context: &MappingContext,
    _substate: &FunctionAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // TODO: implement me, just a placeholder for now
    Err(MappingError::MismatchedSubstateId {
        message: "TODO".to_string(),
    })
}

pub fn to_api_method_access_rules_substate(
    context: &MappingContext,
    substate: &MethodAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let MethodAccessRulesSubstate {
        access_rules,
        child_blueprint_rules: _,
    } = substate;

    Ok(models::Substate::AccessRulesSubstate {
        access_rules: Box::new(to_api_access_rules(context, access_rules)?),
        /* TODO: add child_blueprint_rules */
    })
}

pub fn to_api_function_access_rules_substate(
    context: &MappingContext,
    substate: &FunctionAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let FunctionAccessRulesSubstate {
        access_rules,
        default_auth,
    } = substate;

    Ok(models::Substate::PackageFunctionAccessRulesSubstate {
        function_auth: access_rules
            .iter()
            .map(|(function_key, access_rule)| {
                Ok(models::PackageFunctionAccessRule {
                    blueprint: function_key.blueprint.to_string(),
                    function_name: function_key.ident.to_string(),
                    access_rule: Some(to_api_dynamic_access_rule(context, access_rule)?),
                })
            })
            .collect::<Result<_, _>>()?,
        default_auth: Box::new(to_api_dynamic_access_rule(context, default_auth)?),
    })
}

pub fn to_api_fungible_resource_manager_substate(
    _context: &MappingContext,
    substate: &FungibleResourceManagerSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let FungibleResourceManagerSubstate {
        divisibility,
        total_supply,
    } = substate;

    Ok(models::Substate::FungibleResourceManagerSubstate {
        divisibility: to_api_u8_as_i32(*divisibility),
        total_supply: to_api_decimal(total_supply),
    })
}

pub fn to_api_non_fungible_resource_manager_substate(
    context: &MappingContext,
    substate: &NonFungibleResourceManagerSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let NonFungibleResourceManagerSubstate {
        total_supply,
        id_type,
        non_fungible_type_index,
        non_fungible_table,
        mutable_fields,
    } = substate;

    let owned_nf_store = MappedEntityId::try_from(*non_fungible_table.as_node_id())?;

    Ok(models::Substate::NonFungibleResourceManagerSubstate {
        total_supply: to_api_decimal(total_supply),
        non_fungible_id_type: to_api_fungible_id_type(id_type),
        non_fungible_data_table: Box::new(owned_nf_store.into()),
        non_fungible_data_type_index: Box::new(to_api_local_type_index(
            context,
            non_fungible_type_index,
        )?),
        non_fungible_data_mutable_fields: mutable_fields
            .iter()
            .map(|field| field.to_string())
            .collect(),
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
    let details = match substate {
        TypeInfoSubstate::Object(ObjectInfo {
            blueprint:
                Blueprint {
                    package_address,
                    blueprint_name,
                },
            global,
            type_parent: _,
        }) => models::TypeInfoDetails::ObjectTypeInfoDetails {
            package_address: to_api_package_address(context, package_address),
            blueprint_name: blueprint_name.to_string(),
            global: *global,
            /* TODO: add type_parent */
        },
        TypeInfoSubstate::KeyValueStore(schema) => {
            models::TypeInfoDetails::KeyValueStoreTypeInfoDetails {
                key_value_store_schema: Box::new(to_api_key_value_store_schema(context, schema)?),
            }
        }
        TypeInfoSubstate::Index => {
            panic!("TODO");
        }
        TypeInfoSubstate::SortedIndex => {
            panic!("TODO");
        }
    };

    Ok(models::Substate::TypeInfoSubstate {
        details: Box::new(details),
    })
}

pub fn to_api_key_value_store_schema(
    context: &MappingContext,
    key_value_store_schema: &KeyValueStoreSchema,
) -> Result<models::KeyValueStoreSchema, MappingError> {
    let KeyValueStoreSchema {
        schema,
        key,
        value,
        can_own,
    } = key_value_store_schema;
    Ok(models::KeyValueStoreSchema {
        schema: Box::new(to_api_sbor_data_from_encodable(context, schema)?),
        key_type: Box::new(to_api_local_type_index(context, key)?),
        value_type: Box::new(to_api_local_type_index(context, value)?),
        can_own: *can_own,
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
            &access_rules.get_default(),
        )?),
        method_auth_mutability: access_rules
            .get_all_method_mutability()
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
            &access_rules.get_default_mutability(),
        )?),
    })
}

pub fn to_api_method_auth_entry(
    context: &MappingContext,
    key: &MethodKey,
    entry: &AccessRuleEntry,
) -> Result<models::MethodAuthEntry, MappingError> {
    let access_rule_reference = to_api_access_rule_reference(context, entry)?;
    Ok(models::MethodAuthEntry {
        method: Some(to_api_local_method_reference(key)),
        access_rule_reference: Some(access_rule_reference),
    })
}

pub fn to_api_access_rule_reference(
    context: &MappingContext,
    entry: &AccessRuleEntry,
) -> Result<models::AccessRuleReference, MappingError> {
    Ok(match entry {
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
    })
}

pub fn to_api_method_auth_mutability_entry(
    context: &MappingContext,
    key: &MethodKey,
    access_rule: &AccessRuleEntry,
) -> Result<models::MethodAuthMutabilityEntry, MappingError> {
    Ok(models::MethodAuthMutabilityEntry {
        method: Some(to_api_local_method_reference(key)),
        access_rule_reference: Some(to_api_access_rule_reference(context, access_rule)?),
    })
}

pub fn to_api_local_method_reference(key: &MethodKey) -> models::LocalMethodReference {
    models::LocalMethodReference {
        name: key.ident.to_string(),
        module: to_api_module_type_from_obj(&key.module_id),
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
    substate: &ComponentStateSubstate,
) -> Result<models::Substate, MappingError> {
    let ComponentStateSubstate(scrypto_value) = substate;
    Ok(models::Substate::ComponentStateSubstate {
        data_struct: Box::new(to_api_data_struct_from_scrypto_value(
            context,
            scrypto_value,
        )?),
    })
}

pub fn to_api_data_struct_from_scrypto_value(
    context: &MappingContext,
    scrypto_value: &ScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let scrypto_value = IndexedScryptoValue::from_typed(scrypto_value);
    to_api_data_struct_from_indexed_scrypto_value(context, scrypto_value)
}

pub fn to_api_data_struct_from_bytes(
    context: &MappingContext,
    data: &[u8],
) -> Result<models::DataStruct, MappingError> {
    let scrypto_value =
        IndexedScryptoValue::from_slice(data).map_err(|err| MappingError::ScryptoValueDecode {
            decode_error: err,
            bytes: data.to_vec(),
        })?;
    to_api_data_struct_from_indexed_scrypto_value(context, scrypto_value)
}

pub fn to_api_data_struct_from_indexed_scrypto_value(
    context: &MappingContext,
    scrypto_value: IndexedScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let entities = extract_entities(context, &scrypto_value)?;
    Ok(models::DataStruct {
        struct_data: Box::new(to_api_sbor_data_from_bytes(
            context,
            scrypto_value.as_slice(),
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
    _context: &MappingContext,
    struct_scrypto_value: &IndexedScryptoValue,
) -> Result<Entities, MappingError> {
    let owned_entities = struct_scrypto_value
        .owned_node_ids()
        .iter()
        .map(|node_id| -> Result<models::EntityReference, MappingError> {
            Ok(MappedEntityId::try_from(*node_id)?.into())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let referenced_entities: Result<Vec<_>, _> = Ok(vec![]);
    /* TODO: fixme
    let referenced_entities: Result<Vec<_>, _> = struct_scrypto_value
        .references()
        .iter()
        .filter_map(|node_id| match renode_id {
            RENodeId::GlobalObject(address) => Some(to_global_entity_reference(context, address)),
            _ => None,
        })
        .collect();
     */

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
    let ComponentRoyaltyAccumulatorSubstate { royalty_vault } = substate;

    let vault_id = match royalty_vault {
        Some(own) => Some(Box::new(to_api_entity_reference(*own.as_node_id())?)),
        None => None,
    };

    Ok(models::Substate::ComponentRoyaltyAccumulatorSubstate {
        vault_entity: vault_id,
    })
}

pub fn to_api_package_info_substate(
    context: &MappingContext,
    substate: &PackageInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageInfoSubstate {
        schema,
        dependent_resources,
        dependent_components,
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
        package_schema: Box::new(to_api_package_schema(context, schema)?),
    })
}

pub fn to_api_package_schema(
    context: &MappingContext,
    package_schema: &PackageSchema,
) -> Result<models::PackageSchema, MappingError> {
    let PackageSchema { blueprints } = package_schema;
    Ok(models::PackageSchema {
        blueprint_schemas: blueprints
            .iter()
            .map(|(blueprint_name, blueprint_schema)| {
                Ok((
                    blueprint_name.to_string(),
                    to_api_blueprint_schema(context, blueprint_schema)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_blueprint_schema(
    context: &MappingContext,
    blueprint_schema: &BlueprintSchema,
) -> Result<models::BlueprintSchema, MappingError> {
    let BlueprintSchema {
        parent: _,
        schema,
        substates,
        functions,
        virtual_lazy_load_functions,
        event_schema,
    } = blueprint_schema;
    Ok(models::BlueprintSchema {
        // TODO: add parent
        schema: Box::new(to_api_schema(context, schema)?),
        substates: substates
            .iter()
            .map(|index| to_api_local_type_index(context, index))
            .collect::<Result<_, _>>()?,
        function_schemas: functions
            .iter()
            .map(|(function_name, function_schema)| {
                Ok((
                    function_name.to_string(),
                    to_api_function_schema(context, function_schema)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        virtual_lazy_load_function_schemas: virtual_lazy_load_functions
            .iter()
            .map(|(system_func_id, virtual_lazy_load_schema)| {
                Ok((
                    system_func_id.to_string(),
                    to_api_virtual_lazy_load_schema(virtual_lazy_load_schema)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        event_schemas: event_schema
            .iter()
            .map(|(event_name, type_index)| {
                Ok((
                    event_name.to_string(),
                    to_api_local_type_index(context, type_index)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_local_type_index(
    _context: &MappingContext,
    local_type_index: &LocalTypeIndex,
) -> Result<models::LocalTypeIndex, MappingError> {
    Ok(match local_type_index {
        LocalTypeIndex::WellKnown(index) => models::LocalTypeIndex {
            kind: models::local_type_index::Kind::WellKnown,
            index: to_api_u8_as_i32(*index),
        },
        LocalTypeIndex::SchemaLocalIndex(index) => models::LocalTypeIndex {
            kind: models::local_type_index::Kind::SchemaLocal,
            index: to_api_u16_as_i32((*index).try_into().map_err(|_| {
                MappingError::IntegerError {
                    message: "Type index too large".to_string(),
                }
            })?),
        },
    })
}

pub fn to_api_function_schema(
    context: &MappingContext,
    function_schema: &FunctionSchema,
) -> Result<models::FunctionSchema, MappingError> {
    let FunctionSchema {
        receiver,
        input,
        output,
        export_name,
    } = function_schema;
    Ok(models::FunctionSchema {
        receiver: match receiver {
            Some(Receiver::SelfRef) => models::function_schema::Receiver::ComponentReadOnly,
            Some(Receiver::SelfRefMut) => models::function_schema::Receiver::ComponentMutable,
            None => models::function_schema::Receiver::Function,
        },
        input: Box::new(to_api_local_type_index(context, input)?),
        output: Box::new(to_api_local_type_index(context, output)?),
        export_name: export_name.to_string(),
    })
}

pub fn to_api_virtual_lazy_load_schema(
    virtual_lazy_load_schema: &VirtualLazyLoadSchema,
) -> Result<models::VirtualLazyLoadSchema, MappingError> {
    Ok(models::VirtualLazyLoadSchema {
        export_name: virtual_lazy_load_schema.export_name.to_string(),
    })
}

pub fn to_api_schema(
    context: &MappingContext,
    schema: &ScryptoSchema,
) -> Result<models::SborData, MappingError> {
    to_api_sbor_data_from_encodable(context, schema)
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

pub fn to_api_package_royalty_substate(
    _context: &MappingContext,
    substate: &PackageRoyaltySubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageRoyaltySubstate {
        royalty_vault,
        blueprint_royalty_configs,
    } = substate;

    let vault_entity = royalty_vault
        .map(|royalty_vault| {
            Ok(Box::new(to_api_entity_reference(
                *royalty_vault.as_node_id(),
            )?))
        })
        .transpose()?;
    Ok(models::Substate::PackageRoyaltySubstate {
        vault_entity,
        blueprint_royalties: blueprint_royalty_configs
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

pub fn to_api_validator_substate(
    context: &MappingContext,
    substate: &ValidatorSubstate,
) -> Result<models::Substate, MappingError> {
    let ValidatorSubstate {
        sorted_key: _,
        key,
        is_registered,

        unstake_nft,
        liquidity_token,
        stake_xrd_vault_id,
        pending_xrd_withdraw_vault_id,
    } = substate;

    let owned_stake_vault_id = MappedEntityId::try_from(*stake_xrd_vault_id.as_node_id())?;
    let owned_unstake_vault_id =
        MappedEntityId::try_from(*pending_xrd_withdraw_vault_id.as_node_id())?;

    Ok(models::Substate::ValidatorSubstate {
        // TODO: fixme (remove epoch man, fix validator address)
        epoch_manager_address: "".to_string(), // to_api_component_address(context, manager),
        validator_address: "".to_string(),     // to_api_component_address(context, address),
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
        epoch,
        round,
        max_validators: _, // TODO: add to api response
        rounds_per_epoch,
        num_unstake_epochs,
    } = substate;
    Ok(models::Substate::EpochManagerSubstate {
        address: "".to_string(), // to_api_component_address(context, address), // TODO: fixme/remove
        validator_owner_resource: "".to_string(), // to_api_resource_address(context, validator_owner_resource),
        epoch: to_api_epoch(context, *epoch)?,
        round: to_api_round(*round)?,
        rounds_per_epoch: to_api_round(*rounds_per_epoch)?,
        num_unstake_epochs: to_api_epoch(context, *num_unstake_epochs)?,
    })
}

pub fn to_api_clock_substate(substate: &ClockSubstate) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ClockSubstate {
        current_time_rounded_to_minutes_ms,
    } = substate;

    Ok(models::Substate::ClockSubstate {
        timestamp_rounded_down_to_minute: Box::new(to_api_instant_from_safe_timestamp(
            *current_time_rounded_to_minutes_ms,
        )?),
    })
}

pub fn to_api_resource_type(
    _context: &MappingContext,
    resource_type: &ResourceType,
) -> models::ResourceType {
    match resource_type {
        ResourceType::Fungible { .. } => models::ResourceType::Fungible,
        ResourceType::NonFungible { .. } => models::ResourceType::NonFungible,
    }
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

pub fn to_api_access_controller_substate(
    context: &MappingContext,
    substate: &AccessControllerSubstate,
) -> Result<models::Substate, MappingError> {
    let data = scrypto_encode(substate).unwrap();
    let substate = models::Substate::AccessControllerSubstate {
        data_struct: Box::new(to_api_data_struct_from_bytes(context, &data)?),
    };

    Ok(substate)
}

pub fn to_api_account_substate(
    context: &MappingContext,
    substate: &AccountSubstate,
) -> Result<models::Substate, MappingError> {
    let data = scrypto_encode(substate).unwrap();
    let substate = models::Substate::AccountSubstate {
        data_struct: Box::new(to_api_data_struct_from_bytes(context, &data)?),
    };
    Ok(substate)
}

// TODO: remove KeyValueStoreEntrySubstate from api?
