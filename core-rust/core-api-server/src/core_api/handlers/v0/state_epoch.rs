use crate::core_api::*;
use radix_engine::model::PersistedSubstate;

use scrypto::constants::SYS_SYSTEM_COMPONENT;
use scrypto::engine::types::{GlobalAddress, SubstateOffset, SystemOffset};
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_v0_state_epoch(
    state: Extension<CoreApiState>,
) -> Result<Json<models::V0StateEpochResponse>, RequestHandlingError> {
    core_api_read_handler(state, Json(()), handle_v0_state_epoch_internal)
}

fn handle_v0_state_epoch_internal(
    state_manager: &ActualStateManager,
    _request: (),
) -> Result<models::V0StateEpochResponse, RequestHandlingError> {
    match read_derefed_global_substate(
        state_manager,
        GlobalAddress::Component(SYS_SYSTEM_COMPONENT),
        SubstateOffset::System(SystemOffset::System),
    )? {
        Some(PersistedSubstate::System(system)) => Ok(models::V0StateEpochResponse {
            epoch: to_api_epoch(system.epoch)?,
        }),
        _ => Err(MappingError::MismatchedSubstateId {
            message: "System substate not found".to_owned(),
        }
        .into()),
    }
}
