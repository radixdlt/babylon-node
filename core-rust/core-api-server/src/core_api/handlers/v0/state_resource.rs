use crate::core_api::*;
use radix_engine::engine::Substate;
use radix_engine::types::{Bech32Decoder, SubstateId};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

pub(crate) async fn handle_v0_state_resource(
    state: Extension<CoreApiState>,
    request: Json<models::V0StateResourceRequest>,
) -> Result<Json<models::V0StateResourceResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_state_resource_internal)
}

fn handle_v0_state_resource_internal(
    state_manager: &mut ActualStateManager,
    request: models::V0StateResourceRequest,
) -> Result<models::V0StateResourceResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);

    let resource_address = extract_resource_address(&bech32_decoder, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    if let Some(output_value) = state_manager
        .store
        .get_substate(&SubstateId::ResourceManager(resource_address))
    {
        if let Substate::Resource(resource_manager) = output_value.substate {
            return Ok(models::V0StateResourceResponse {
                manager: Some(to_api_resource_substate(&resource_manager)),
            });
        }
        return Err(MappingError::MismatchedSubstateId {
            message: "Resource Substate was not of the right type".to_owned(),
        }
        .into());
    }
    Err(not_found_error("Resource not found"))
}
