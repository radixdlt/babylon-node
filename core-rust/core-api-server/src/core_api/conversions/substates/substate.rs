use super::super::*;
use crate::core_api::models;
use radix_engine::blueprints::account::{
    AccountTypedFieldSubstateValue, AccountTypedSubstateValue,
};
use radix_engine::blueprints::pool::multi_resource_pool::{
    MultiResourcePoolTypedFieldSubstateValue, MultiResourcePoolTypedSubstateValue,
};
use radix_engine::blueprints::pool::one_resource_pool::{
    OneResourcePoolTypedFieldSubstateValue, OneResourcePoolTypedSubstateValue,
};
use radix_engine::blueprints::pool::two_resource_pool::{
    TwoResourcePoolTypedFieldSubstateValue, TwoResourcePoolTypedSubstateValue,
};

use radix_engine::types::*;

use radix_engine_queries::typed_substate_layout::*;

use super::super::MappingError;
use super::*;

pub fn to_api_substate(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    typed_substate_key: &TypedSubstateKey,
    typed_substate_value: &TypedSubstateValue,
) -> Result<models::Substate, MappingError> {
    Ok(match typed_substate_value {
        TypedSubstateValue::BootLoader(BootLoaderSubstateValue::Vm(vm_boot_substate)) => {
            to_api_vm_boot_substate(context, state_mapping_lookups, vm_boot_substate)?
        }
        TypedSubstateValue::TypeInfoModule(TypedTypeInfoModuleSubstateValue::TypeInfo(
            type_info_substate,
        )) => to_api_type_info_substate(context, state_mapping_lookups, type_info_substate)?,
        TypedSubstateValue::RoleAssignmentModule(
            TypedRoleAssignmentModuleSubstateValue::OwnerRole(substate),
        ) => to_api_owner_role_substate(context, substate)?,
        TypedSubstateValue::RoleAssignmentModule(TypedRoleAssignmentModuleSubstateValue::Rule(
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
            PackageTypedSubstateValue::Field(PackageTypedFieldSubstateValue::RoyaltyAccumulator(
                substate,
            )),
        )) => to_api_package_royalty_accumulator_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleResourceManager(
            FungibleResourceManagerTypedSubstateValue::Field(
                FungibleResourceManagerTypedFieldSubstateValue::Divisibility(substate),
            ),
        )) => to_api_fungible_resource_manager_divisibility_substate(substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleResourceManager(
            FungibleResourceManagerTypedSubstateValue::Field(
                FungibleResourceManagerTypedFieldSubstateValue::TotalSupply(substate),
            ),
        )) => to_api_fungible_resource_manager_total_supply_substate(substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateValue::Field(
                    NonFungibleResourceManagerTypedFieldSubstateValue::IdType(substate),
                ),
            ),
        ) => to_api_non_fungible_resource_manager_id_type_substate(substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateValue::Field(
                    NonFungibleResourceManagerTypedFieldSubstateValue::TotalSupply(substate),
                ),
            ),
        ) => to_api_non_fungible_resource_manager_total_supply_substate(substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateValue::Field(
                    NonFungibleResourceManagerTypedFieldSubstateValue::MutableFields(substate),
                ),
            ),
        ) => to_api_non_fungible_resource_manager_mutable_fields_substate(context, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateValue::DataKeyValue(substate),
            ),
        ) => to_api_non_fungible_resource_manager_data_substate(
            context,
            typed_substate_key,
            substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleVault(
            FungibleVaultTypedSubstateValue::Field(FungibleVaultTypedFieldSubstateValue::Balance(
                fungible_vault_balance_substate,
            )),
        )) => to_api_fungible_vault_balance_substate(context, fungible_vault_balance_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleVault(
            FungibleVaultTypedSubstateValue::Field(
                FungibleVaultTypedFieldSubstateValue::FreezeStatus(substate),
            ),
        )) => to_api_fungible_vault_frozen_status_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVault(
            NonFungibleVaultTypedSubstateValue::Field(
                NonFungibleVaultTypedFieldSubstateValue::Balance(substate),
            ),
        )) => to_api_non_fungible_vault_balance_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVault(
            NonFungibleVaultTypedSubstateValue::Field(
                NonFungibleVaultTypedFieldSubstateValue::FreezeStatus(substate),
            ),
        )) => to_api_non_fungible_vault_frozen_status_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVault(
            NonFungibleVaultTypedSubstateValue::NonFungibleIndex(substate),
        )) => to_api_non_fungible_vault_contents_entry_substate(
            context,
            typed_substate_key,
            substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::NonFungibleVault(
            NonFungibleVaultTypedSubstateValue::Field(
                NonFungibleVaultTypedFieldSubstateValue::LockedResource(_),
            ),
        )) => Err(MappingError::UnexpectedPersistedData {
            message: "LockedNonFungible".to_owned(),
        })?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::FungibleVault(
            FungibleVaultTypedSubstateValue::Field(
                FungibleVaultTypedFieldSubstateValue::LockedBalance(_),
            ),
        )) => Err(MappingError::UnexpectedPersistedData {
            message: "LockedFungible".to_owned(),
        })?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::State(substate),
            ),
        )) => to_api_consensus_manager_state_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::Configuration(substate),
            ),
        )) => to_api_consensus_manager_config_substate(substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::CurrentValidatorSet(substate),
            ),
        )) => to_api_current_validator_set_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::CurrentProposalStatistic(substate),
            ),
        )) => to_api_current_proposal_statistic_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::ProposerMinuteTimestamp(substate),
            ),
        )) => to_api_current_time_rounded_to_minutes_substate(substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::ProposerMilliTimestamp(substate),
            ),
        )) => to_api_current_time_substate(substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::Field(
                ConsensusManagerTypedFieldSubstateValue::ValidatorRewards(substate),
            ),
        )) => to_api_validator_rewards_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::ConsensusManager(
            ConsensusManagerTypedSubstateValue::RegisteredValidatorByStakeSortedIndex(substate),
        )) => to_api_registered_validators_by_stake_index_entry_substate(
            context,
            typed_substate_key,
            substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Validator(
            ValidatorTypedSubstateValue::Field(ValidatorTypedFieldSubstateValue::State(
                validator_substate,
            )),
        )) => to_api_validator_substate(context, validator_substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Validator(
            ValidatorTypedSubstateValue::Field(
                ValidatorTypedFieldSubstateValue::ProtocolUpdateReadinessSignal(substate),
            ),
        )) => to_api_validator_protocol_update_readiness_signal_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Account(
            AccountTypedSubstateValue::Field(AccountTypedFieldSubstateValue::DepositRule(substate)),
        )) => to_api_account_state_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Account(
            AccountTypedSubstateValue::ResourceVaultKeyValue(substate),
        )) => to_api_account_vault_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Account(
            AccountTypedSubstateValue::ResourcePreferenceKeyValue(substate),
        )) => to_api_account_resource_preference_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Account(
            AccountTypedSubstateValue::AuthorizedDepositorKeyValue(substate),
        )) => to_api_account_authorized_depositor_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::AccessController(
            AccessControllerTypedSubstateValue::Field(
                AccessControllerTypedFieldSubstateValue::State(substate),
            ),
        )) => to_api_access_controller_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::GenericScryptoComponent(
            GenericScryptoComponentFieldValue::State(substate),
        )) => to_api_generic_scrypto_component_state_substate(context, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::GenericKeyValueStoreEntry(substate),
        ) => to_api_generic_key_value_store_substate(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::OneResourcePool(
            OneResourcePoolTypedSubstateValue::Field(
                OneResourcePoolTypedFieldSubstateValue::State(substate),
            ),
        )) => to_api_one_resource_pool_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::TwoResourcePool(
            TwoResourcePoolTypedSubstateValue::Field(
                TwoResourcePoolTypedFieldSubstateValue::State(substate),
            ),
        )) => to_api_two_resource_pool_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::MultiResourcePool(
            MultiResourcePoolTypedSubstateValue::Field(
                MultiResourcePoolTypedFieldSubstateValue::State(substate),
            ),
        )) => to_api_multi_resource_pool_substate(context, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::BlueprintVersionDefinitionKeyValue(substate),
        )) => to_api_package_blueprint_definition_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::BlueprintVersionDependenciesKeyValue(substate),
        )) => to_api_package_blueprint_dependencies_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::CodeVmTypeKeyValue(substate),
        )) => to_api_package_code_vm_type_entry_substate(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::CodeOriginalCodeKeyValue(substate),
        )) => {
            to_api_package_code_original_code_entry_substate(context, typed_substate_key, substate)?
        }
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::CodeInstrumentedCodeKeyValue(substate),
        )) => to_api_package_code_instrumented_code_entry_substate(
            context,
            typed_substate_key,
            substate,
        )?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::BlueprintVersionAuthConfigKeyValue(substate),
        )) => to_api_package_auth_template_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::BlueprintVersionRoyaltyConfigKeyValue(substate),
        )) => to_api_package_blueprint_royalty_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::MainModule(TypedMainModuleSubstateValue::TransactionTracker(
            TypedTransactionTrackerFieldValue::TransactionTracker(substate),
        )) => to_api_transaction_tracker_substate(context, substate)?,
        TypedSubstateValue::MainModule(
            TypedMainModuleSubstateValue::TransactionTrackerCollectionEntry(substate),
        ) => to_api_transaction_tracker_collection_entry(context, typed_substate_key, substate)?,
        TypedSubstateValue::Schema(substate) => {
            to_api_schema_entry_substate(context, typed_substate_key, substate)?
        }
    })
}
