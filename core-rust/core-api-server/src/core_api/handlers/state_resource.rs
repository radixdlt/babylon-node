use crate::core_api::*;
use radix_engine::blueprints::resource::{
    FungibleResourceManagerSubstate, NonFungibleResourceManagerSubstate,
};

use radix_engine::system::node_modules::access_rules::MethodAccessRulesSubstate;
use radix_engine_interface::types::SysModuleId;
use radix_engine_interface::types::{
    AccessRulesOffset, FungibleResourceManagerOffset, NonFungibleResourceManagerOffset,
};
use std::ops::Deref;

use radix_engine_common::types::EntityType;

enum ManagerByType {
    Fungible(FungibleResourceManagerSubstate),
    NonFungible(NonFungibleResourceManagerSubstate),
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
    // TODO: this is ugly as hell, can we do better?
    let is_fungible = resource_node_id.entity_type() == Some(EntityType::GlobalFungibleResource);
    let manager = if is_fungible {
        ManagerByType::Fungible(read_mandatory_substate(
            database.deref(),
            resource_node_id,
            SysModuleId::Object.into(),
            &FungibleResourceManagerOffset::ResourceManager.into(),
        )?)
    } else {
        ManagerByType::NonFungible(read_mandatory_substate(
            database.deref(),
            resource_node_id,
            SysModuleId::Object.into(),
            &NonFungibleResourceManagerOffset::ResourceManager.into(),
        )?)
    };

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        SysModuleId::AccessRules.into(),
        &AccessRulesOffset::AccessRules.into(),
    )?;

    // TODO: fix this, it was AccessRules1 module
    let vault_method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        SysModuleId::AccessRules.into(),
        &AccessRulesOffset::AccessRules.into(),
    )?;

    Ok(models::StateResourceResponse {
        manager: Some(match &manager {
            ManagerByType::Fungible(manager) => {
                to_api_fungible_resource_manager_substate(&mapping_context, manager)?
            }
            ManagerByType::NonFungible(manager) => {
                to_api_non_fungible_resource_manager_substate(&mapping_context, manager)?
            }
        }),
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
        vault_access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &vault_method_access_rules_substate,
        )?),
    })
    .map(Json)
}
