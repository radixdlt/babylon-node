use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::StateManagerSubstateQueries;

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
    let epoch = state_manager.store.get_epoch();
    Ok(models::V0StateEpochResponse {
        epoch: to_api_epoch(epoch)?,
    })
}
