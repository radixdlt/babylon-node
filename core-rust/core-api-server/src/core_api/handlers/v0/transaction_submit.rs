use crate::core_api::handlers::extract_unvalidated_transaction;
use crate::core_api::*;

use state_manager::jni::state_manager::ActualStateManager;

use state_manager::MempoolAddError;

pub(crate) async fn handle_v0_transaction_submit(
    state: Extension<CoreApiState>,
    request: Json<models::V0TransactionSubmitRequest>,
) -> Result<Json<models::V0TransactionSubmitResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_submit_internal)
}

#[tracing::instrument(level = "debug", skip(state_manager), err(Debug))]
fn handle_v0_transaction_submit_internal(
    state_manager: &mut ActualStateManager,
    request: models::V0TransactionSubmitRequest,
) -> Result<models::V0TransactionSubmitResponse, RequestHandlingError> {
    let transaction = extract_unvalidated_transaction(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager.check_for_rejection_and_add_to_mempool_from_core_api(transaction);

    match result {
        Ok(_) => Ok(models::V0TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::Duplicate) => Ok(models::V0TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Full {
            current_size: _,
            max_size: _,
        }) => Err(client_error("Mempool is full")),
        Err(MempoolAddError::Rejected(reason)) => {
            Err(client_error(&format!("Rejected: {}", reason)))
        }
    }
}
