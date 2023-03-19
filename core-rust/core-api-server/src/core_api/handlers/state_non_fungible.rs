use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{
    scrypto_encode, KeyValueStoreOffset, RENodeId, ResourceManagerOffset, SubstateId,
    SubstateOffset,
};
use radix_engine_interface::api::types::NodeModuleId;

use crate::core_api::models::StateNonFungibleResponse;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_state_non_fungible(
    state: State<CoreApiState>,
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

    let resource_manager = {
        let substate_offset =
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(resource_address.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        match loaded_substate {
            PersistedSubstate::ResourceManager(_) => {
                return Err(client_error(
                    "The specified resource is fungible, not non-fungible",
                ))
            }
            PersistedSubstate::NonFungibleResourceManager(substate) => substate,
            _ => return Err(wrong_substate_type(substate_offset)),
        }
    };

    let non_fungible_id_type = resource_manager.id_type;

    let non_fungible_id = extract_non_fungible_id_from_simple_representation(
        non_fungible_id_type,
        &request.non_fungible_id,
    )
    .map_err(|err| err.into_response_error("non_fungible_id"))?;

    let non_fungible_substate_offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
        scrypto_encode(&non_fungible_id).unwrap(),
    ));

    let substate_id = SubstateId(
        RENodeId::KeyValueStore(resource_manager.non_fungible_table),
        NodeModuleId::SELF,
        non_fungible_substate_offset,
    );

    let key_value_store_entry_substate = {
        let loaded_substate = read_known_substate_from_id(state_manager, &substate_id)?;
        let PersistedSubstate::KeyValueStoreEntry(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_id.2));
        };
        substate
    };

    Ok(StateNonFungibleResponse {
        non_fungible: Some(to_api_key_value_story_entry_substate(
            &mapping_context,
            &substate_id,
            &key_value_store_entry_substate,
        )?),
    })
}
