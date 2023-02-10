use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{
    AccessControllerOffset, AccessRulesChainOffset, GlobalAddress, MetadataOffset, SubstateOffset,
};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::dump_component_state;

use super::map_to_descendent_id;

pub(crate) async fn handle_state_access_controller(
    state: Extension<CoreApiState>,
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

    let component_node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Component(controller_address))?;

    let component_state = {
        let substate_offset =
            SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let loaded_substate =
            read_known_substate(state_manager, component_node_id, &substate_offset)?;
        let PersistedSubstate::AccessController(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_metadata = {
        let substate_offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let loaded_substate =
            read_known_substate(state_manager, component_node_id, &substate_offset)?;
        let PersistedSubstate::Metadata(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_access_rules = {
        let substate_offset =
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let loaded_substate =
            read_known_substate(state_manager, component_node_id, &substate_offset)?;
        let PersistedSubstate::AccessRulesChain(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let component_dump = dump_component_state(state_manager.store(), controller_address)
        .map_err(|err| server_error(format!("Error traversing component state: {:?}", err)))?;

    let state_owned_vaults = component_dump
        .vaults
        .into_iter()
        .map(|vault| to_api_vault_substate(&mapping_context, &vault))
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
        metadata: Some(to_api_metadata_substate(
            &mapping_context,
            &component_metadata,
        )?),
        state_owned_vaults,
        descendent_ids,
    })
}
