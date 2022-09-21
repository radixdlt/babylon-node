use crate::core_api::*;
use radix_engine::engine::Substate;
use radix_engine::types::SubstateId;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_v0_state_epoch(
    state: Extension<CoreApiState>,
) -> Result<Json<models::V0StateEpochResponse>, RequestHandlingError> {
    core_api_handler(state, Json(()), handle_v0_state_epoch_internal)
}

fn handle_v0_state_epoch_internal(
    state_manager: &mut ActualStateManager,
    _request: (),
) -> Result<models::V0StateEpochResponse, RequestHandlingError> {
    if let Some(output_value) = state_manager.store.get_substate(&SubstateId::System) {
        if let Substate::System(system) = output_value.substate {
            return Ok(models::V0StateEpochResponse {
                epoch: to_api_epoch(system.epoch)?,
            });
        }
    }
    Err(MappingError::MismatchedSubstateId {
        message: "System Substate not found".to_owned(),
    }
    .into())
}
