use crate::core_api::*;
use radix_engine::blueprints::resource::{
    FungibleResourceManagerDivisibilitySubstate, FungibleResourceManagerTotalSupplySubstate,
    NonFungibleResourceManagerDataSchemaSubstate, NonFungibleResourceManagerDataSubstate,
    NonFungibleResourceManagerIdTypeSubstate, NonFungibleResourceManagerTotalSupplySubstate,
};
use radix_engine::system::node_modules::access_rules::MethodAccessRulesSubstate;
use radix_engine_interface::types::SysModuleId;
use radix_engine_interface::types::{
    AccessRulesOffset, FungibleResourceManagerOffset, NonFungibleResourceManagerOffset,
};
use std::ops::Deref;

use radix_engine_common::types::EntityType;

enum ManagerByType {
    Fungible(
        FungibleResourceManagerDivisibilitySubstate,
        FungibleResourceManagerTotalSupplySubstate,
    ),
    NonFungible(
        NonFungibleResourceManagerIdTypeSubstate,
        NonFungibleResourceManagerTotalSupplySubstate,
        NonFungibleResourceManagerDataSchemaSubstate,
        NonFungibleResourceManagerDataSubstate,
    ),
}

pub(crate) async fn handle_state_resource(
    state: State<CoreApiState>,
    Json(request): Json<models::StateResourceRequest>,
) -> Result<Json<models::StateResourceResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let database = state.database.read();

    let resource_node_id = resource_address.as_node_id();
    let is_fungible = resource_node_id.entity_type() == Some(EntityType::GlobalFungibleResource);
    let manager = if is_fungible {
        ManagerByType::Fungible(
            read_mandatory_substate(
                database.deref(),
                resource_node_id,
                SysModuleId::Object.into(),
                &FungibleResourceManagerOffset::Divisibility.into(),
            )?,
            read_mandatory_substate(
                database.deref(),
                resource_node_id,
                SysModuleId::Object.into(),
                &FungibleResourceManagerOffset::TotalSupply.into(),
            )?,
        )
    } else {
        ManagerByType::NonFungible(
            read_mandatory_substate(
                database.deref(),
                resource_node_id,
                SysModuleId::Object.into(),
                &NonFungibleResourceManagerOffset::IdType.into(),
            )?,
            read_mandatory_substate(
                database.deref(),
                resource_node_id,
                SysModuleId::Object.into(),
                &NonFungibleResourceManagerOffset::TotalSupply.into(),
            )?,
            read_mandatory_substate(
                database.deref(),
                resource_node_id,
                SysModuleId::Object.into(),
                &NonFungibleResourceManagerOffset::DataSchema.into(),
            )?,
            read_mandatory_substate(
                database.deref(),
                resource_node_id,
                SysModuleId::Object.into(),
                &NonFungibleResourceManagerOffset::Data.into(),
            )?,
        )
    };

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        SysModuleId::AccessRules.into(),
        &AccessRulesOffset::AccessRules.into(),
    )?;

    Ok(models::StateResourceResponse {
        manager: Box::new(to_api_resource_manager(&mapping_context, &manager)?),
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
        vault_access_rules: None, /* TODO: bring it back */
    })
    .map(Json)
}

fn to_api_resource_manager(
    context: &MappingContext,
    manager: &ManagerByType,
) -> Result<models::StateResourceResponseManager, MappingError> {
    Ok(match manager {
        ManagerByType::Fungible(divisiility, total_supply) => {
            models::StateResourceResponseManager::StateFungibleResource {
                divisibility: Box::new(to_api_fungible_resource_manager_divisibility_substate(
                    divisiility,
                )?),
                total_supply: Box::new(to_api_fungible_resource_manager_total_supply_substate(
                    total_supply,
                )?),
            }
        }
        ManagerByType::NonFungible(id_type, total_supply, data_schema, data) => {
            models::StateResourceResponseManager::StateNonFungibleResource {
                id_type: Box::new(to_api_non_fungible_resource_manager_id_type_substate(
                    id_type,
                )?),
                total_supply: Box::new(to_api_non_fungible_resource_manager_total_supply_substate(
                    total_supply,
                )?),
                data_schema: Box::new(to_api_non_fungible_resource_manager_data_schema_substate(
                    context,
                    data_schema,
                )?),
                data: Box::new(to_api_non_fungible_resource_manager_data_substate(data)?),
            }
        }
    })
}
