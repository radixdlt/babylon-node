use crate::core_api::models::*;
use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_v0_state_epoch(
    state: Extension<CoreApiState>,
) -> Result<Json<V0StateEpochResponse>, RequestHandlingError> {
    core_api_handler(state, Json(()), handle_v0_state_epoch_internal)
}

fn handle_v0_state_epoch_internal(
    state_manager: &mut ActualStateManager,
    request: (),
) -> Result<V0StateEpochResponse, RequestHandlingError> {
    todo!()
}
