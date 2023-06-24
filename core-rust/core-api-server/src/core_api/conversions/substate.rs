use radix_engine::blueprints::access_controller::AccessControllerSubstate;
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::blueprints::transaction_tracker::TransactionTrackerSubstate;

use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::system::KeyValueEntrySubstate;
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

trait WrapperMethods {
    type Content;
    fn get_definitely_present_value(&self) -> Result<&Self::Content, MappingError>;
}

impl<Content> WrapperMethods for KeyValueEntrySubstate<Content> {
    type Content = Content;

    fn get_definitely_present_value(&self) -> Result<&Self::Content, MappingError> {
        match self.value.as_ref() {
            Some(value) => Ok(value),
            None => Err(MappingError::KeyValueStoreEntryUnexpectedlyAbsent),
        }
    }
}

pub fn to_api_substate(
    context: &MappingContext,
    substate_key: &SubstateKey,
    typed_substate_key: &TypedSubstateKey,
    typed_substate_value: &TypedSubstateValue,
) -> Result<models::Substate, MappingError> {
    Ok(match typed_substate_value {
        TypedSubstateValue::TypeInfoModule(TypedTypeInfoModuleSubstateValue::TypeInfo(
            type_info_substate,
        )) => to_api_type_info_substate(context, type_info_substate)?,
        TypedSubstateValue::AccessRulesModule(TypedAccessRulesModuleSubstateValue::OwnerRole(substate)) => {
            to_api_owner_role_substate(context, substate)?
        }
        TypedSubstateValue::AccessRulesModule(TypedAccessRulesModuleSubstateValue::Rule(substate)) => {
            to_api_access_rule_entry(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::AccessRulesModule(TypedAccessRulesModuleSubstateValue::Mutability(substate)) => {
            to_api_mutability_entry(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::RoyaltyModule(
            TypedRoyaltyModuleSubstateValue::ComponentRoyaltyAccumulator(
                component_royalty_accumulator_substate,
            ),
        ) => to_api_component_royalty_accumulator_substate(
            context,
            component_royalty_accumulator_substate,
        )?,

        TypedSubstateValue::RoyaltyModule(
            TypedRoyaltyModuleSubstateValue::ComponentRoyaltyConfig(
                component_royalty_config_substate,
            ),
        ) => to_api_component_royalty_config_substate(
            context,
            typed_substate_key,
            component_royalty_config_substate,
        )?,
        TypedSubstateValue::MetadataModule(
            TypedMetadataModuleSubstateValue::MetadataEntry(
                metadata_value_substate
            ),
        ) => {
            to_api_metadata_value_substate(context, substate_key, metadata_value_substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::Code(_),
        )) => panic!("Unused - to be removed in Scrypto"),
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            TypedPackageFieldValue::Royalty(package_royalty_accumulator_substate),
        )) => to_api_package_royalty_accumulator_substate(
            context,
            package_royalty_accumulator_substate,
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
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManagerField(
            TypedConsensusManagerFieldValue::ValidatorRewards(substate),
        )) => to_api_validator_rewards_substate(context, substate)?,
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
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::PackageBlueprint(
            substate,
        )) => to_api_package_blueprint_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::PackageBlueprintDependencies(substate),
        ) => to_api_package_blueprint_dependencies_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::PackageSchema(substate)) => {
            to_api_package_schema_entry(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::PackageCode(substate)) => {
            to_api_package_code_entry(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::PackageAuthTemplate(
            substate,
        )) => to_api_package_auth_template_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::PackageRoyalty(substate)) => {
            to_api_package_royalty_entry(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::TransactionTracker(
            TypedTransactionTrackerFieldValue::TransactionTracker(substate),
        )) => to_api_transaction_tracker_substate(context, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::TransactionTrackerCollectionEntry(substate),
        ) => to_api_transaction_tracker_collection_entry(context, typed_substate_key, substate)?,
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
    Ok(models::Substate::OneResourcePoolFieldStateSubstate {
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
    Ok(models::Substate::TwoResourcePoolFieldStateSubstate {
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
    Ok(models::Substate::MultiResourcePoolFieldStateSubstate {
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

pub fn to_api_transaction_tracker_substate(
    context: &MappingContext,
    substate: &TransactionTrackerSubstate,
) -> Result<models::Substate, MappingError> {
    let TransactionTrackerSubstate {
        start_epoch,
        start_partition,
        partition_range_start_inclusive,
        partition_range_end_inclusive,
        epochs_per_partition,
    } = substate;
    Ok(models::Substate::TransactionTrackerFieldStateSubstate {
        start_epoch: to_api_epoch(context, Epoch::of(*start_epoch))?,
        start_partition: to_api_u8_as_i32(*start_partition),
        partition_range_start_inclusive: to_api_u8_as_i32(*partition_range_start_inclusive),
        partition_range_end_inclusive: to_api_u8_as_i32(*partition_range_end_inclusive),
        epochs_per_partition: to_api_epoch(context, Epoch::of(*epochs_per_partition))?,
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
    substate: &KeyValueEntrySubstate<AccountResourceDepositRuleEntry>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountResourceDepositRuleIndexKey(resource_address)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Account Deposit Rule Key".to_string() });
    };
    Ok(models::Substate::AccountDepositRuleIndexEntrySubstate {
        resource_address: to_api_resource_address(context, resource_address)?,
        deposit_rule: substate.value.flatten().map(|rule| match rule {
            ResourceDepositRule::Neither => models::DepositRule::Neither,
            ResourceDepositRule::Allowed => models::DepositRule::Allowed,
            ResourceDepositRule::Disallowed => models::DepositRule::Disallowed,
        }),
        is_locked: !substate.is_mutable()
    })
}

pub fn to_api_access_rule_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<AccessRule>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::AccessRulesModule(TypedAccessRulesSubstateKey::Rule(ModuleRoleKey{ module, key })) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Module Role Key".to_string() });
    };
    Ok(models::Substate::AccessRulesModuleRuleEntrySubstate {
        object_module_id: to_api_object_module_id(module),
        role_key: key.key.to_string(),
        access_rule: substate
            .value
            .as_ref()
            .map(|access_rule| -> Result<_, MappingError> {
                Ok(Box::new(to_api_access_rule(context, access_rule)?))
            })
            .transpose()?,
    })
}

pub fn to_api_mutability_entry(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<RoleList>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::AccessRulesModule(TypedAccessRulesSubstateKey::Mutability(ModuleRoleKey{ module, key })) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Mutability Key".to_string() });
    };
    Ok(models::Substate::AccessRulesModuleMutabilityEntrySubstate {
        object_module_id: to_api_object_module_id(module),
        role_key: key.key.to_string(),
        mutable_role_keys: substate.value.as_ref().map(|role_list| {
            role_list
                .list
                .iter()
                .map(|key| key.key.to_string())
                .collect::<Vec<_>>()
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
    let (is_deleted, data_struct) = match &substate.value {
        Some(value) => (
            false,
            Some(Box::new(to_api_data_struct_from_scrypto_raw_value(
                context, value,
            )?)),
        ),
        None => (true, None),
    };
    Ok(models::Substate::GenericKeyValueStoreEntrySubstate {
        is_deleted,
        data_struct,
        is_locked: !substate.is_mutable(),
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

pub fn to_api_validator_rewards_substate(
    context: &MappingContext,
    substate: &ValidatorRewardsSubstate,
) -> Result<models::Substate, MappingError> {
    let ValidatorRewardsSubstate {
        proposer_rewards,
        rewards_vault,
    } = substate;
    Ok(
        models::Substate::ConsensusManagerFieldValidatorRewardsSubstate {
            proposer_rewards: proposer_rewards
                .iter()
                .map(|(validator_index, xrd_amount)| {
                    to_api_proposer_reward(context, validator_index, xrd_amount)
                })
                .collect::<Result<Vec<_>, MappingError>>()?,
            rewards_vault: Box::new(to_api_entity_reference(
                context,
                rewards_vault.0.as_node_id(),
            )?),
        },
    )
}

pub fn to_api_proposer_reward(
    _context: &MappingContext,
    validator_index: &ValidatorIndex,
    xrd_amount: &Decimal,
) -> Result<models::ProposerReward, MappingError> {
    Ok(models::ProposerReward {
        validator_index: Box::new(to_api_active_validator_index(*validator_index)),
        xrd_amount: to_api_decimal(xrd_amount),
    })
}

pub fn to_api_metadata_value_substate(
    context: &MappingContext,
    substate_key: &SubstateKey,
    substate: &MetadataEntrySubstate,
) -> Result<models::Substate, MappingError> {
    let SubstateKey::Map(key_bytes) = substate_key else {
        return Err(MappingError::InvalidMetadataKey { message: "Was not a map key".to_string() });
    };
    let field_name: String =
        scrypto_decode(key_bytes).map_err(|_| MappingError::InvalidMetadataKey {
            message: "Was not a string".to_string(),
        })?;
    let (is_deleted, data_struct) = match &substate.value {
        Some(entry) => (
            false,
            Some(Box::new(to_api_data_struct_from_bytes(
                context,
                &scrypto_encode(entry).unwrap(),
            )?)),
        ),
        None => (true, None),
    };
    Ok(models::Substate::MetadataModuleEntrySubstate {
        field_name,
        is_deleted,
        data_struct,
        is_locked: !substate.is_mutable(),
    })
}

pub fn to_api_owner_role_substate(
    context: &MappingContext,
    owner_role: &OwnerRole,
) -> Result<models::Substate, MappingError> {
    Ok(models::Substate::AccessRulesModuleFieldOwnerRoleSubstate {
        owner_role: Box::new(match owner_role {
            OwnerRole::None => models::OwnerRole::NoneOwnerRole {},
            OwnerRole::Fixed(access_rule) => models::OwnerRole::FixedOwnerRole {
                access_rule: Box::new(to_api_access_rule(context, access_rule)?),
            },
            OwnerRole::Updateable(access_rule) => models::OwnerRole::UpdateableOwnerRole {
                access_rule: Box::new(to_api_access_rule(context, access_rule)?),
            },
        }),
    })
}

pub fn to_api_non_fungible_id_type(id_type: &NonFungibleIdType) -> models::NonFungibleIdType {
    match id_type {
        NonFungibleIdType::String => models::NonFungibleIdType::String,
        NonFungibleIdType::Integer => models::NonFungibleIdType::Integer,
        NonFungibleIdType::Bytes => models::NonFungibleIdType::Bytes,
        NonFungibleIdType::RUID => models::NonFungibleIdType::RUID,
    }
}

pub fn to_api_type_info_substate(
    context: &MappingContext,
    substate: &TypeInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let details = match substate {
        TypeInfoSubstate::Object(ObjectInfo {
            blueprint_id:
                BlueprintId {
                    package_address,
                    blueprint_name,
                },
            version,
            global,
            outer_object,
            instance_schema,
            features,
        }) => models::TypeInfoDetails::ObjectTypeInfoDetails {
            package_address: to_api_package_address(context, package_address)?,
            blueprint_name: blueprint_name.to_string(),
            blueprint_version: to_api_blueprint_version(context, version)?,
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

pub fn to_api_royalty_amount(royalty_amount: &RoyaltyAmount) -> Option<models::RoyaltyAmount> {
    match royalty_amount {
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
    Ok(models::Substate::RoyaltyModuleFieldAccumulatorSubstate {
        vault_entity: Box::new(to_api_entity_reference(
            context,
            royalty_vault.0.as_node_id(),
        )?),
    })
}

pub fn to_api_component_royalty_config_substate(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &ComponentRoyaltyConfigSubstate,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::RoyaltyModule(TypedRoyaltyModuleSubstateKey::RoyaltyConfigEntryKey(method_name)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "RoyaltyConfigEntryKey".to_string() });
    };
    Ok(models::Substate::RoyaltyModuleMethodConfigEntrySubstate {
        is_locked: !substate.is_mutable(),
        method_name: method_name.clone(),
        royalty_amount: substate.value.as_ref().and_then(to_api_royalty_amount).map(Box::new),
    })
}

pub fn to_api_package_blueprint_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<BlueprintDefinition>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageBlueprintKey(BlueprintVersionKey{ blueprint, version })) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Package Blueprint Key".to_string() });
    };
    Ok(models::Substate::PackageBlueprintEntrySubstate {
        name: blueprint.to_string(),
        version: to_api_blueprint_version(context, version)?,
        definition: substate
            .value
            .as_ref()
            .map(|definition| -> Result<_, MappingError> {
                Ok(Box::new(to_api_blueprint_definition(context, definition)?))
            })
            .transpose()?,
    })
}

pub fn to_api_package_blueprint_dependencies_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<BlueprintDependencies>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageBlueprintDependenciesKey(BlueprintVersionKey{ blueprint, version })) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Package Blueprint Key".to_string() });
    };
    Ok(
        models::Substate::PackageBlueprintDependenciesEntrySubstate {
            name: blueprint.to_string(),
            version: to_api_blueprint_version(context, version)?,
            dependencies: substate
                .value
                .as_ref()
                .map(|dependencies| -> Result<_, MappingError> {
                    Ok(Box::new(to_api_blueprint_dependencies(
                        context,
                        dependencies,
                    )?))
                })
                .transpose()?,
        },
    )
}

pub fn to_api_package_schema_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<ScryptoSchema>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageSchemaKey(hash)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Package Schema Key".to_string() });
    };
    Ok(models::Substate::PackageSchemaEntrySubstate {
        schema_hash: to_api_hash(hash),
        schema: substate
            .value
            .as_ref()
            .map(|schema| -> Result<_, MappingError> {
                Ok(Box::new(to_api_scrypto_schema(context, schema)?))
            })
            .transpose()?,
    })
}

pub fn to_api_package_royalty_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<RoyaltyConfig>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageRoyaltyKey(BlueprintVersionKey{ blueprint, version})) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Package Royalty Key".to_string() });
    };
    Ok(models::Substate::PackageRoyaltyEntrySubstate {
        name: blueprint.to_string(),
        version: to_api_blueprint_version(context, version)?,
        royalty_config: substate
            .value
            .as_ref()
            .map(|config| -> Result<_, MappingError> {
                Ok(Box::new(to_api_royalty_config(config)))
            })
            .transpose()?,
    })
}

pub fn to_api_package_auth_template_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<AuthConfig>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageAuthTemplateKey(BlueprintVersionKey{ blueprint, version})) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Package Code Key".to_string() });
    };
    Ok(models::Substate::PackageAuthTemplateEntrySubstate {
        name: blueprint.to_string(),
        version: to_api_blueprint_version(context, version)?,
        auth_config: substate
            .value
            .as_ref()
            .map(|config| -> Result<_, MappingError> {
                Ok(Box::new(to_api_auth_config(context, config)?))
            })
            .transpose()?,
    })
}

pub fn to_api_auth_config(
    context: &MappingContext,
    config: &AuthConfig,
) -> Result<models::AuthConfig, MappingError> {
    let AuthConfig {
        function_auth,
        method_auth,
    } = config;
    let MethodAuthTemplate::Static { auth, outer_auth } = method_auth;
    Ok(models::AuthConfig {
        function_auth: function_auth
            .iter()
            .map(|(identifier, access_rule)| {
                Ok((
                    identifier.to_string(),
                    to_api_access_rule(context, access_rule)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        method_auth: Box::new(to_api_static_method_auth_template(
            context, auth, outer_auth,
        )?),
    })
}

pub fn to_api_static_method_auth_template(
    context: &MappingContext,
    auth: &BTreeMap<MethodKey, MethodPermission>,
    outer_auth: &BTreeMap<MethodKey, MethodPermission>,
) -> Result<models::StaticMethodAuthTemplate, MappingError> {
    Ok(models::StaticMethodAuthTemplate {
        auth: auth
            .iter()
            .map(|(key, permission)| {
                Ok((
                    key.ident.to_string(),
                    to_api_method_permission(context, permission)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        outer_auth: outer_auth
            .iter()
            .map(|(key, permission)| {
                Ok((
                    key.ident.to_string(),
                    to_api_method_permission(context, permission)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_method_permission(
    _context: &MappingContext,
    permission: &MethodPermission,
) -> Result<models::MethodPermission, MappingError> {
    Ok(match permission {
        MethodPermission::Public => models::MethodPermission::PublicMethodPermission {},
        MethodPermission::Protected(role_list) => {
            models::MethodPermission::ProtectedMethodPermission {
                allowed_role_keys: role_list
                    .list
                    .iter()
                    .map(|key| key.key.to_string())
                    .collect::<Vec<_>>(),
            }
        }
    })
}

pub fn to_api_package_code_entry(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<PackageCodeSubstate>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageCodeKey(hash)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Package Code Key".to_string() });
    };

    // Use compiler to unpack to ensure we map all fields
    let PackageCodeSubstate { vm_type, code } = substate.get_definitely_present_value()?;

    Ok(models::Substate::PackageCodeEntrySubstate {
        code_hash: to_api_hash(hash),
        vm_type: match vm_type {
            VmType::Native => models::VmType::Native,
            VmType::ScryptoV1 => models::VmType::ScryptoV1,
        },
        code_hex: to_hex(code),
    })
}

pub fn to_api_transaction_tracker_collection_entry(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<TransactionStatus>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(intent_hash)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Transaction Tracker Collection Key".to_string() });
    };
    Ok(
        models::Substate::TransactionTrackerCollectionEntrySubstate {
            intent_hash: to_api_hash(intent_hash.as_hash()),
            status: substate.value.as_ref().map(|status| match status {
                TransactionStatus::CommittedSuccess => {
                    models::TransactionTrackerTransactionStatus::CommittedSuccess
                }
                TransactionStatus::CommittedFailure => {
                    models::TransactionTrackerTransactionStatus::CommittedFailure
                }
                TransactionStatus::Cancelled => {
                    models::TransactionTrackerTransactionStatus::Cancelled
                }
            }),
        },
    )
}

pub fn to_api_blueprint_definition(
    context: &MappingContext,
    blueprint_definition: &BlueprintDefinition,
) -> Result<models::BlueprintDefinition, MappingError> {
    let BlueprintDefinition {
        interface,
        function_exports,
        virtual_lazy_load_functions,
    } = blueprint_definition;
    Ok(models::BlueprintDefinition {
        interface: Box::new(to_api_blueprint_interface(context, interface)?),
        function_exports: function_exports
            .iter()
            .map(|(function_name, package_export)| {
                Ok((
                    function_name.to_string(),
                    to_api_package_export(context, package_export)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        virtual_lazy_load_functions: virtual_lazy_load_functions
            .iter()
            .map(|(function_id, package_export)| {
                Ok((
                    function_id.to_string(),
                    to_api_package_export(context, package_export)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_blueprint_dependencies(
    context: &MappingContext,
    dependencies: &BlueprintDependencies,
) -> Result<models::BlueprintDependencies, MappingError> {
    let BlueprintDependencies { dependencies } = dependencies;
    Ok(models::BlueprintDependencies {
        dependencies: dependencies
            .iter()
            .map(|address| to_api_global_address(context, address))
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_package_export(
    _context: &MappingContext,
    package_export: &PackageExport,
) -> Result<models::PackageExport, MappingError> {
    let PackageExport {
        code_hash,
        export_name,
    } = package_export;
    Ok(models::PackageExport {
        code_hash: to_api_hash(code_hash),
        export_name: export_name.to_string(),
    })
}

pub fn to_api_blueprint_interface(
    context: &MappingContext,
    blueprint_interface: &BlueprintInterface,
) -> Result<models::BlueprintInterface, MappingError> {
    let BlueprintInterface {
        outer_blueprint,
        generics,
        state,
        functions,
        features,
        events,
    } = blueprint_interface;
    Ok(models::BlueprintInterface {
        outer_blueprint: outer_blueprint.clone(),
        generic_type_parameters: generics
            .iter()
            .map(|generic| match generic {
                Generic::Any => models::GenericTypeParameter {
                    constraints: models::GenericTypeParameterContraints::Any,
                },
            })
            .collect::<Vec<_>>(),
        features: features.iter().cloned().collect(),
        state: Box::new(to_api_indexed_state_schema(context, state)?),
        functions: functions
            .iter()
            .map(|(function_name, function_schema)| {
                Ok((
                    function_name.to_string(),
                    to_api_function_schema(context, function_schema)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        events: events
            .iter()
            .map(|(event_name, type_pointer)| {
                Ok((
                    event_name.to_string(),
                    to_api_type_pointer(context, type_pointer)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_indexed_state_schema(
    context: &MappingContext,
    indexed_state_schema: &IndexedStateSchema,
) -> Result<models::IndexedStateSchema, MappingError> {
    let IndexedStateSchema {
        fields,
        collections,
        num_partitions,
    } = indexed_state_schema;
    Ok(models::IndexedStateSchema {
        fields: fields
            .as_ref()
            .map(|(partition_offset, fields)| {
                to_api_blueprint_schema_fields_partition(context, *partition_offset, fields)
            })
            .transpose()?
            .map(Box::new),
        collections: collections
            .iter()
            .map(|(partition_offset, collection_schema)| {
                to_api_blueprint_schema_collection_partition(
                    context,
                    *partition_offset,
                    collection_schema,
                )
            })
            .collect::<Result<_, _>>()?,
        num_partitions: to_api_u8_as_i32(*num_partitions),
    })
}

pub fn to_api_type_pointer(
    context: &MappingContext,
    type_pointer: &TypePointer,
) -> Result<models::TypePointer, MappingError> {
    Ok(match type_pointer {
        TypePointer::Package(hash, local_type_index) => models::TypePointer::PackageTypePointer {
            schema_hash: to_api_hash(hash),
            local_type_index: Box::new(to_api_local_type_index(context, local_type_index)?),
        },
        TypePointer::Instance(index) => models::TypePointer::InstanceTypePointer {
            index: to_api_u8_as_i32(*index),
        },
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
    } = function_schema;
    Ok(models::FunctionSchema {
        receiver_info: receiver
            .as_ref()
            .map(|receiver_info| Box::new(to_api_receiver_info(receiver_info))),
        input: Some(to_api_type_pointer(context, input)?),
        output: Some(to_api_type_pointer(context, output)?),
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

pub fn to_api_blueprint_schema_fields_partition(
    context: &MappingContext,
    partition_offset: PartitionOffset,
    fields: &[FieldSchema<TypePointer>],
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
    field_schema: &FieldSchema<TypePointer>,
) -> Result<models::FieldSchema, MappingError> {
    let FieldSchema { field, condition } = field_schema;
    Ok(models::FieldSchema {
        field_type_pointer: Some(to_api_type_pointer(context, field)?),
        if_feature: match condition {
            Condition::Always => None,
            Condition::IfFeature(feature) => Some(feature.to_owned()),
        },
    })
}

pub fn to_api_blueprint_schema_collection_partition(
    context: &MappingContext,
    partition_offset: PartitionOffset,
    collection_schema: &BlueprintCollectionSchema<TypePointer>,
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
    collection_schema: &BlueprintCollectionSchema<TypePointer>,
) -> Result<models::BlueprintCollectionSchema, MappingError> {
    Ok(match collection_schema {
        BlueprintCollectionSchema::KeyValueStore(BlueprintKeyValueStoreSchema {
            key,
            value,
            can_own,
        }) => models::BlueprintCollectionSchema::KeyValueBlueprintCollectionSchema {
            key_type_pointer: Box::new(to_api_type_pointer(context, key)?),
            value_type_pointer: Box::new(to_api_type_pointer(context, value)?),
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

pub fn to_api_scrypto_schema(
    context: &MappingContext,
    schema: &ScryptoSchema,
) -> Result<models::ScryptoSchema, MappingError> {
    Ok(models::ScryptoSchema {
        sbor_data: Box::new(to_api_sbor_data_from_encodable(context, schema)?),
    })
}

pub fn to_api_package_royalty_accumulator_substate(
    context: &MappingContext,
    substate: &PackageRoyaltyAccumulatorSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let PackageRoyaltyAccumulatorSubstate { royalty_vault } = substate;

    let vault_entity = royalty_vault
        .map(|royalty_vault| {
            Ok(Box::new(to_api_entity_reference(
                context,
                royalty_vault.as_node_id(),
            )?))
        })
        .transpose()?;
    Ok(models::Substate::PackageFieldRoyaltyAccumulatorSubstate { vault_entity })
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
        current_leader,
    } = substate;
    Ok(models::Substate::ConsensusManagerFieldStateSubstate {
        epoch: to_api_epoch(context, *epoch)?,
        round: to_api_round(*round)?,
        epoch_start: Box::new(to_api_instant_from_safe_timestamp(*epoch_start_milli)?),
        current_leader: current_leader
            .as_ref()
            .map(|validator_index| to_api_active_validator_index(*validator_index))
            .map(Box::new),
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

    Ok(models::Substate::ConsensusManagerFieldCurrentTimeSubstate {
        proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(*epoch_milli)?),
    })
}

pub fn to_api_current_time_rounded_to_minutes_substate(
    substate: &ProposerMinuteTimestampSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ProposerMinuteTimestampSubstate { epoch_minute } = substate;

    Ok(
        models::Substate::ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate {
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
    let (is_deleted, data_struct) = match &substate.value {
        Some(value) => (
            false,
            Some(Box::new(to_api_data_struct_from_scrypto_raw_value(
                context, value,
            )?)),
        ),
        None => (true, None),
    };
    Ok(
        models::Substate::NonFungibleResourceManagerDataEntrySubstate {
            is_deleted,
            data_struct,
            is_locked: !substate.is_mutable(),
        },
    )
}
