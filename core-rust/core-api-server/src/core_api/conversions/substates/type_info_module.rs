use super::super::*;
use super::*;
use crate::core_api::models;

use crate::engine_prelude::*;

pub fn to_api_system_boot_substate(
    context: &MappingContext,
    _state_mapping_lookups: &StateMappingLookups,
    substate: &SystemBoot,
) -> Result<models::Substate, MappingError> {
    let value = match substate {
        SystemBoot::V1(system_parameters) => models::BootLoaderModuleFieldSystemBootValue::new(
            to_api_system_parameters(context, system_parameters)?,
        ),
    };

    Ok(models::Substate::BootLoaderModuleFieldSystemBootSubstate {
        is_locked: true,
        value: Box::new(value),
    })
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
        is_locked: true,
        value: Box::new(value),
    })
}

pub fn to_api_type_info_substate(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    substate: &TypeInfoSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(system_field_substate!(
        substate,
        TypeInfoModuleFieldTypeInfo,
        value => {
            let details = match value {
                TypeInfoSubstate::Object(ObjectInfo {blueprint_info, object_type}) => {
                    let (global, module_versions) = match object_type {
                        ObjectType::Global { modules } => (true, Some(modules)),
                        ObjectType::Owned => (false, None)
                    };
                    models::TypeInfoDetails::ObjectTypeInfoDetails {
                        module_versions: module_versions
                            .iter()
                            .flat_map(|modules| modules.iter())
                            .map(|(module_id, version)| -> Result<_, MappingError> {
                                Ok(models::ModuleVersion {
                                    module: to_api_attached_module_id(module_id),
                                    version: to_api_blueprint_version(context, version)?,
                                })
                            })
                            .collect::<Result<_, _>>()?,
                        blueprint_info: Box::new(to_api_blueprint_info(
                            context,
                            state_mapping_lookups,
                            blueprint_info,
                        )?),
                        global,
                    }
                },
                TypeInfoSubstate::KeyValueStore(key_value_store_info) => {
                    models::TypeInfoDetails::KeyValueStoreTypeInfoDetails {
                        key_value_store_info: Box::new(to_api_key_value_store_info(
                            context,
                            state_mapping_lookups,
                            key_value_store_info,
                        )?),
                    }
                }
                TypeInfoSubstate::GlobalAddressReservation(_) => {
                    return Err(MappingError::UnexpectedPersistedData {
                        message: "GlobalAddressReservation was persisted".to_string(),
                    })
                }
                TypeInfoSubstate::GlobalAddressPhantom(_) => {
                    return Err(MappingError::UnexpectedPersistedData {
                        message: "GlobalAddressPhantom was persisted".to_string(),
                    })
                }
            }
        },
        Value {
            details: Some(details),
        }
    ))
}

pub fn to_api_blueprint_info(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    blueprint_info: &BlueprintInfo,
) -> Result<models::BlueprintInfo, MappingError> {
    let BlueprintInfo {
        blueprint_id:
            BlueprintId {
                package_address,
                blueprint_name,
            },
        blueprint_version,
        outer_obj_info,
        features,
        generic_substitutions,
    } = blueprint_info;

    Ok(models::BlueprintInfo {
        package_address: to_api_package_address(context, package_address)?,
        blueprint_name: blueprint_name.to_string(),
        blueprint_version: to_api_blueprint_version(context, blueprint_version)?,
        outer_object: match outer_obj_info {
            OuterObjectInfo::Some { outer_object } => {
                Some(to_api_global_address(context, outer_object)?)
            }
            OuterObjectInfo::None => None,
        },
        generic_substitutions: generic_substitutions
            .iter()
            .map(|substitution| {
                to_api_generic_substitution(context, state_mapping_lookups, substitution)
            })
            .collect::<Result<Vec<_>, _>>()?,
        features: features.iter().cloned().collect(),
    })
}

pub fn to_api_generic_substitution(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    substitution: &GenericSubstitution,
) -> Result<models::GenericSubstitution, MappingError> {
    Ok(match substitution {
        GenericSubstitution::Local(scoped_type_id) => {
            models::GenericSubstitution::LocalGenericSubstitution {
                scoped_type_id: Box::new(to_api_scoped_type_id(context, scoped_type_id)?),
            }
        }
        GenericSubstitution::Remote(blueprint_type_identifier) => {
            let resolved =
                state_mapping_lookups.resolve_generic_remote(blueprint_type_identifier)?;
            models::GenericSubstitution::RemoteGenericSubstitution {
                blueprint_type_identifier: Box::new(to_api_blueprint_type_identifier(
                    context,
                    blueprint_type_identifier,
                )?),
                resolved_full_type_id: resolved
                    .map(|scoped_type_id| -> Result<_, MappingError> {
                        Ok(Box::new(to_api_fully_scoped_type_id(
                            context,
                            &scoped_type_id,
                        )?))
                    })
                    .transpose()?,
            }
        }
    })
}

pub fn to_api_blueprint_type_identifier(
    context: &MappingContext,
    blueprint_type_identifier: &BlueprintTypeIdentifier,
) -> Result<models::BlueprintTypeIdentifier, MappingError> {
    let BlueprintTypeIdentifier {
        package_address,
        blueprint_name,
        type_name,
    } = blueprint_type_identifier;
    Ok(models::BlueprintTypeIdentifier {
        package_address: to_api_package_address(context, package_address)?,
        blueprint_name: blueprint_name.clone(),
        type_name: type_name.clone(),
    })
}

pub fn to_api_key_value_store_info(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    key_value_store_info: &KeyValueStoreInfo,
) -> Result<models::KeyValueStoreInfo, MappingError> {
    let KeyValueStoreInfo {
        generic_substitutions:
            KeyValueStoreGenericSubstitutions {
                key_generic_substitution,
                value_generic_substitution,
                allow_ownership,
            },
    } = key_value_store_info;
    Ok(models::KeyValueStoreInfo {
        key_generic_substitution: Some(to_api_generic_substitution(
            context,
            state_mapping_lookups,
            key_generic_substitution,
        )?),
        value_generic_substitution: Some(to_api_generic_substitution(
            context,
            state_mapping_lookups,
            value_generic_substitution,
        )?),
        allow_ownership: *allow_ownership,
    })
}

fn to_api_system_parameters(
    context: &MappingContext,
    system_parameters: &SystemParameters,
) -> Result<models::SystemParameters, MappingError> {
    let SystemParameters {
        network_definition,
        costing_parameters,
        limit_parameters,
        max_per_function_royalty_in_xrd,
    } = system_parameters;
    Ok(models::SystemParameters {
        network_definition: Box::new(to_api_network_definition(context, network_definition)?),
        costing_parameters: Box::new(to_api_system_costing_parameters(
            context,
            costing_parameters,
        )?),
        limit_parameters: Box::new(to_api_limit_parameters(context, limit_parameters)?),
        xrd_max_per_function_royalty: to_api_decimal(max_per_function_royalty_in_xrd),
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
        logical_name: logical_name.clone(),
        hrp_suffix: hrp_suffix.clone(),
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
