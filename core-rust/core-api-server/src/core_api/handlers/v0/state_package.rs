use crate::core_api::*;
use radix_engine::engine::Substate;
use radix_engine::types::{Bech32Decoder, SubstateId};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

pub(crate) async fn handle_v0_state_package(
    state: Extension<CoreApiState>,
    request: Json<models::V0StatePackageRequest>,
) -> Result<Json<models::V0StatePackageResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_v0_state_package_internal)
}

fn handle_v0_state_package_internal(
    state_manager: &ActualStateManager,
    request: models::V0StatePackageRequest,
) -> Result<models::V0StatePackageResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);

    let package_address = extract_package_address(&bech32_decoder, &request.package_address)
        .map_err(|err| err.into_response_error("package_address"))?;

    if let Some(output_value) = state_manager
        .store
        .get_substate(&SubstateId::Package(package_address))
    {
        if let Substate::Package(package) = output_value.substate {
            return Ok(models::V0StatePackageResponse {
                package: Some(to_api_package_substate(&package)),
            });
        }
        return Err(MappingError::MismatchedSubstateId {
            message: "Package Substate was not of the right type".to_owned(),
        }
        .into());
    }
    Err(not_found_error("Package not found"))
}
