use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{Bech32Decoder, Bech32Encoder, SubstateId};

use scrypto::engine::types::{
    GlobalAddress, NonFungibleStoreOffset, RENodeId, ResourceManagerOffset, SubstateOffset,
};

use crate::core_api::models::V0StateNonFungibleResponse;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

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

    match read_derefed_global_substate(
        state_manager,
        GlobalAddress::Resource(resource_address),
        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
    )? {
        Some(PersistedSubstate::ResourceManager(resource_manager)) => {
            if let Some(non_fungible_store_id) = resource_manager.nf_store_id {
                let non_fungible_substate_id = SubstateId(
                    RENodeId::NonFungibleStore(non_fungible_store_id),
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
                        non_fungible_id,
                    )),
                );

                let nft_substate = state_manager.store.get_substate(&non_fungible_substate_id);
                if let Some(PersistedSubstate::NonFungible(non_fungible)) =
                    nft_substate.map(|o| o.substate)
                {
                    Ok(V0StateNonFungibleResponse {
                        non_fungible: Some(to_api_non_fungible_substate(
                            &bech32_encoder,
                            &non_fungible_substate_id,
                            &non_fungible,
                        )?),
                    })
                } else {
                    Err(not_found_error("Non-fungible not found"))
                }
            } else {
                Err(MappingError::MismatchedSubstateId {
                    message: "Resource is not an NFT".to_owned(),
                }
                .into())
            }
        }
        Some(..) => Err(MappingError::MismatchedSubstateId {
            message: "Resource manager substate was not of the right type".to_owned(),
        }
        .into()),
        None => Err(not_found_error("Non-fungible resource not found")),
    }
}
