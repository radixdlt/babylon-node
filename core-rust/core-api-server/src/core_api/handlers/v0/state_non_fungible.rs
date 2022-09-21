use crate::core_api::*;
use radix_engine::engine::Substate;
use radix_engine::types::{Bech32Decoder, SubstateId};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

pub(crate) async fn handle_v0_state_non_fungible(
    state: Extension<CoreApiState>,
    request: Json<models::V0StateNonFungibleRequest>,
) -> Result<Json<models::V0StateNonFungibleResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_state_non_fungible_internal)
}

fn handle_v0_state_non_fungible_internal(
    state_manager: &mut ActualStateManager,
    request: models::V0StateNonFungibleRequest,
) -> Result<models::V0StateNonFungibleResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);

    let resource_address = extract_resource_address(&bech32_decoder, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let non_fungible_id = extract_non_fungible_id(&request.non_fungible_id_hex)
        .map_err(|err| err.into_response_error("non_fungible_id"))?;

    let substate_id = SubstateId::NonFungible(resource_address, non_fungible_id);

    if let Some(output_value) = state_manager.store.get_substate(&substate_id) {
        if let Substate::NonFungible(non_fungible) = output_value.substate {
            return Ok(models::V0StateNonFungibleResponse {
                non_fungible: Some(to_api_non_fungible_substate(&substate_id, &non_fungible)?),
            });
        }
        return Err(MappingError::MismatchedSubstateId {
            message: "Non-fungible substate was not of the right type".to_owned(),
        }
        .into());
    }
    Err(not_found_error("Non-fungible not found"))
}
