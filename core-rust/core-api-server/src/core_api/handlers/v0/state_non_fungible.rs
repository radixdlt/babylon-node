use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{
    Bech32Decoder, Bech32Encoder, GlobalAddress, NonFungibleStoreOffset, RENodeId,
    ResourceManagerOffset, SubstateId, SubstateOffset,
};

use crate::core_api::models::V0StateNonFungibleResponse;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_v0_state_non_fungible(
    state: Extension<CoreApiState>,
    request: Json<models::V0StateNonFungibleRequest>,
) -> Result<Json<models::V0StateNonFungibleResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_v0_state_non_fungible_internal)
}

fn handle_v0_state_non_fungible_internal(
    state_manager: &ActualStateManager,
    request: models::V0StateNonFungibleRequest,
) -> Result<V0StateNonFungibleResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);
    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let resource_address = extract_resource_address(&bech32_decoder, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let non_fungible_id = extract_non_fungible_id(&request.non_fungible_id_hex)
        .map_err(|err| err.into_response_error("non_fungible_id"))?;

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
        let loaded_substate =
            read_known_substate(state_manager, non_fungible_node_id, &substate_offset)?;
        let PersistedSubstate::NonFungible(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let non_fungible_substate_id = SubstateId(non_fungible_node_id, non_fungible_substate_offset);

    Ok(V0StateNonFungibleResponse {
        non_fungible: Some(to_api_non_fungible_substate(
            &bech32_encoder,
            &non_fungible_substate_id,
            &non_fungible,
        )?),
    })
}
