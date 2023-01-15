use crate::core_api::*;
use radix_engine::types::{AccessRulesChainOffset, Bech32Decoder, MetadataOffset};
use radix_engine::{
    model::PersistedSubstate,
    types::{Bech32Encoder, GlobalAddress, ResourceManagerOffset, SubstateOffset},
};

use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_state_resource(
    state: Extension<CoreApiState>,
    request: Json<models::StateResourceRequest>,
) -> Result<Json<models::StateResourceResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_state_resource_internal)
}

fn handle_state_resource_internal(
    state_manager: &ActualStateManager,
    request: models::StateResourceRequest,
) -> Result<models::StateResourceResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let bech32_decoder = Bech32Decoder::new(&state_manager.network);
    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let resource_address = extract_resource_address(&bech32_decoder, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let resource_node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Resource(resource_address))?;

    let resource_manager = {
        let substate_offset =
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let loaded_substate =
            read_known_substate(state_manager, resource_node_id, &substate_offset)?;
        let PersistedSubstate::ResourceManager(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let metadata = {
        let substate_offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let loaded_substate =
            read_known_substate(state_manager, resource_node_id, &substate_offset)?;
        let PersistedSubstate::Metadata(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let access_rules = {
        let substate_offset =
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let loaded_substate =
            read_known_substate(state_manager, resource_node_id, &substate_offset)?;
        let PersistedSubstate::AccessRulesChain(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let vault_access_rules = {
        let substate_offset =
            SubstateOffset::VaultAccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let loaded_substate =
            read_known_substate(state_manager, resource_node_id, &substate_offset)?;
        let PersistedSubstate::AccessRulesChain(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::StateResourceResponse {
        manager: Some(to_api_resource_manager_substate(
            &bech32_encoder,
            &resource_manager,
        )?),
        metadata: Some(to_api_metadata_substate(&bech32_encoder, &metadata)?),
        access_rules: Some(to_api_access_rules_chain_substate(
            &bech32_encoder,
            &access_rules,
        )?),
        vault_access_rules: Some(to_api_access_rules_chain_substate(
            &bech32_encoder,
            &vault_access_rules,
        )?),
    })
}
