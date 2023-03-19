use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{ResourceManagerOffset, SubstateOffset};
use radix_engine_interface::api::types::{AccessRulesOffset, NodeModuleId, RENodeId};

use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_state_resource(
    state: State<CoreApiState>,
    request: Json<models::StateResourceRequest>,
) -> Result<Json<models::StateResourceResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_state_resource_internal)
}

fn handle_state_resource_internal(
    state_manager: &ActualStateManager,
    request: models::StateResourceRequest,
) -> Result<models::StateResourceResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    let extraction_context = ExtractionContext::new(&state_manager.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let resource_manager = {
        let substate_offset =
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(resource_address.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::ResourceManager(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(resource_address.into()),
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let vault_access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(resource_address.into()),
            NodeModuleId::AccessRules1,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let mapping_context = MappingContext::new(&state_manager.network);

    Ok(models::StateResourceResponse {
        manager: Some(to_api_fungible_resource_manager_substate(
            &mapping_context,
            &resource_manager,
        )?),
        access_rules: Some(to_api_access_rules_chain_substate(
            &mapping_context,
            &access_rules,
        )?),
        vault_access_rules: Some(to_api_access_rules_chain_substate(
            &mapping_context,
            &vault_access_rules,
        )?),
    })
}
