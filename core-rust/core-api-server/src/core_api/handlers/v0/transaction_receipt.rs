use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_v0_transaction_receipt(
    state: Extension<CoreApiState>,
    request: Json<models::V0CommittedTransactionRequest>,
) -> Result<Json<models::V0CommittedTransactionResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_receipt_internal)
}

fn handle_v0_transaction_receipt_internal(
    _state_manager: &mut ActualStateManager,
    _request: models::V0CommittedTransactionRequest,
) -> Result<models::V0CommittedTransactionResponse, RequestHandlingError> {
    Err(not_found_error("API endpoint not implemented yet!"))
}
