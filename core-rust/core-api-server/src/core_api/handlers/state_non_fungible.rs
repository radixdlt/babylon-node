use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{
    GlobalAddress, NonFungibleStoreOffset, RENodeId, ResourceManagerOffset, SubstateId,
    SubstateOffset,
};
use radix_engine_interface::api::types::NodeModuleId;
use radix_engine_interface::blueprints::resource::ResourceType;

use crate::core_api::models::StateNonFungibleResponse;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_state_non_fungible(
    state: Extension<CoreApiState>,
    request: Json<models::StateNonFungibleRequest>,
) -> Result<Json<StateNonFungibleResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_state_non_fungible_internal)
}

fn handle_state_non_fungible_internal(
    state_manager: &ActualStateManager,
    request: models::StateNonFungibleRequest,
) -> Result<StateNonFungibleResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let mapping_context = MappingContext::new(&state_manager.network);
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

    let non_fungible_id_type = match resource_manager.resource_type {
        ResourceType::Fungible { .. } => {
            return Err(client_error(
                "The specified resource is fungible, not non-fungible",
            ))
        }
        ResourceType::NonFungible { id_type } => id_type,
    };

    let non_fungible_id = extract_non_fungible_id_from_simple_representation(
        non_fungible_id_type,
        &request.non_fungible_id,
    )
    .map_err(|err| err.into_response_error("non_fungible_id"))?;

    let Some(non_fungible_store_id) = resource_manager.nf_store_id else {
        return Err(MappingError::MismatchedSubstateId {
            message: "Resource is not an NFT".to_owned(),
        }
        .into())
    };

    let non_fungible_node_id = RENodeId::NonFungibleStore(non_fungible_store_id);
    let non_fungible_substate_offset =
        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(non_fungible_id));

    let non_fungible = {
        let substate_offset = non_fungible_substate_offset.clone();
        let loaded_substate = read_known_substate(
            state_manager,
            non_fungible_node_id,
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::NonFungible(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let non_fungible_substate_id = SubstateId(
        non_fungible_node_id,
        NodeModuleId::SELF,
        non_fungible_substate_offset,
    );

    Ok(StateNonFungibleResponse {
        non_fungible: Some(to_api_non_fungible_substate(
            &mapping_context,
            &non_fungible_substate_id,
            &non_fungible,
        )?),
    })
}
