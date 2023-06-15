use radix_engine::blueprints::access_controller::AccessControllerSubstate;
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::blueprints::package::PackageCodeTypeSubstate;
use radix_engine::system::node_modules::access_rules::*;
use radix_engine::system::node_modules::metadata::MetadataValueSubstate;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::system::{KeyValueEntrySubstate, SubstateMutability};
use radix_engine_interface::blueprints::account::{AccountDefaultDepositRule, ResourceDepositRule};
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, EpochChangeCondition,
};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::schema::*;

use super::*;
use crate::core_api::models;

use radix_engine::types::*;

use radix_engine_queries::typed_substate_layout::*;

use super::MappingError;

pub fn to_api_substate(
    context: &MappingContext,
    substate_key: &SubstateKey,
    typed_substate_key: &TypedSubstateKey,
    typed_substate_value: &TypedSubstateValue,
) -> Result<models::Substate, MappingError> {
    Ok(match typed_substate_value {
        TypedSubstateValue::TypeInfoModuleFieldValue(TypedTypeInfoModuleFieldValue::TypeInfo(
            type_info_substate,
        )) => to_api_type_info_substate(context, type_info_substate)?,
        TypedSubstateValue::AccessRulesModuleFieldValue(
            TypedAccessRulesModuleFieldValue::MethodAccessRules(method_access_rules_substate),
        ) => to_api_method_access_rules_substate(context, method_access_rules_substate)?,
        TypedSubstateValue::RoyaltyModuleFieldValue(
            TypedRoyaltyModuleFieldValue::ComponentRoyaltyConfig(component_royalty_config_substate),
        ) => to_api_component_royalty_config_substate(context, component_royalty_config_substate)?,
        TypedSubstateValue::RoyaltyModuleFieldValue(
            TypedRoyaltyModuleFieldValue::ComponentRoyaltyAccumulator(
                component_royalty_accumulator_substate,
            ),
        ) => to_api_component_royalty_accumulator_substate(
            context,
            component_royalty_accumulator_substate,
        )?,
        TypedSubstateValue::MetadataModuleEntryValue(metadata_value_substate) => {
            to_api_metadata_value_substate(context, substate_key, metadata_value_substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::Info(package_info_substate),
        )) => to_api_package_info_substate(context, package_info_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::CodeType(package_code_type_substate),
        )) => to_api_package_code_type_substate(context, package_code_type_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::Code(package_code_substate),
        )) => to_api_package_code_substate(context, package_code_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::Royalty(package_royalty_substate),
        )) => to_api_package_royalty_substate(context, package_royalty_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::FunctionAccessRules(package_function_access_rules_substate),
        )) => to_api_package_function_access_rules_substate(
            context,
            package_function_access_rules_substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleResource(
            TypedFungibleResourceManagerFieldValue::Divisibility(
                fungible_resource_manager_divisibility_substate,
            ),
        )) => to_api_fungible_resource_manager_divisibility_substate(
            fungible_resource_manager_divisibility_substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleResource(
            TypedFungibleResourceManagerFieldValue::TotalSupply(
                fungible_resource_manager_total_supply_substate,
            ),
        )) => to_api_fungible_resource_manager_total_supply_substate(
            fungible_resource_manager_total_supply_substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleResource(
            TypedNonFungibleResourceManagerFieldValue::IdType(
                non_fungible_resource_manager_id_type_substate,
            ),
        )) => to_api_non_fungible_resource_manager_id_type_substate(
            non_fungible_resource_manager_id_type_substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleResource(
            TypedNonFungibleResourceManagerFieldValue::TotalSupply(
                non_fungible_resource_manager_total_supply_substate,
            ),
        )) => to_api_non_fungible_resource_manager_total_supply_substate(
            non_fungible_resource_manager_total_supply_substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleResource(
            TypedNonFungibleResourceManagerFieldValue::MutableFields(substate),
        )) => to_api_non_fungible_resource_manager_mutable_fields_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleResourceData(
            substate,
        )) => to_api_non_fungible_resource_manager_data_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleVault(
            TypedFungibleVaultFieldValue::Balance(fungible_vault_balance_substate),
        )) => to_api_fungible_vault_balance_substate(context, fungible_vault_balance_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVaultField(
            TypedNonFungibleVaultFieldValue::Balance(non_fungible_vault_balance_substate),
        )) => to_api_non_fungible_vault_balance_substate(
            context,
            non_fungible_vault_balance_substate,
        )?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::NonFungibleVaultContentsIndexEntry(entry),
        ) => to_api_non_fungible_vault_contents_entry_substate(context, entry)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::ConsensusManager(substate),
        )) => to_api_consensus_manager_state_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::Config(substate),
        )) => to_api_consensus_manager_config_substate(substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::CurrentValidatorSet(substate),
        )) => to_api_current_validator_set_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::CurrentProposalStatistic(substate),
        )) => to_api_current_proposal_statistic_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::CurrentTimeRoundedToMinutes(substate),
        )) => to_api_current_time_rounded_to_minutes_substate(substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::CurrentTime(substate),
        )) => to_api_current_time_substate(substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::ConsensusManagerRegisteredValidatorsByStakeIndexEntry(
                entry,
            ),
        ) => to_api_registered_validator_set_substate(context, entry)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Validator(
            TypedValidatorFieldValue::Validator(validator_substate),
        )) => to_api_validator_substate(context, validator_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Account(
            TypedAccountFieldValue::Account(substate),
        )) => to_api_account_state_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::AccountVaultIndex(
            substate,
        )) => to_api_account_vault_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::AccountResourceDepositRuleIndex(substate),
        ) => to_api_account_deposit_rule_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::AccessController(
            TypedAccessControllerFieldValue::AccessController(access_controller_substate),
        )) => to_api_access_controller_substate(context, access_controller_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::GenericScryptoComponent(
            GenericScryptoComponentFieldValue::State(generic_scrypto_sbor_payload),
        )) => to_api_generic_scrypto_component_state_substate(
            context,
            &generic_scrypto_sbor_payload.data,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::GenericKeyValueStore(
            substate,
        )) => to_api_generic_key_value_store_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::OneResourcePool(
            TypedOneResourcePoolFieldValue::OneResourcePool(substate),
        )) => to_api_one_resource_pool_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::TwoResourcePool(
            TypedTwoResourcePoolFieldValue::TwoResourcePool(substate),
        )) => to_api_two_resource_pool_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::MultiResourcePool(
            TypedMultiResourcePoolFieldValue::MultiResourcePool(substate),
        )) => to_api_multi_resource_pool_substate(context, substate)?,
    })
}

pub fn to_api_one_resource_pool_substate(
    context: &MappingContext,
    substate: &OneResourcePoolSubstate,
) -> Result<models::Substate, MappingError> {
    let OneResourcePoolSubstate {
        vault,
        pool_unit_resource_manager,
    } = substate;
    Ok(models::Substate::OneResourcePoolSubstate {
        vault: Box::new(to_api_entity_reference(context, vault.0.as_node_id())?),
        pool_unit_resource_address: to_api_resource_address(
            context,
            &pool_unit_resource_manager.0,
        )?,
    })
}

pub fn to_api_two_resource_pool_substate(
    context: &MappingContext,
    substate: &TwoResourcePoolSubstate,
) -> Result<models::Substate, MappingError> {
    let TwoResourcePoolSubstate {
        vaults,
        pool_unit_resource_manager,
    } = substate;
    Ok(models::Substate::TwoResourcePoolSubstate {
        vaults: vaults
            .iter()
            .map(|(resource_address, vault)| to_api_pool_vault(context, resource_address, vault))
            .collect::<Result<Vec<_>, _>>()?,
        pool_unit_resource_address: to_api_resource_address(
            context,
            &pool_unit_resource_manager.0,
        )?,
    })
}

pub fn to_api_multi_resource_pool_substate(
    context: &MappingContext,
    substate: &MultiResourcePoolSubstate,
) -> Result<models::Substate, MappingError> {
    let MultiResourcePoolSubstate {
        vaults,
        pool_unit_resource_manager,
    } = substate;
    Ok(models::Substate::MultiResourcePoolSubstate {
        vaults: vaults
            .iter()
            .map(|(resource_address, vault)| to_api_pool_vault(context, resource_address, vault))
            .collect::<Result<Vec<_>, _>>()?,
        pool_unit_resource_address: to_api_resource_address(
            context,
            &pool_unit_resource_manager.0,
        )?,
    })
}

pub fn to_api_pool_vault(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    vault: &Vault,
) -> Result<models::PoolVault, MappingError> {
    Ok(models::PoolVault {
        vault: Box::new(to_api_entity_reference(context, vault.0.as_node_id())?),
        resource_address: to_api_resource_address(context, resource_address)?,
    })
}

pub fn to_api_account_state_substate(
    _context: &MappingContext,
    substate: &AccountSubstate,
) -> Result<models::Substate, MappingError> {
    let AccountSubstate {
        default_deposit_rule,
    } = substate;
    Ok(models::Substate::AccountFieldStateSubstate {
        default_deposit_rule: match default_deposit_rule {
            AccountDefaultDepositRule::Accept => models::DefaultDepositRule::Accept,
            AccountDefaultDepositRule::Reject => models::DefaultDepositRule::Reject,
            AccountDefaultDepositRule::AllowExisting => models::DefaultDepositRule::AllowExisting,
        },
    })
}

pub fn to_api_account_vault_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    vault_substate: &KeyValueEntrySubstate<Own>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountVaultIndexKey(resource_address)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Account Vault Key".to_string() });
    };
    Ok(models::Substate::AccountVaultIndexEntrySubstate {
        resource_address: to_api_resource_address(context, resource_address)?,
        vault: vault_substate
            .value
            .as_ref()
            .map(|v| -> Result<_, MappingError> {
                Ok(Box::new(to_api_entity_reference(context, v.as_node_id())?))
            })
            .transpose()?,
    })
}

pub fn to_api_account_deposit_rule_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &Option<ResourceDepositRule>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountVaultIndexKey(resource_address)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Account Deposit Rule Key".to_string() });
    };
    Ok(models::Substate::AccountDepositRuleIndexEntrySubstate {
        resource_address: to_api_resource_address(context, resource_address)?,
        deposit_rule: substate.map(|rule| match rule {
            ResourceDepositRule::Neither => models::DepositRule::Neither,
            ResourceDepositRule::Allowed => models::DepositRule::Allowed,
            ResourceDepositRule::Disallowed => models::DepositRule::Disallowed,
        }),
    })
}

pub fn to_api_generic_scrypto_component_state_substate(
    context: &MappingContext,
    data: &Vec<u8>,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::GenericScryptoComponentFieldStateSubstate {
            data_struct: Box::new(to_api_data_struct_from_bytes(context, data.as_ref())?),
        },
    )
}

pub fn to_api_generic_key_value_store_substate(
    context: &MappingContext,
    substate: &KeyValueEntrySubstate<ScryptoRawValue<'_>>,
) -> Result<models::Substate, MappingError> {
    let KeyValueEntrySubstate { value, mutability } = substate;
    let (is_deleted, data_struct) = match value {
        Some(value) => (
            false,
            Some(Box::new(to_api_data_struct_from_scrypto_raw_value(
                context, value,
            )?)),
        ),
        None => (true, None),
    };
    let is_mutable = match mutability {
        SubstateMutability::Mutable => true,
        SubstateMutability::Immutable => false,
    };
    Ok(models::Substate::GenericKeyValueStoreEntrySubstate {
        is_deleted,
        data_struct,
        is_mutable,
    })
}

pub fn to_api_registered_validator_set_substate(
    context: &MappingContext,
    substate: &EpochRegisteredValidatorByStakeEntry,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate {
            active_validator: Box::new(to_api_active_validator(
                context,
                &substate.component_address,
                &substate.validator,
            )?),
        },
    )
}

pub fn to_api_current_validator_set_substate(
    context: &MappingContext,
    substate: &CurrentValidatorSetSubstate,
) -> Result<models::Substate, MappingError> {
    let CurrentValidatorSetSubstate { validator_set } = substate;
    let validator_set = validator_set
        .validators_by_stake_desc
        .iter()
        .map(|(address, validator)| to_api_active_validator(context, address, validator))
        .collect::<Result<_, _>>()?;
    Ok(models::Substate::ConsensusManagerFieldCurrentValidatorSetSubstate { validator_set })
}

pub fn to_api_current_proposal_statistic_substate(
    _context: &MappingContext,
    substate: &CurrentProposalStatisticSubstate,
) -> Result<models::Substate, MappingError> {
    let CurrentProposalStatisticSubstate {
        validator_statistics,
    } = substate;
    Ok(
        models::Substate::ConsensusManagerFieldCurrentProposalStatisticSubstate {
            completed: validator_statistics
                .iter()
                .map(|s| to_api_ten_trillion_capped_u64(s.made, "completed_proposals"))
                .collect::<Result<_, _>>()?,
            missed: validator_statistics
                .iter()
                .map(|s| to_api_ten_trillion_capped_u64(s.missed, "missed_proposals"))
                .collect::<Result<_, _>>()?,
        },
    )
}

pub fn to_api_metadata_value_substate(
    context: &MappingContext,
    substate_key: &SubstateKey,
    substate: &MetadataValueSubstate,
) -> Result<models::Substate, MappingError> {
    let SubstateKey::Map(key_bytes) = substate_key else {
        return Err(MappingError::InvalidMetadataKey { message: "Was not a map key".to_string() });
    };
    let field_name: String =
        scrypto_decode(key_bytes).map_err(|_| MappingError::InvalidMetadataKey {
            message: "Was not a string".to_string(),
        })?;
    let MetadataValueSubstate { value, mutability } = substate;
    let (is_deleted, data_struct) = match value {
        Some(entry) => (
            false,
            Some(Box::new(to_api_data_struct_from_bytes(
                context,
                &scrypto_encode(entry).unwrap(),
            )?)),
        ),
        None => (true, None),
    };
    let is_mutable = match mutability {
        SubstateMutability::Mutable => true,
        SubstateMutability::Immutable => false,
    };
    Ok(models::Substate::MetadataModuleEntrySubstate {
        field_name,
        is_deleted,
        data_struct,
        is_mutable,
    })
}

pub fn to_api_method_access_rules_substate(
    context: &MappingContext,
    substate: &MethodAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let MethodAccessRulesSubstate {
        roles,
        role_mutability,
    } = substate;

    Ok(
        models::Substate::AccessRulesModuleFieldAccessRulesSubstate {
            roles: to_api_role_rules(context, roles)?,
            role_mutability: to_api_mutability_rules(context, role_mutability)?,
        },
    )
}

pub fn to_api_package_function_access_rules_substate(
    context: &MappingContext,
    substate: &FunctionAccessRulesSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let FunctionAccessRulesSubstate { access_rules } = substate;

    Ok(models::Substate::PackageFieldFunctionAccessRulesSubstate {
        function_auth: access_rules
            .iter()
            .map(|(function_key, access_rule)| {
                Ok(models::PackageFunctionAccessRule {
                    blueprint: function_key.blueprint.to_string(),
                    function_name: function_key.ident.to_string(),
                    access_rule: Some(to_api_access_rule(context, access_rule)?),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_non_fungible_id_type(id_type: &NonFungibleIdType) -> models::NonFungibleIdType {
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
                BlueprintId {
                    package_address,
                    blueprint_name,
                },
            global,
            outer_object,
            instance_schema,
            features,
        }) => models::TypeInfoDetails::ObjectTypeInfoDetails {
            package_address: to_api_package_address(context, package_address)?,
            blueprint_name: blueprint_name.to_string(),
            global: *global,
            outer_object: outer_object
                .map(|o| to_api_global_address(context, &o))
                .transpose()?,
            instance_schema: instance_schema
                .as_ref()
                .map(|instance_schema| {
                    Ok(Box::new(to_api_instance_schema(context, instance_schema)?))
                })
                .transpose()?,
            features: features.iter().cloned().collect(),
        },
        TypeInfoSubstate::KeyValueStore(key_value_store_info) => {
            models::TypeInfoDetails::KeyValueStoreTypeInfoDetails {
                key_value_store_info: Box::new(to_api_key_value_store_info(
                    context,
                    key_value_store_info,
                )?),
            }
        }
        TypeInfoSubstate::GlobalAddressReservation(global_address) => {
            models::TypeInfoDetails::GlobalAddressReservationTypeInfoDetails {
                global_address: to_api_global_address(context, global_address)?,
            }
        }
        TypeInfoSubstate::GlobalAddressPhantom(global_address_phantom) => {
            let GlobalAddressPhantom {
                blueprint_id:
                    BlueprintId {
                        package_address,
                        blueprint_name,
                    },
            } = global_address_phantom;
            models::TypeInfoDetails::GlobalAddressPhantomTypeInfoDetails {
                global_address_phantom: Box::new(models::GlobalAddressPhantom {
                    package_address: to_api_package_address(context, package_address)?,
                    blueprint_name: blueprint_name.to_string(),
                }),
            }
        }
    };

    Ok(models::Substate::TypeInfoModuleFieldTypeInfoSubstate {
        details: Box::new(details),
    })
}

pub fn to_api_instance_schema(
    context: &MappingContext,
    instance_schema: &InstanceSchema,
) -> Result<models::InstanceSchema, MappingError> {
    Ok(models::InstanceSchema {
        schema: Box::new(to_api_scrypto_schema(context, &instance_schema.schema)?),
        provided_types: instance_schema
            .type_index
            .iter()
            .map(|local_type_index| to_api_local_type_index(context, local_type_index))
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_key_value_store_info(
    context: &MappingContext,
    key_value_store_info: &KeyValueStoreInfo,
) -> Result<models::KeyValueStoreInfo, MappingError> {
    let KeyValueStoreInfo { schema } = key_value_store_info;
    Ok(models::KeyValueStoreInfo {
        kv_store_schema: Box::new(to_api_key_value_store_schema(context, schema)?),
    })
}

pub fn to_api_key_value_store_schema(
    context: &MappingContext,
    key_value_store_schema: &KeyValueStoreSchema,
) -> Result<models::KeyValueStoreSchema, MappingError> {
    let KeyValueStoreSchema {
        key,
        value,
        can_own,
        schema,
    } = key_value_store_schema;
    Ok(models::KeyValueStoreSchema {
        schema: Box::new(to_api_scrypto_schema(context, schema)?),
        key_type: Box::new(to_api_local_type_index(context, key)?),
        value_type: Box::new(to_api_local_type_index(context, value)?),
        can_own: *can_own,
    })
}

pub fn to_api_role_rules(
    context: &MappingContext,
    rules: &BTreeMap<RoleKey, AccessRule>,
) -> Result<Vec<models::RoleRule>, MappingError> {
    rules
        .iter()
        .map(|(key, rule)| to_api_role_rule(context, key, rule))
        .collect::<Result<_, _>>()
}

pub fn to_api_role_rule(
    context: &MappingContext,
    key: &RoleKey,
    rule: &AccessRule,
) -> Result<models::RoleRule, MappingError> {
    Ok(models::RoleRule {
        role_key: key.key.clone(),
        access_rule: Some(to_api_access_rule(context, rule)?),
    })
}

pub fn to_api_mutability_rules(
    context: &MappingContext,
    rules: &BTreeMap<RoleKey, (RoleList, bool)>,
) -> Result<Vec<models::MutabilityRule>, MappingError> {
    rules
        .iter()
        .map(|(key, (updaters, mutable))| to_api_mutability_rule(context, key, updaters, *mutable))
        .collect::<Result<_, _>>()
}

pub fn to_api_mutability_rule(
    _context: &MappingContext,
    key: &RoleKey,
    updaters: &RoleList,
    mutable: bool,
) -> Result<models::MutabilityRule, MappingError> {
    Ok(models::MutabilityRule {
        role_key: key.key.clone(),
        updater_role_keys: updaters.clone().to_list(),
        updaters_mutable: mutable,
    })
}

pub fn to_api_access_rule(
    context: &MappingContext,
    access_rule: &AccessRule,
) -> Result<models::AccessRule, MappingError> {
    Ok(match access_rule {
        AccessRule::Protected(access_rule_node) => models::AccessRule::ProtectedAccessRule {
            access_rule: Box::new(to_api_access_rule_node(context, access_rule_node)?),
        },
        AccessRule::AllowAll => models::AccessRule::AllowAllAccessRule {},
        AccessRule::DenyAll => models::AccessRule::DenyAllAccessRule {},
    })
}

pub fn to_api_access_rule_node(
    context: &MappingContext,
    access_rule: &AccessRuleNode,
) -> Result<models::AccessRuleNode, MappingError> {
    Ok(match access_rule {
        AccessRuleNode::ProofRule(proof_rule) => models::AccessRuleNode::ProofAccessRuleNode {
            proof_rule: Box::new(to_api_proof_rule(context, proof_rule)?),
        },
        AccessRuleNode::AnyOf(access_rules) => models::AccessRuleNode::AnyOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_access_rule_node(context, ar))
                .collect::<Result<_, _>>()?,
        },
        AccessRuleNode::AllOf(access_rules) => models::AccessRuleNode::AllOfAccessRuleNode {
            access_rules: access_rules
                .iter()
                .map(|ar| to_api_access_rule_node(context, ar))
                .collect::<Result<_, _>>()?,
        },
    })
}

pub fn to_api_proof_rule(
    context: &MappingContext,
    proof_rule: &ProofRule,
) -> Result<models::ProofRule, MappingError> {
    Ok(match proof_rule {
        ProofRule::Require(resource_or_non_fungible) => models::ProofRule::RequireProofRule {
            requirement: Box::new(to_api_requirement(context, resource_or_non_fungible)?),
        },
        ProofRule::AmountOf(amount, resource) => models::ProofRule::AmountOfProofRule {
            amount: to_api_decimal(amount),
            resource: to_api_resource_address(context, resource)?,
        },
        ProofRule::AllOf(resource_or_non_fungible_list) => models::ProofRule::AllOfProofRule {
            list: to_api_resource_or_non_fungible_list(context, resource_or_non_fungible_list)?,
        },
        ProofRule::AnyOf(resource_or_non_fungible_list) => models::ProofRule::AnyOfProofRule {
            list: to_api_resource_or_non_fungible_list(context, resource_or_non_fungible_list)?,
        },
        ProofRule::CountOf(count, resource_or_non_fungible_list) => {
            models::ProofRule::CountOfProofRule {
                count: *count as i32,
                list: to_api_resource_or_non_fungible_list(context, resource_or_non_fungible_list)?,
            }
        }
    })
}

pub fn to_api_resource_or_non_fungible_list(
    context: &MappingContext,
    requirement_list: &[ResourceOrNonFungible],
) -> Result<Vec<models::Requirement>, MappingError> {
    let mut res = Vec::new();
    for resource_or_non_fungible in requirement_list.iter() {
        res.push(to_api_requirement(context, resource_or_non_fungible)?);
    }
    Ok(res)
}

pub fn to_api_requirement(
    context: &MappingContext,
    requirement: &ResourceOrNonFungible,
) -> Result<models::Requirement, MappingError> {
    Ok(match requirement {
        ResourceOrNonFungible::Resource(resource_address) => {
            models::Requirement::ResourceRequirement {
                resource: to_api_resource_address(context, resource_address)?,
            }
        }
        ResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
            models::Requirement::NonFungibleRequirement {
                non_fungible: Box::new(to_api_non_fungible_global_id(
                    context,
                    non_fungible_global_id,
                )?),
            }
        }
    })
}

pub fn to_api_ecdsa_secp256k1_public_key(
    key: &Secp256k1PublicKey,
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
) -> Result<models::ActiveValidator, MappingError> {
    Ok(models::ActiveValidator {
        address: to_api_component_address(context, address)?,
        key: Box::new(to_api_ecdsa_secp256k1_public_key(&validator.key)),
        stake: to_api_decimal(&validator.stake),
    })
}

pub fn to_api_non_fungible_local_id(
    non_fungible_id: &NonFungibleLocalId,
) -> models::NonFungibleLocalId {
    models::NonFungibleLocalId {
        simple_rep: non_fungible_id.to_string(),
        id_type: to_api_non_fungible_id_type(&non_fungible_id.id_type()),
        sbor_hex: to_hex(scrypto_encode(non_fungible_id).unwrap()),
    }
}

pub fn to_api_non_fungible_global_id(
    context: &MappingContext,
    non_fungible_global_id: &NonFungibleGlobalId,
) -> Result<models::NonFungibleGlobalId, MappingError> {
    Ok(models::NonFungibleGlobalId {
        resource_address: to_api_resource_address(
            context,
            &non_fungible_global_id.resource_address(),
        )?,
        local_id: Box::new(to_api_non_fungible_local_id(
            non_fungible_global_id.local_id(),
        )),
    })
}

pub fn to_api_component_state_substate(
    context: &MappingContext,
    scrypto_value: &ScryptoValue,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::GenericScryptoComponentFieldStateSubstate {
            data_struct: Box::new(to_api_data_struct_from_scrypto_value(
                context,
                scrypto_value,
            )?),
        },
    )
}

pub fn to_api_data_struct_from_scrypto_value(
    context: &MappingContext,
    scrypto_value: &ScryptoValue,
) -> Result<models::DataStruct, MappingError> {
    let scrypto_value = IndexedScryptoValue::from_typed(scrypto_value);
    to_api_data_struct_from_indexed_scrypto_value(context, scrypto_value)
}

pub fn to_api_data_struct_from_scrypto_raw_value(
    context: &MappingContext,
    scrypto_raw_value: &ScryptoRawValue<'_>,
) -> Result<models::DataStruct, MappingError> {
    let scrypto_value =
        IndexedScryptoValue::from_vec(scrypto_encode(scrypto_raw_value).unwrap()).unwrap();
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
    pub referenced_entities: Vec<models::EntityReference>,
}

fn extract_entities(
    context: &MappingContext,
    struct_scrypto_value: &IndexedScryptoValue,
) -> Result<Entities, MappingError> {
    let owned_entities = struct_scrypto_value
        .owned_nodes()
        .iter()
        .map(|node_id| to_api_entity_reference(context, node_id))
        .collect::<Result<Vec<_>, _>>()?;

    let referenced_entities = struct_scrypto_value
        .references()
        .iter()
        .map(|node_id| to_api_entity_reference(context, node_id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Entities {
        owned_entities,
        referenced_entities,
    })
}

pub fn to_api_component_royalty_config_substate(
    _context: &MappingContext,
    substate: &ComponentRoyaltyConfigSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentRoyaltyConfigSubstate { royalty_config } = substate;

    Ok(models::Substate::RoyaltyModuleFieldConfigSubstate {
        royalty_config: Box::new(to_api_royalty_config(royalty_config)),
    })
}

pub fn to_api_royalty_config(royalty_config: &RoyaltyConfig) -> models::RoyaltyConfig {
    models::RoyaltyConfig {
        method_rules: royalty_config
            .rules
            .iter()
            .map(|(method_name, royalty_amount)| models::MethodRoyaltyRule {
                method_name: method_name.to_owned(),
                royalty_amount: to_api_royalty_amount(royalty_amount).map(Box::new),
            })
            .collect(),
    }
}

pub fn to_api_royalty_amount(royalty_rule: &RoyaltyAmount) -> Option<models::RoyaltyAmount> {
    match royalty_rule {
        RoyaltyAmount::Free => None,
        RoyaltyAmount::Xrd(amount) => Some(models::RoyaltyAmount::new(
            to_api_decimal(amount),
            models::royalty_amount::Unit::XRD,
        )),
        RoyaltyAmount::Usd(amount) => Some(models::RoyaltyAmount::new(
            to_api_decimal(amount),
            models::royalty_amount::Unit::USD,
        )),
    }
}

pub fn to_api_component_royalty_accumulator_substate(
    context: &MappingContext,
    substate: &ComponentRoyaltyAccumulatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentRoyaltyAccumulatorSubstate { royalty_vault } = substate;

    let vault_id = match royalty_vault {
        Some(own) => Some(Box::new(to_api_entity_reference(
            context,
            own.as_node_id(),
        )?)),
        None => None,
    };

    Ok(models::Substate::RoyaltyModuleFieldAccumulatorSubstate {
        vault_entity: vault_id,
    })
}

pub fn to_api_package_info_substate(
    context: &MappingContext,
    substate: &PackageInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageInfoSubstate { schema } = substate;

    Ok(models::Substate::PackageFieldInfoSubstate {
        package_schema: Box::new(to_api_package_schema(context, schema)?),
    })
}

pub fn to_api_package_schema(
    context: &MappingContext,
    package_schema: &IndexedPackageSchema,
) -> Result<models::PackageSchema, MappingError> {
    let IndexedPackageSchema { blueprints } = package_schema;
    Ok(models::PackageSchema {
        blueprint_definitions: blueprints
            .iter()
            .map(|(blueprint_name, blueprint_definition)| {
                Ok((
                    blueprint_name.to_owned(),
                    to_api_blueprint_definition(context, blueprint_definition)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_blueprint_definition(
    context: &MappingContext,
    blueprint_definition: &BlueprintDefinition,
) -> Result<models::BlueprintDefinition, MappingError> {
    let BlueprintDefinition { schema, template } = blueprint_definition;
    Ok(models::BlueprintDefinition {
        schema: Box::new(to_api_blueprint_schema(context, schema)?),
        template: Box::new(to_api_blueprint_template(context, template)?),
    })
}

pub fn to_api_blueprint_template(
    context: &MappingContext,
    blueprint_template: &BlueprintTemplate,
) -> Result<models::BlueprintTemplate, MappingError> {
    let BlueprintTemplate {
        method_auth_template,
        outer_method_auth_template,
    } = blueprint_template;
    Ok(models::BlueprintTemplate {
        method_auth_template: to_api_method_auth_template(context, method_auth_template)?,
        outer_method_auth_template: to_api_method_auth_template(
            context,
            outer_method_auth_template,
        )?,
    })
}

pub fn to_api_method_auth_template(
    context: &MappingContext,
    method_auth_template: &BTreeMap<SchemaMethodKey, SchemaMethodPermission>,
) -> Result<Vec<models::MethodAuthTemplateEntry>, MappingError> {
    method_auth_template
        .iter()
        .map(|(key, permission)| to_api_method_auth_template_entry(context, key, permission))
        .collect::<Result<Vec<_>, _>>()
}

pub fn to_api_method_auth_template_entry(
    _context: &MappingContext,
    key: &SchemaMethodKey,
    permission: &SchemaMethodPermission,
) -> Result<models::MethodAuthTemplateEntry, MappingError> {
    let SchemaMethodKey { module_id, ident } = key;
    Ok(models::MethodAuthTemplateEntry {
        key: Box::new(models::MethodKey {
            object_module_id: to_api_object_module_id(&resolve_object_module_id(*module_id)?),
            ident: ident.to_owned(),
        }),
        permission: Some(match permission {
            SchemaMethodPermission::Public => models::MethodPermission::PublicMethodPermission {},
            SchemaMethodPermission::Protected(allowed_role_keys) => {
                models::MethodPermission::ProtectedMethodPermission {
                    allowed_role_keys: allowed_role_keys.clone(),
                }
            }
        }),
    })
}

pub fn to_api_blueprint_schema(
    context: &MappingContext,
    blueprint_schema: &IndexedBlueprintSchema,
) -> Result<models::BlueprintSchema, MappingError> {
    let IndexedBlueprintSchema {
        outer_blueprint,
        schema,
        functions,
        virtual_lazy_load_functions,
        event_schema,
        fields,
        collections,
        dependencies,
        features,
    } = blueprint_schema;
    Ok(models::BlueprintSchema {
        outer_blueprint: outer_blueprint.clone(),
        schema: Box::new(to_api_scrypto_schema(context, schema)?),
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
        field_partition: fields
            .as_ref()
            .map(|(partition_offset, field_types)| {
                Ok(Box::new(to_api_blueprint_schema_fields_partition(
                    context,
                    *partition_offset,
                    field_types,
                )?))
            })
            .transpose()?,
        collection_partitions: collections
            .iter()
            .map(|(partition_offset, blueprint_collection_schema)| {
                to_api_blueprint_schema_collection_partition(
                    context,
                    *partition_offset,
                    blueprint_collection_schema,
                )
            })
            .collect::<Result<_, _>>()?,
        dependencies: dependencies
            .iter()
            .map(|dependency| to_api_global_address(context, dependency))
            .collect::<Result<Vec<_>, _>>()?,
        features: features.iter().cloned().collect(),
    })
}

pub fn to_api_local_type_index(
    context: &MappingContext,
    local_type_index: &LocalTypeIndex,
) -> Result<models::LocalTypeIndex, MappingError> {
    Ok(match local_type_index {
        LocalTypeIndex::WellKnown(index) => models::LocalTypeIndex {
            kind: models::local_type_index::Kind::WellKnown,
            index: to_api_u8_as_i32(*index),
            as_sbor: Box::new(to_api_sbor_data_from_encodable(context, local_type_index)?),
        },
        LocalTypeIndex::SchemaLocalIndex(index) => models::LocalTypeIndex {
            kind: models::local_type_index::Kind::SchemaLocal,
            index: to_api_u16_as_i32((*index).try_into().map_err(|_| {
                MappingError::IntegerError {
                    message: "Type index too large".to_string(),
                }
            })?),
            as_sbor: Box::new(to_api_sbor_data_from_encodable(context, local_type_index)?),
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
        export,
    } = function_schema;
    Ok(models::FunctionSchema {
        receiver_info: receiver
            .as_ref()
            .map(|receiver_info| Box::new(to_api_receiver_info(receiver_info))),
        input: Box::new(to_api_local_type_index(context, input)?),
        output: Box::new(to_api_local_type_index(context, output)?),
        export: export.to_string(),
    })
}

pub fn to_api_receiver_info(receiver_info: &ReceiverInfo) -> models::ReceiverInfo {
    let ReceiverInfo {
        receiver,
        ref_types,
    } = receiver_info;
    models::ReceiverInfo {
        receiver: match receiver {
            Receiver::SelfRef => models::receiver_info::Receiver::SelfRef,
            Receiver::SelfRefMut => models::receiver_info::Receiver::SelfRefMut,
        },
        reference_type: Box::new(models::ReferenceType {
            raw_bits: to_api_u32_as_i64(ref_types.bits()),
            normal: ref_types.intersects(RefTypes::NORMAL),
            direct_access: ref_types.intersects(RefTypes::DIRECT_ACCESS),
        }),
    }
}

pub fn to_api_virtual_lazy_load_schema(
    virtual_lazy_load_schema: &VirtualLazyLoadSchema,
) -> Result<models::VirtualLazyLoadSchema, MappingError> {
    Ok(models::VirtualLazyLoadSchema {
        export_name: virtual_lazy_load_schema.export_name.to_string(),
    })
}

pub fn to_api_blueprint_schema_fields_partition(
    context: &MappingContext,
    partition_offset: PartitionOffset,
    fields: &[FieldSchema],
) -> Result<models::BlueprintSchemaFieldPartition, MappingError> {
    Ok(models::BlueprintSchemaFieldPartition {
        partition_offset: to_api_u8_as_i32(partition_offset.0),
        fields: fields
            .iter()
            .map(|field| to_api_blueprint_field_schema(context, field))
            .collect::<Result<_, MappingError>>()?,
    })
}

pub fn to_api_blueprint_field_schema(
    context: &MappingContext,
    field: &FieldSchema,
) -> Result<models::BlueprintFieldSchema, MappingError> {
    let (type_index, feature) = match field {
        FieldSchema::Normal { value } => (value, None),
        FieldSchema::Conditional { value, feature } => (value, Some(feature)),
    };
    Ok(models::BlueprintFieldSchema {
        _type: Box::new(to_api_local_type_index(context, type_index)?),
        feature: feature.cloned(),
    })
}

pub fn to_api_blueprint_schema_collection_partition(
    context: &MappingContext,
    partition_offset: PartitionOffset,
    collection_schema: &BlueprintCollectionSchema,
) -> Result<models::BlueprintSchemaCollectionPartition, MappingError> {
    Ok(models::BlueprintSchemaCollectionPartition {
        partition_offset: to_api_u8_as_i32(partition_offset.0),
        collection_schema: Some(to_api_blueprint_collection_schema(
            context,
            collection_schema,
        )?),
    })
}

pub fn to_api_blueprint_collection_schema(
    context: &MappingContext,
    collection_schema: &BlueprintCollectionSchema,
) -> Result<models::BlueprintCollectionSchema, MappingError> {
    Ok(match collection_schema {
        BlueprintCollectionSchema::KeyValueStore(BlueprintKeyValueStoreSchema {
            key,
            value,
            can_own,
        }) => models::BlueprintCollectionSchema::KeyValueBlueprintCollectionSchema {
            key_type_reference: Box::new(to_api_blueprint_schema_type_reference(context, key)?),
            value_type_reference: Box::new(to_api_blueprint_schema_type_reference(context, value)?),
            can_own: *can_own,
        },
        BlueprintCollectionSchema::Index(BlueprintIndexSchema {}) => {
            models::BlueprintCollectionSchema::IndexBlueprintCollectionSchema {}
        }
        BlueprintCollectionSchema::SortedIndex(BlueprintSortedIndexSchema {}) => {
            models::BlueprintCollectionSchema::SortedIndexBlueprintCollectionSchema {}
        }
    })
}

pub fn to_api_blueprint_schema_type_reference(
    context: &MappingContext,
    type_ref: &TypeRef,
) -> Result<models::BlueprintTypeReference, MappingError> {
    Ok(match type_ref {
        TypeRef::Blueprint(local_type_index) => {
            models::BlueprintTypeReference::BlueprintSchemaBlueprintTypeReference {
                local_type_index: Box::new(to_api_local_type_index(context, local_type_index)?),
            }
        }
        TypeRef::Instance(instance_provided_type_index) => {
            models::BlueprintTypeReference::InstanceSchemaBlueprintTypeReference {
                instance_provided_type_index: to_api_u8_as_i32(*instance_provided_type_index),
            }
        }
    })
}

pub fn to_api_scrypto_schema(
    context: &MappingContext,
    schema: &ScryptoSchema,
) -> Result<models::ScryptoSchema, MappingError> {
    Ok(models::ScryptoSchema {
        sbor_data: Box::new(to_api_sbor_data_from_encodable(context, schema)?),
    })
}

pub fn to_api_package_code_substate(
    _context: &MappingContext,
    substate: &PackageCodeSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageCodeSubstate { code } = substate;

    Ok(models::Substate::PackageFieldCodeSubstate {
        code_hex: to_hex(code),
    })
}

pub fn to_api_package_code_type_substate(
    _context: &MappingContext,
    substate: &PackageCodeTypeSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::PackageFieldCodeTypeSubstate {
        code_type: match substate {
            PackageCodeTypeSubstate::Native => models::PackageCodeType::Native,
            PackageCodeTypeSubstate::Wasm => models::PackageCodeType::Wasm,
        },
    })
}

pub fn to_api_package_royalty_substate(
    context: &MappingContext,
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
                context,
                royalty_vault.as_node_id(),
            )?))
        })
        .transpose()?;
    Ok(models::Substate::PackageFieldRoyaltySubstate {
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
        sorted_key,
        key,
        is_registered,
        validator_fee_factor,
        validator_fee_change_request,
        stake_unit_resource,
        stake_xrd_vault_id,
        unstake_nft,
        pending_xrd_withdraw_vault_id,
        locked_owner_stake_unit_vault_id,
        pending_owner_stake_unit_unlock_vault_id,
        pending_owner_stake_unit_withdrawals,
        already_unlocked_owner_stake_unit_amount,
    } = substate;

    Ok(models::Substate::ValidatorFieldStateSubstate {
        sorted_key: sorted_key.as_ref().map(|key| {
            Box::new(to_api_substate_key(&SubstateKey::Sorted((
                key.0,
                key.1.clone(),
            ))))
        }),
        public_key: Box::new(to_api_ecdsa_secp256k1_public_key(key)),
        is_registered: *is_registered,
        validator_fee_factor: to_api_decimal(validator_fee_factor),
        validator_fee_change_request: validator_fee_change_request
            .as_ref()
            .map(|validator_fee_change_request| -> Result<_, _> {
                let ValidatorFeeChangeRequest {
                    epoch_effective,
                    new_fee_factor,
                } = validator_fee_change_request;
                Ok(Box::new(models::ValidatorFeeChangeRequest {
                    epoch_effective: to_api_epoch(context, *epoch_effective)?,
                    new_fee_factor: to_api_decimal(new_fee_factor),
                }))
            })
            .transpose()?,
        stake_unit_resource_address: to_api_resource_address(context, stake_unit_resource)?,
        stake_xrd_vault: Box::new(to_api_entity_reference(
            context,
            stake_xrd_vault_id.as_node_id(),
        )?),
        unstake_claim_token_resource_address: to_api_resource_address(context, unstake_nft)?,
        pending_xrd_withdraw_vault: Box::new(to_api_entity_reference(
            context,
            pending_xrd_withdraw_vault_id.as_node_id(),
        )?),
        locked_owner_stake_unit_vault: Box::new(to_api_entity_reference(
            context,
            locked_owner_stake_unit_vault_id.as_node_id(),
        )?),
        pending_owner_stake_unit_unlock_vault: Box::new(to_api_entity_reference(
            context,
            pending_owner_stake_unit_unlock_vault_id.as_node_id(),
        )?),
        pending_owner_stake_unit_withdrawals: pending_owner_stake_unit_withdrawals
            .iter()
            .map(|(epoch, amount)| -> Result<_, _> {
                Ok(models::PendingOwnerStakeWithdrawal {
                    epoch_unlocked: to_api_epoch(context, *epoch)?,
                    stake_unit_amount: to_api_decimal(amount),
                })
            })
            .collect::<Result<_, _>>()?,
        already_unlocked_owner_stake_unit_amount: to_api_decimal(
            already_unlocked_owner_stake_unit_amount,
        ),
    })
}

pub fn to_api_consensus_manager_state_substate(
    context: &MappingContext,
    substate: &ConsensusManagerSubstate,
) -> Result<models::Substate, MappingError> {
    let ConsensusManagerSubstate {
        epoch,
        round,
        epoch_start_milli,
    } = substate;
    Ok(models::Substate::ConsensusManagerFieldStateSubstate {
        epoch: to_api_epoch(context, *epoch)?,
        round: to_api_round(*round)?,
        epoch_start: Box::new(to_api_instant_from_safe_timestamp(*epoch_start_milli)?),
    })
}

pub fn to_api_consensus_manager_config_substate(
    substate: &ConsensusManagerConfigSubstate,
) -> Result<models::Substate, MappingError> {
    let ConsensusManagerConfigSubstate {
        config:
            ConsensusManagerConfig {
                max_validators,
                epoch_change_condition,
                num_unstake_epochs,
                total_emission_xrd_per_epoch,
                min_validator_reliability,
                num_owner_stake_units_unlock_epochs,
                num_fee_increase_delay_epochs,
            },
    } = substate;
    Ok(models::Substate::ConsensusManagerFieldConfigSubstate {
        max_validators: to_api_ten_trillion_capped_u64(
            u64::from(*max_validators),
            "max_validators",
        )?,
        epoch_change_condition: Box::new(to_api_epoch_change_condition(epoch_change_condition)?),
        num_unstake_epochs: to_api_ten_trillion_capped_u64(
            *num_unstake_epochs,
            "num_unstake_epochs",
        )?,
        total_emission_xrd_per_epoch: to_api_decimal(total_emission_xrd_per_epoch),
        min_validator_reliability: to_api_decimal(min_validator_reliability),
        num_owner_stake_units_unlock_epochs: to_api_ten_trillion_capped_u64(
            *num_owner_stake_units_unlock_epochs,
            "num_owner_stake_units_unlock_epochs",
        )?,
        num_fee_increase_delay_epochs: to_api_ten_trillion_capped_u64(
            *num_fee_increase_delay_epochs,
            "num_fee_increase_delay_epochs",
        )?,
    })
}

pub fn to_api_epoch_change_condition(
    epoch_change_condition: &EpochChangeCondition,
) -> Result<models::EpochChangeCondition, MappingError> {
    let EpochChangeCondition {
        min_round_count,
        max_round_count,
        target_duration_millis,
    } = epoch_change_condition;
    Ok(models::EpochChangeCondition {
        min_round_count: to_api_ten_trillion_capped_u64(*min_round_count, "min_round_count")?,
        max_round_count: to_api_ten_trillion_capped_u64(*max_round_count, "max_round_count")?,
        target_duration_millis: to_api_ten_trillion_capped_u64(
            *target_duration_millis,
            "target_duration_millis",
        )?,
    })
}

pub fn to_api_current_time_substate(
    substate: &ProposerMilliTimestampSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ProposerMilliTimestampSubstate { epoch_milli } = substate;

    Ok(models::Substate::ConsensusManagerCurrentTimeSubstate {
        proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(*epoch_milli)?),
    })
}

pub fn to_api_current_time_rounded_to_minutes_substate(
    substate: &ProposerMinuteTimestampSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ProposerMinuteTimestampSubstate { epoch_minute } = substate;

    Ok(
        models::Substate::ConsensusManagerCurrentTimeRoundedToMinutesSubstate {
            proposer_timestamp_rounded_down_to_minute: Box::new(
                to_api_instant_from_safe_timestamp(i64::from(*epoch_minute) * 60 * 1000)?,
            ),
        },
    )
}

pub fn to_api_fungible_vault_balance_substate(
    _context: &MappingContext,
    balance: &FungibleVaultBalanceSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::FungibleVaultFieldBalanceSubstate {
        amount: to_api_decimal(&balance.amount()),
    })
}

pub fn to_api_non_fungible_vault_balance_substate(
    _context: &MappingContext,
    substate: &NonFungibleVaultBalanceSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::NonFungibleVaultFieldBalanceSubstate {
        amount: to_api_decimal(&substate.amount),
    })
}

pub fn to_api_non_fungible_vault_contents_entry_substate(
    _context: &MappingContext,
    non_fungible_id: &NonFungibleLocalId,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::NonFungibleVaultContentsIndexEntrySubstate {
            non_fungible_local_id: Box::new(to_api_non_fungible_local_id(non_fungible_id)),
        },
    )
}

pub fn to_api_fungible_resource_amount(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    amount: &Decimal,
) -> Result<models::ResourceAmount, MappingError> {
    Ok(models::ResourceAmount::FungibleResourceAmount {
        resource_address: to_api_resource_address(context, resource_address)?,
        amount: to_api_decimal(amount),
    })
}

pub fn to_api_non_fungible_resource_amount(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    amount: &Decimal,
    ids: &BTreeSet<NonFungibleLocalId>,
) -> Result<models::ResourceAmount, MappingError> {
    let non_fungible_ids = ids.iter().map(to_api_non_fungible_local_id).collect();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: to_api_resource_address(context, resource_address)?,
        amount: to_api_decimal(amount),
        non_fungible_ids,
    })
}

pub fn to_api_access_controller_substate(
    context: &MappingContext,
    substate: &AccessControllerSubstate,
) -> Result<models::Substate, MappingError> {
    let data = scrypto_encode(substate).unwrap();
    let substate = models::Substate::AccessControllerFieldStateSubstate {
        data_struct: Box::new(to_api_data_struct_from_bytes(context, &data)?),
    };

    Ok(substate)
}

pub fn to_api_fungible_resource_manager_divisibility_substate(
    substate: &FungibleResourceManagerDivisibilitySubstate,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::FungibleResourceManagerFieldDivisibilitySubstate {
            divisibility: to_api_u8_as_i32(*substate),
        },
    )
}

pub fn to_api_fungible_resource_manager_total_supply_substate(
    substate: &FungibleResourceManagerTotalSupplySubstate,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::FungibleResourceManagerFieldTotalSupplySubstate {
            total_supply: to_api_decimal(substate),
        },
    )
}

pub fn to_api_non_fungible_resource_manager_id_type_substate(
    substate: &NonFungibleResourceManagerIdTypeSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::NonFungibleResourceManagerFieldIdTypeSubstate {
            non_fungible_id_type: to_api_non_fungible_id_type(substate),
        },
    )
}

pub fn to_api_non_fungible_resource_manager_total_supply_substate(
    substate: &NonFungibleResourceManagerTotalSupplySubstate,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::NonFungibleResourceManagerFieldTotalSupplySubstate {
            total_supply: to_api_decimal(substate),
        },
    )
}

pub fn to_api_non_fungible_resource_manager_mutable_fields_substate(
    _context: &MappingContext,
    substate: &NonFungibleResourceManagerMutableFieldsSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(
        models::Substate::NonFungibleResourceManagerFieldMutableFieldsSubstate {
            mutable_fields: substate.mutable_fields.iter().cloned().collect(),
        },
    )
}

pub fn to_api_non_fungible_resource_manager_data_substate(
    context: &MappingContext,
    substate: &KeyValueEntrySubstate<ScryptoRawValue<'_>>,
) -> Result<models::Substate, MappingError> {
    let KeyValueEntrySubstate { value, mutability } = substate;
    let (is_deleted, data_struct) = match value {
        Some(value) => (
            false,
            Some(Box::new(to_api_data_struct_from_scrypto_raw_value(
                context, value,
            )?)),
        ),
        None => (true, None),
    };
    let is_mutable = match mutability {
        SubstateMutability::Mutable => true,
        SubstateMutability::Immutable => false,
    };
    Ok(
        models::Substate::NonFungibleResourceManagerDataEntrySubstate {
            is_deleted,
            data_struct,
            is_mutable,
        },
    )
}
