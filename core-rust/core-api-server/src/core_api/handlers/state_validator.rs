use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{
    AccessRulesChainOffset, GlobalAddress, MetadataOffset, SubstateOffset, ValidatorOffset,
};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::dump_component_state;

use super::map_to_descendent_id;

pub(crate) async fn handle_state_validator(
    state: Extension<CoreApiState>,
    request: Json<models::StateValidatorRequest>,
) -> Result<Json<models::StateValidatorResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_state_validator_internal)
}

fn handle_state_validator_internal(
    state_manager: &ActualStateManager,
    request: models::StateValidatorRequest,
) -> Result<models::StateValidatorResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let mapping_context = MappingContext::new(&state_manager.network);
    let extraction_context = ExtractionContext::new(&state_manager.network);

    let validator_address =
        extract_component_address(&extraction_context, &request.validator_address)
            .map_err(|err| err.into_response_error("validator_address"))?;

    if !request.validator_address.starts_with("validator_") {
        return Err(client_error(
            "Only validator addresses work for this endpoint. Try another endpoint instead.",
        ));
    }

    let component_node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Component(validator_address))?;

    let component_state = {
        let substate_offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let loaded_substate =
            read_known_substate(state_manager, component_node_id, &substate_offset)?;
        let PersistedSubstate::Validator(substate) = loaded_substate else {
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

    let component_dump = dump_component_state(state_manager.store(), validator_address)
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

    Ok(models::StateValidatorResponse {
        state: Some(to_api_validator_substate(
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
