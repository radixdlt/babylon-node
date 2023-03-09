use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{NodeModuleId, SubstateOffset, ValidatorOffset};
use radix_engine_interface::api::types::{AccessRulesOffset, RENodeId};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::{dump_component_state, VaultData};

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

    let component_state = {
        let substate_offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(validator_address.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::Validator(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(validator_address.into()),
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let component_dump = dump_component_state(state_manager.store(), validator_address)
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

    Ok(models::StateValidatorResponse {
        state: Some(to_api_validator_substate(
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
