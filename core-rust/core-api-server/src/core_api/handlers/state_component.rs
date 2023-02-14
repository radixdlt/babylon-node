use crate::core_api::*;
use radix_engine::system::substates::PersistedSubstate;
use radix_engine::types::{
    AccessRulesChainOffset, ComponentOffset, GlobalAddress, MetadataOffset, RENodeId, SubstateId,
    SubstateOffset,
};
use radix_engine_interface::api::types::{NodeModuleId, RoyaltyOffset};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::dump_component_state;

pub(crate) async fn handle_state_component(
    state: Extension<CoreApiState>,
    request: Json<models::StateComponentRequest>,
) -> Result<Json<models::StateComponentResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_state_component_internal)
}

fn handle_state_component_internal(
    state_manager: &ActualStateManager,
    request: models::StateComponentRequest,
) -> Result<models::StateComponentResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let mapping_context = MappingContext::new(&state_manager.network);
    let extraction_context = ExtractionContext::new(&state_manager.network);

    let component_address =
        extract_component_address(&extraction_context, &request.component_address)
            .map_err(|err| err.into_response_error("component_address"))?;

    if !request.component_address.starts_with("component_")
        && !request.component_address.starts_with("account_")
    {
        // Until we have improvements to the state model for objects, only components should be supported here
        return Err(client_error("Only component addresses starting component_ or account_ currently work with this endpoint. Try another endpoint instead."));
    }

    let component_node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Component(component_address))?;

    let component_info = {
        let substate_offset = SubstateOffset::Component(ComponentOffset::Info);
        let loaded_substate = read_known_substate(
            state_manager,
            component_node_id,
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::ComponentInfo(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_state = {
        let substate_offset = SubstateOffset::Component(ComponentOffset::State);
        let loaded_substate = read_known_substate(
            state_manager,
            component_node_id,
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::ComponentState(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_royalty_config = {
        let substate_offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
        let loaded_substate = read_known_substate(
            state_manager,
            component_node_id,
            NodeModuleId::ComponentRoyalty,
            &substate_offset,
        )?;
        let PersistedSubstate::ComponentRoyaltyConfig(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_royalty_accumulator = {
        let substate_offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
        let loaded_substate = read_known_substate(
            state_manager,
            component_node_id,
            NodeModuleId::ComponentRoyalty,
            &substate_offset,
        )?;
        let PersistedSubstate::ComponentRoyaltyAccumulator(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_metadata = {
        let substate_offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let loaded_substate = read_known_substate(
            state_manager,
            component_node_id,
            NodeModuleId::Metadata,
            &substate_offset,
        )?;
        let PersistedSubstate::Metadata(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_access_rules = {
        let substate_offset =
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let loaded_substate = read_known_substate(
            state_manager,
            component_node_id,
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::AccessRulesChain(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let component_dump = dump_component_state(state_manager.store(), component_address)
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

    Ok(models::StateComponentResponse {
        info: Some(to_api_component_info_substate(
            &mapping_context,
            &component_info,
        )?),
        state: Some(to_api_component_state_substate(
            &mapping_context,
            &component_state,
        )?),
        royalty_config: Some(to_api_component_royalty_config_substate(
            &mapping_context,
            &component_royalty_config,
        )?),
        royalty_accumulator: Some(to_api_component_royalty_accumulator_substate(
            &component_royalty_accumulator,
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

pub(crate) fn map_to_descendent_id(
    parent: Option<SubstateId>,
    node: RENodeId,
    depth: u32,
) -> Result<models::StateComponentDescendentId, MappingError> {
    Ok(models::StateComponentDescendentId {
        parent: Box::new(to_api_substate_id(parent.unwrap())?),
        entity: Box::new(to_api_entity_reference(node)?),
        depth: depth as i32, // Won't go over 100 due to component dumper max depth
    })
}
