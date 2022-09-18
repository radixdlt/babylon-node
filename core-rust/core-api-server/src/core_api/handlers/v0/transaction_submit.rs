use crate::core_api::handlers::extract_notarized_transaction;
use crate::core_api::*;

use state_manager::jni::state_manager::ActualStateManager;

use state_manager::MempoolAddError;

pub(crate) async fn handle_v0_transaction_submit(
    state: Extension<CoreApiState>,
    request: Json<models::V0TransactionSubmitRequest>,
) -> Result<Json<models::V0TransactionSubmitResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_submit_internal)
}

fn handle_v0_transaction_submit_internal(
    state_manager: &mut ActualStateManager,
    request: models::V0TransactionSubmitRequest,
) -> Result<models::V0TransactionSubmitResponse, RequestHandlingError> {
    let notarized_transaction =
        extract_notarized_transaction(state_manager, &request.notarized_transaction)
            .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager.add_to_mempool(notarized_transaction.into());

    match result {
        Ok(_) => Ok(models::V0TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::Duplicate) => Ok(models::V0TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Full {
            current_size: _,
            max_size: _,
        }) => Err(client_error("Mempool is full")),
        Err(MempoolAddError::Rejected { reason }) => {
            Err(client_error(&format!("Rejected reason({})", reason)))
        }
    }
}
