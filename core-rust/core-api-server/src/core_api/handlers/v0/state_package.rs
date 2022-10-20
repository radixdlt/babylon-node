use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{Bech32Decoder, SubstateId};
use scrypto::engine::types::{PackageOffset, RENodeId, SubstateOffset};
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

    let substate_id = SubstateId(
        RENodeId::Package(package_address),
        SubstateOffset::Package(PackageOffset::Package),
    );

    if let Some(output_value) = state_manager.store.get_substate(&substate_id) {
        if let PersistedSubstate::Package(package) = output_value.substate {
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
