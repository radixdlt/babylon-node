use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{
    scrypto_encode, KeyValueStoreOffset, RENodeId, ResourceManagerOffset, SubstateId,
    SubstateOffset,
};
use radix_engine_interface::api::types::NodeModuleId;
use std::ops::Deref;

use crate::core_api::models::StateNonFungibleResponse;

pub(crate) async fn handle_state_non_fungible(
    state: State<CoreApiState>,
    Json(request): Json<models::StateNonFungibleRequest>,
) -> Result<Json<StateNonFungibleResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let state_manager = state.state_manager.read();
    let resource_manager = {
        let substate_offset =
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let loaded_substate = read_mandatory_substate(
            state_manager.deref(),
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

    let non_fungible_id =
        extract_non_fungible_id_from_simple_representation(&request.non_fungible_id)
            .map_err(|err| err.into_response_error("non_fungible_id"))?;

    if non_fungible_id.id_type() != non_fungible_id_type {
        return Err(ExtractionError::WrongNonFungibleIdType {
            expected: non_fungible_id_type,
            actual: non_fungible_id.id_type(),
        }
        .into_response_error("non_fungible_id"));
    }

    let non_fungible_substate_offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
        scrypto_encode(&non_fungible_id).unwrap(),
    ));

    let substate_id = SubstateId(
        RENodeId::KeyValueStore(resource_manager.non_fungible_table),
        NodeModuleId::SELF,
        non_fungible_substate_offset,
    );

    let key_value_store_entry_substate = {
        let loaded_substate = read_optional_substate_from_id(state_manager.deref(), &substate_id);
        match loaded_substate {
            Some(PersistedSubstate::KeyValueStoreEntry(substate)) => substate,
            None => {
                return Err(not_found_error(
                    "The specified non-fungible id does not exist under that non-fungible resource",
                ))
            }
            _ => return Err(wrong_substate_type(substate_id.2)),
        }
    };

    Ok(StateNonFungibleResponse {
        non_fungible: Some(to_api_key_value_story_entry_substate(
            &mapping_context,
            &substate_id,
            &key_value_store_entry_substate,
        )?),
    })
    .map(Json)
}
