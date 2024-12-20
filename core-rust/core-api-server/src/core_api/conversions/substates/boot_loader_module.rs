use super::super::*;

use crate::core_api::models;
use crate::prelude::*;

pub fn to_api_system_boot_substate(
    context: &MappingContext,
    _state_mapping_lookups: &StateMappingLookups,
    substate: &SystemBoot,
) -> Result<models::Substate, MappingError> {
    let value = match substate {
        SystemBoot::V1(system_parameters) => models::BootLoaderModuleFieldSystemBootValue {
            system_version: None,
            system_parameters: Box::new(to_api_system_parameters(context, system_parameters)?),
        },
        SystemBoot::V2(system_version, system_parameters) => {
            models::BootLoaderModuleFieldSystemBootValue {
                system_version: Some(to_api_system_version(system_version)),
                system_parameters: Box::new(to_api_system_parameters(context, system_parameters)?),
            }
        }
    };

    Ok(models::Substate::BootLoaderModuleFieldSystemBootSubstate {
        is_locked: false,
        value: Box::new(value),
    })
}

fn to_api_system_version(system_version: &SystemVersion) -> models::SystemVersion {
    match system_version {
        SystemVersion::V1 => models::SystemVersion::V1,
        SystemVersion::V2 => models::SystemVersion::V2,
        SystemVersion::V3 => models::SystemVersion::V3,
    }
}

pub fn to_api_vm_boot_substate(
    _context: &MappingContext,
    _state_mapping_lookups: &StateMappingLookups,
    substate: &VmBoot,
) -> Result<models::Substate, MappingError> {
    let value = match substate {
        VmBoot::V1 { scrypto_version } => {
            models::BootLoaderModuleFieldVmBootValue::new(*scrypto_version as i64)
        }
    };

    Ok(models::Substate::BootLoaderModuleFieldVmBootSubstate {
        is_locked: false,
        value: Box::new(value),
    })
}

pub fn to_api_kernel_boot_substate(
    _context: &MappingContext,
    _state_mapping_lookups: &StateMappingLookups,
    substate: &KernelBoot,
) -> Result<models::Substate, MappingError> {
    let value = match substate {
        // Note: this is how OpenAPI generator represents an empty object type, even when named:
        KernelBoot::V1 => models::BootLoaderModuleFieldKernelBootValue {
            always_visible_nodes_version: None,
        },
        KernelBoot::V2 {
            global_nodes_version,
        } => models::BootLoaderModuleFieldKernelBootValue {
            always_visible_nodes_version: Some(match global_nodes_version {
                AlwaysVisibleGlobalNodesVersion::V1 => models::AlwaysVisibleGlobalNodesVersion::V1,
                AlwaysVisibleGlobalNodesVersion::V2 => models::AlwaysVisibleGlobalNodesVersion::V2,
            }),
        },
    };

    Ok(models::Substate::BootLoaderModuleFieldKernelBootSubstate {
        is_locked: false,
        value: Box::new(value),
    })
}

fn to_api_system_parameters(
    context: &MappingContext,
    system_parameters: &SystemParameters,
) -> Result<models::SystemParameters, MappingError> {
    let SystemParameters {
        network_definition,
        costing_module_config,
        costing_parameters,
        limit_parameters,
    } = system_parameters;
    Ok(models::SystemParameters {
        network_definition: Box::new(to_api_network_definition(context, network_definition)?),
        costing_module_config: Box::new(to_api_costing_module_config(
            context,
            costing_module_config,
        )?),
        costing_parameters: Box::new(to_api_system_costing_parameters(
            context,
            costing_parameters,
        )?),
        limit_parameters: Box::new(to_api_limit_parameters(context, limit_parameters)?),
    })
}

fn to_api_costing_module_config(
    _context: &MappingContext,
    costing_module_config: &CostingModuleConfig,
) -> Result<models::CostingModuleConfig, MappingError> {
    let CostingModuleConfig {
        max_per_function_royalty_in_xrd,
        apply_execution_cost_2,
        apply_boot_ref_check_costing,
    } = costing_module_config;
    Ok(models::CostingModuleConfig {
        xrd_max_per_function_royalty: to_api_decimal(max_per_function_royalty_in_xrd),
        apply_execution_cost_for_all_system_calls: *apply_execution_cost_2,
        apply_boot_ref_check_costing: *apply_boot_ref_check_costing,
    })
}

fn to_api_network_definition(
    _context: &MappingContext,
    network_definition: &NetworkDefinition,
) -> Result<models::NetworkDefinition, MappingError> {
    let NetworkDefinition {
        id,
        logical_name,
        hrp_suffix,
    } = network_definition;
    Ok(models::NetworkDefinition {
        id: to_api_u8_as_i32(*id),
        logical_name: logical_name.to_string(),
        hrp_suffix: hrp_suffix.to_string(),
    })
}

fn to_api_system_costing_parameters(
    _context: &MappingContext,
    costing_parameters: &CostingParameters,
) -> Result<models::SystemCostingParameters, MappingError> {
    let CostingParameters {
        execution_cost_unit_price,
        execution_cost_unit_limit,
        execution_cost_unit_loan,
        finalization_cost_unit_price,
        finalization_cost_unit_limit,
        usd_price,
        state_storage_price,
        archive_storage_price,
    } = costing_parameters;
    Ok(models::SystemCostingParameters {
        execution_cost_unit_price: to_api_decimal(execution_cost_unit_price),
        execution_cost_unit_limit: to_api_u32_as_i64(*execution_cost_unit_limit),
        execution_cost_unit_loan: to_api_u32_as_i64(*execution_cost_unit_loan),
        finalization_cost_unit_price: to_api_decimal(finalization_cost_unit_price),
        finalization_cost_unit_limit: to_api_u32_as_i64(*finalization_cost_unit_limit),
        xrd_usd_price: to_api_decimal(usd_price),
        xrd_storage_price: to_api_decimal(state_storage_price),
        xrd_archive_storage_price: to_api_decimal(archive_storage_price),
    })
}

fn to_api_limit_parameters(
    _context: &MappingContext,
    limit_parameters: &LimitParameters,
) -> Result<models::LimitParameters, MappingError> {
    let LimitParameters {
        max_call_depth,
        max_heap_substate_total_bytes,
        max_track_substate_total_bytes,
        max_substate_key_size,
        max_substate_value_size,
        max_invoke_input_size,
        max_event_size,
        max_log_size,
        max_panic_message_size,
        max_number_of_logs,
        max_number_of_events,
    } = limit_parameters;
    Ok(models::LimitParameters {
        max_call_depth: to_api_usize_as_string(*max_call_depth),
        max_heap_substate_total_bytes: to_api_usize_as_string(*max_heap_substate_total_bytes),
        max_track_substate_total_bytes: to_api_usize_as_string(*max_track_substate_total_bytes),
        max_substate_key_size: to_api_usize_as_string(*max_substate_key_size),
        max_substate_value_size: to_api_usize_as_string(*max_substate_value_size),
        max_invoke_input_size: to_api_usize_as_string(*max_invoke_input_size),
        max_event_size: to_api_usize_as_string(*max_event_size),
        max_log_size: to_api_usize_as_string(*max_log_size),
        max_panic_message_size: to_api_usize_as_string(*max_panic_message_size),
        max_number_of_logs: to_api_usize_as_string(*max_number_of_logs),
        max_number_of_events: to_api_usize_as_string(*max_number_of_events),
    })
}

pub fn to_api_transaction_validator_configuration_substate(
    _context: &MappingContext,
    substate: &TransactionValidationConfigurationSubstate,
) -> Result<models::Substate, MappingError> {
    let config = match substate.as_versions() {
        TransactionValidationConfigurationVersions::V1(config) => {
            let TransactionValidationConfigV1 {
                max_signer_signatures_per_intent,
                max_references_per_intent,
                min_tip_percentage,
                max_tip_percentage,
                max_epoch_range,
                max_instructions,
                message_validation,
                v1_transactions_allow_notary_to_duplicate_signer,
                preparation_settings,
                manifest_validation,
                v2_transactions_allowed,
                min_tip_basis_points,
                max_tip_basis_points,
                max_subintent_depth,
                max_total_signature_validations,
                max_total_references,
            } = config;

            let message_validation = {
                let MessageValidationConfig {
                    max_plaintext_message_length,
                    max_encrypted_message_length,
                    max_mime_type_length,
                    max_decryptors,
                } = message_validation;
                Box::new(models::MessageValidationConfig {
                    max_plaintext_message_length: to_api_usize_as_string(
                        *max_plaintext_message_length,
                    ),
                    max_encrypted_message_length: to_api_usize_as_string(
                        *max_encrypted_message_length,
                    ),
                    max_mime_type_length: to_api_usize_as_string(*max_mime_type_length),
                    max_decryptors: to_api_usize_as_string(*max_decryptors),
                })
            };

            let preparation_settings = {
                let PreparationSettingsV1 {
                    v2_transactions_permitted,
                    max_user_payload_length,
                    max_ledger_payload_length,
                    max_child_subintents_per_intent,
                    max_subintents_per_transaction,
                    max_blobs,
                } = preparation_settings;
                Box::new(models::PreparationSettings {
                    v2_transactions_permitted: *v2_transactions_permitted,
                    max_user_payload_length: to_api_usize_as_string(*max_user_payload_length),
                    max_ledger_payload_length: to_api_usize_as_string(*max_ledger_payload_length),
                    max_child_subintents_per_intent: to_api_usize_as_string(
                        *max_child_subintents_per_intent,
                    ),
                    max_subintents_per_transaction: to_api_usize_as_string(
                        *max_subintents_per_transaction,
                    ),
                    max_blobs: to_api_usize_as_string(*max_blobs),
                })
            };

            let manifest_validation = match manifest_validation {
                ManifestValidationRuleset::BabylonBasicValidator => {
                    models::ManifestValidationRuleset::Basic
                }
                ManifestValidationRuleset::Interpreter(
                    InterpreterValidationRulesetSpecifier::Cuttlefish,
                ) => models::ManifestValidationRuleset::Cuttlefish,
                ManifestValidationRuleset::Interpreter(
                    InterpreterValidationRulesetSpecifier::AllValidations,
                ) => {
                    return Err(MappingError::UnexpectedPersistedData {
                        message: "InterpreterValidationRulesetSpecifier::AllValidations is only expected in testing".to_string(),
                    });
                }
            };

            Box::new(models::TransactionValidationConfig {
                max_signer_signatures_per_intent: to_api_usize_as_string(
                    *max_signer_signatures_per_intent,
                ),
                max_references_per_intent: to_api_usize_as_string(*max_references_per_intent),
                min_tip_percentage: to_api_u16_as_i32(*min_tip_percentage),
                max_tip_percentage: to_api_u16_as_i32(*max_tip_percentage),
                max_epoch_range: to_api_u64_as_string(*max_epoch_range),
                max_instructions: to_api_usize_as_string(*max_instructions),
                message_validation,
                v1_transactions_allow_notary_to_duplicate_signer:
                    *v1_transactions_allow_notary_to_duplicate_signer,
                preparation_settings,
                manifest_validation,
                v2_transactions_allowed: *v2_transactions_allowed,
                min_tip_basis_points: to_api_u32_as_i64(*min_tip_basis_points),
                max_tip_basis_points: to_api_u32_as_i64(*max_tip_basis_points),
                max_subintent_depth: to_api_usize_as_string(*max_subintent_depth),
                max_total_signature_validations: to_api_usize_as_string(
                    *max_total_signature_validations,
                ),
                max_total_references: to_api_usize_as_string(*max_total_references),
            })
        }
    };

    Ok(
        models::Substate::BootLoaderModuleFieldTransactionValidationConfigurationSubstate {
            is_locked: false,
            config,
        },
    )
}
