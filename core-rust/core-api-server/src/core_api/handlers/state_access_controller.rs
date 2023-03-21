use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{AccessControllerOffset, NodeModuleId, SubstateOffset};
use radix_engine_interface::api::types::{AccessRulesOffset, RENodeId};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::{dump_component_state, VaultData};

use super::map_to_descendent_id;

pub(crate) async fn handle_state_access_controller(
    state: State<CoreApiState>,
    request: Json<models::StateAccessControllerRequest>,
) -> Result<Json<models::StateAccessControllerResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_state_access_controller_internal)
}

fn handle_state_access_controller_internal(
    state_manager: &ActualStateManager,
    request: models::StateAccessControllerRequest,
) -> Result<models::StateAccessControllerResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let mapping_context = MappingContext::new(&state_manager.network);
    let extraction_context = ExtractionContext::new(&state_manager.network);

    let controller_address =
        extract_component_address(&extraction_context, &request.controller_address)
            .map_err(|err| err.into_response_error("controller_address"))?;

    if !request.controller_address.starts_with("accesscontroller_")
        && !request.controller_address.starts_with("controller_")
    {
        return Err(client_error("Only access controller addresses work for this endpoint. Try another endpoint instead."));
    }

    let component_state = {
        let substate_offset =
            SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let loaded_substate = read_mandatory_substate_or_server_error(
            state_manager,
            RENodeId::GlobalObject(controller_address.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::AccessController(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_mandatory_substate_or_server_error(
            state_manager,
            RENodeId::GlobalObject(controller_address.into()),
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let component_dump = dump_component_state(state_manager.store(), controller_address)
        .map_err(|err| server_error(format!("Error traversing component state: {err:?}")))?;

    let state_owned_vaults = component_dump
        .vaults
        .into_iter()
        .map(|vault| match vault {
            VaultData::NonFungible {
                resource_address,
                ids,
            } => to_api_non_fungible_resource_amount(&mapping_context, &resource_address, &ids),
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
            &component_state,
        )?),
        access_rules: Some(to_api_access_rules_chain_substate(
            &mapping_context,
            &component_access_rules,
        )?),
        state_owned_vaults,
        descendent_ids,
    })
}
