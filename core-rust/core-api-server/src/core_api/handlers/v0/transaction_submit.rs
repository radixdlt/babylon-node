use state_manager::{MempoolAddError, MempoolAddSource};

use crate::core_api::handlers::extract_unvalidated_transaction;
use crate::core_api::*;

#[tracing::instrument(level = "debug", skip(state), err(Debug))]
pub(crate) async fn handle_v0_transaction_submit(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::V0TransactionSubmitRequest>,
) -> Result<Json<models::V0TransactionSubmitResponse>, RequestHandlingError> {
    let mut state_manager = state.state_manager.write();

    let transaction = extract_unvalidated_transaction(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager.check_for_rejection_and_add_to_mempool(MempoolAddSource::CoreApi, transaction);

    match result {
        Ok(_) => Ok(models::V0TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::Duplicate) => Ok(models::V0TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Full {
            current_size: _,
            max_size: _,
        }) => Err(client_error("Mempool is full")),
        Err(MempoolAddError::Rejected(rejection)) => {
            Err(client_error(&format!("Rejected: {}", rejection.reason)))
        }
    }
    .map(Json)
}
