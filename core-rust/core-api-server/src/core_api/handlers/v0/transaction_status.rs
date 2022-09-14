use crate::core_api::models::*;
use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_v0_transaction_status(
    state: Extension<CoreApiState>,
    request: Json<V0TransactionStatusRequest>,
) -> Result<Json<V0TransactionStatusResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_status_internal)
}

fn handle_v0_transaction_status_internal(
    state_manager: &mut ActualStateManager,
    request: V0TransactionStatusRequest,
) -> Result<V0TransactionStatusResponse, RequestHandlingError> {
    todo!()
}
