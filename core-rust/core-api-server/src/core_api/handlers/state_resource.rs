use crate::core_api::*;
use radix_engine::system::substates::PersistedSubstate;
use radix_engine::types::{
    AccessRulesChainOffset, GlobalAddress, MetadataOffset, ResourceManagerOffset, SubstateOffset,
};
use radix_engine_interface::api::types::NodeModuleId;

use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_state_resource(
    state: Extension<CoreApiState>,
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

    let resource_node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Resource(resource_address))?;

    let resource_manager = {
        let substate_offset =
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let loaded_substate = read_known_substate(
            state_manager,
            resource_node_id,
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::ResourceManager(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let metadata = {
        let substate_offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let loaded_substate = read_known_substate(
            state_manager,
            resource_node_id,
            NodeModuleId::Metadata,
            &substate_offset,
        )?;
        let PersistedSubstate::Metadata(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let access_rules = {
        let substate_offset =
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let loaded_substate = read_known_substate(
            state_manager,
            resource_node_id,
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::AccessRulesChain(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let vault_access_rules = {
        let substate_offset =
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let loaded_substate = read_known_substate(
            state_manager,
            resource_node_id,
            NodeModuleId::AccessRules1,
            &substate_offset,
        )?;
        let PersistedSubstate::AccessRulesChain(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let mapping_context = MappingContext::new(&state_manager.network);

    Ok(models::StateResourceResponse {
        manager: Some(to_api_resource_manager_substate(
            &mapping_context,
            &resource_manager,
        )?),
        metadata: Some(to_api_metadata_substate(&mapping_context, &metadata)?),
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
