use super::super::*;
use crate::core_api::models;

use radix_engine::types::*;

use radix_engine_queries::typed_substate_layout::*;

use super::*;
use super::super::MappingError;

pub fn to_api_substate(
    context: &MappingContext,
    typed_substate_key: &TypedSubstateKey,
    typed_substate_value: &TypedSubstateValue,
) -> Result<models::Substate, MappingError> {
    Ok(match typed_substate_value {
        TypedSubstateValue::TypeInfoModule(TypedTypeInfoModuleSubstateValue::TypeInfo(
            type_info_substate,
        )) => to_api_type_info_substate(context, type_info_substate)?,
        TypedSubstateValue::AccessRulesModule(TypedAccessRulesModuleSubstateValue::OwnerRole(
            substate,
        )) => to_api_owner_role_substate(context, substate)?,
        TypedSubstateValue::AccessRulesModule(TypedAccessRulesModuleSubstateValue::Rule(
            substate,
        )) => to_api_access_rule_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::RoyaltyModule(TypedRoyaltyModuleSubstateValue::ComponentRoyalty(
            component_royalty_accumulator_substate,
        )) => to_api_component_royalty_substate(context, component_royalty_accumulator_substate)?,
        TypedSubstateValue::RoyaltyModule(
            TypedRoyaltyModuleSubstateValue::ComponentMethodRoyalty(substate),
        ) => to_api_component_method_royalty_substate(context, typed_substate_key, substate)?,
        TypedSubstateValue::MetadataModule(TypedMetadataModuleSubstateValue::MetadataEntry(
            metadata_value_substate,
        )) => to_api_metadata_value_substate(context, typed_substate_key, metadata_value_substate)?,
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
        )) => to_api_non_fungible_resource_manager_data_substate(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleVault(
            TypedFungibleVaultFieldValue::Balance(fungible_vault_balance_substate),
        )) => to_api_fungible_vault_balance_substate(context, fungible_vault_balance_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleVault(
            TypedFungibleVaultFieldValue::VaultFrozenFlag(substate),
        )) => to_api_fungible_vault_frozen_status_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVaultField(
            TypedNonFungibleVaultFieldValue::Balance(substate),
        )) => to_api_non_fungible_vault_balance_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVaultField(
            TypedNonFungibleVaultFieldValue::VaultFrozenFlag(substate),
        )) => to_api_non_fungible_vault_frozen_status_substate(context, substate)?,
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
        ) => to_api_registered_validator_set_substate(context, typed_substate_key, entry)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Validator(
            TypedValidatorFieldValue::Validator(validator_substate),
        )) => to_api_validator_substate(context, validator_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Validator(
            TypedValidatorFieldValue::ProtocolUpdateReadinessSignal(substate),
        )) => to_api_validator_protocol_update_readiness_signal_substate(context, substate)?,
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
            GenericScryptoComponentFieldValue::State(substate),
        )) => to_api_generic_scrypto_component_state_substate(
            context,
            substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::GenericKeyValueStore(
            substate,
        )) => to_api_generic_key_value_store_substate(context, typed_substate_key, substate)?,
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
        )) => to_api_package_blueprint_definition_entry(context, typed_substate_key, substate)?,
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
            to_api_package_blueprint_royalty_entry(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::TransactionTracker(
            TypedTransactionTrackerFieldValue::TransactionTracker(substate),
        )) => to_api_transaction_tracker_substate(context, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::TransactionTrackerCollectionEntry(substate),
        ) => to_api_transaction_tracker_collection_entry(context, typed_substate_key, substate)?,
    })
}
