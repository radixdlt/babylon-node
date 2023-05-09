use crate::core_api::*;
use radix_engine::types::AccessControllerOffset;

use radix_engine::blueprints::access_controller::AccessControllerSubstate;
use radix_engine::system::node_modules::access_rules::MethodAccessRulesSubstate;
use radix_engine_interface::types::{
    AccessRulesOffset, ACCESS_RULES_BASE_MODULE, OBJECT_BASE_MODULE,
};
use state_manager::query::{dump_component_state, VaultData};
use std::ops::Deref;

use super::map_to_descendent_id;

pub(crate) async fn handle_state_access_controller(
    state: State<CoreApiState>,
    Json(request): Json<models::StateAccessControllerRequest>,
) -> Result<Json<models::StateAccessControllerResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let controller_address =
        extract_component_address(&extraction_context, &request.controller_address)
            .map_err(|err| err.into_response_error("controller_address"))?;

    if !request.controller_address.starts_with("accesscontroller_")
        && !request.controller_address.starts_with("controller_")
    {
        return Err(client_error("Only access controller addresses work for this endpoint. Try another endpoint instead."));
    }

    let database = state.database.read();

    let access_controller_substate: AccessControllerSubstate = read_mandatory_substate(
        database.deref(),
        controller_address.as_node_id(),
        OBJECT_BASE_MODULE,
        &AccessControllerOffset::AccessController.into(),
    )?;

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        controller_address.as_node_id(),
        ACCESS_RULES_BASE_MODULE,
        &AccessRulesOffset::AccessRules.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), controller_address);

    let state_owned_vaults = component_dump
        .vaults
        .into_values()
        .map(|vault_data| match vault_data {
            VaultData::NonFungible {
                resource_address,
                amount,
                ids,
            } => to_api_non_fungible_resource_amount(
                &mapping_context,
                &resource_address,
                &amount,
                &ids,
            ),
            VaultData::Fungible {
                resource_address,
                amount,
            } => to_api_fungible_resource_amount(&mapping_context, &resource_address, &amount),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let descendent_ids = component_dump
        .descendents
        .into_iter()
        .filter(|(_, _, depth)| *depth > 0)
        .map(|(parent, node, depth)| map_to_descendent_id(parent, node, depth))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(models::StateAccessControllerResponse {
        state: Some(to_api_access_controller_substate(
            &mapping_context,
            &access_controller_substate,
        )?),
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
        state_owned_vaults,
        descendent_ids,
    })
    .map(Json)
}
