use crate::core_api::models::*;
use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_v0_transaction_receipt(
    state: Extension<CoreApiState>,
    request: Json<V0CommittedTransactionRequest>,
) -> Result<Json<V0CommittedTransactionResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_receipt_internal)
}

fn handle_v0_transaction_receipt_internal(
    state_manager: &mut ActualStateManager,
    request: V0CommittedTransactionRequest,
) -> Result<V0CommittedTransactionResponse, RequestHandlingError> {
    todo!()
}
