use super::super::*;

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
        SystemBoot::V2(..) => todo!(),
    };

    Ok(models::Substate::BootLoaderModuleFieldSystemBootSubstate {
        is_locked: false,
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
        KernelBoot::V1 => serde_json::Value::Object(serde_json::Map::default()),
        KernelBoot::V2(_) => todo!(),
    };

    Ok(models::Substate::BootLoaderModuleFieldKernelBootSubstate {
        is_locked: false,
        value,
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
