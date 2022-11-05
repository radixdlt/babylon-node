use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::Bech32Decoder;
use scrypto::engine::types::{GlobalAddress, ResourceManagerOffset, SubstateOffset};

use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_v0_state_resource(
    state: Extension<CoreApiState>,
    request: Json<models::V0StateResourceRequest>,
) -> Result<Json<models::V0StateResourceResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_v0_state_resource_internal)
}

fn handle_v0_state_resource_internal(
    state_manager: &ActualStateManager,
    request: models::V0StateResourceRequest,
) -> Result<models::V0StateResourceResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);

    let resource_address = extract_resource_address(&bech32_decoder, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    match read_derefed_global_substate(
        state_manager,
        GlobalAddress::Resource(resource_address),
        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
    ) {
        Some(PersistedSubstate::ResourceManager(resource_manager)) => {
            Ok(models::V0StateResourceResponse {
                manager: Some(to_api_resource_substate(&resource_manager)),
            })
        }
        Some(..) => Err(MappingError::MismatchedSubstateId {
            message: "Resource substate was not of the right type".to_owned(),
        }
        .into()),
        None => Err(not_found_error("Resource not found")),
    }
}
