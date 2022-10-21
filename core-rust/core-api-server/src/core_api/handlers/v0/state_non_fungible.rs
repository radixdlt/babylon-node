use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{Bech32Decoder, Bech32Encoder, SubstateId};
use scrypto::abi::ScryptoType::Hash;
use scrypto::engine::types::{
    NonFungibleStoreOffset, RENodeId, ResourceManagerOffset, SubstateOffset,
};
use scrypto::prelude::hash;
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
) -> Result<models::V0StateNonFungibleResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);
    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let resource_address = extract_resource_address(&bech32_decoder, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let non_fungible_id = extract_non_fungible_id(&request.non_fungible_id_hex)
        .map_err(|err| err.into_response_error("non_fungible_id"))?;

    let resource_manager_substate_id = SubstateId(
        RENodeId::ResourceManager((hash(vec![]), 0)), // TODO: fixme
        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
    );

    if let Some(output_value) = state_manager
        .store
        .get_substate(&resource_manager_substate_id)
    {
        if let PersistedSubstate::ResourceManager(resource_manager_substate) = output_value.substate
        {
            if let Some(non_fungible_store_id) = resource_manager_substate.nf_store_id {
                let non_fungible_substate_id = SubstateId(
                    RENodeId::NonFungibleStore(non_fungible_store_id),
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
                        non_fungible_id,
                    )),
                );
                if let Some(output_value) =
                    state_manager.store.get_substate(&non_fungible_substate_id)
                {
                    if let PersistedSubstate::NonFungible(non_fungible) = output_value.substate {
                        return Ok(models::V0StateNonFungibleResponse {
                            non_fungible: Some(to_api_non_fungible_substate(
                                &bech32_encoder,
                                &non_fungible_substate_id,
                                &non_fungible,
                            )?),
                        });
                    }
                }
            }
        }
        return Err(MappingError::MismatchedSubstateId {
            message: "Non-fungible substate was not of the right type".to_owned(),
        }
        .into());
    }

    Err(not_found_error("Non-fungible not found"))
}
