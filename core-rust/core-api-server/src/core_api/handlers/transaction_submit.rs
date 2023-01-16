use crate::core_api::*;

use state_manager::transaction::UserTransactionValidator;
use state_manager::{MempoolAddError, MempoolAddSource};
use transaction::model::NotarizedTransaction;

#[tracing::instrument(level = "debug", skip(state), err(Debug))]
pub(crate) async fn handle_transaction_submit(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::TransactionSubmitRequest>,
) -> Result<Json<models::TransactionSubmitResponse>, RequestHandlingError> {
    let mut state_manager = state.state_manager.write();

    assert_matching_network(&request.network, &state_manager.network)?;

    let notarized_transaction = extract_unvalidated_transaction(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager
        .check_for_rejection_and_add_to_mempool(MempoolAddSource::CoreApi, notarized_transaction);

    match result {
        Ok(_) => Ok(models::TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::Duplicate) => Ok(models::TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Full {
            current_size: _,
            max_size: _,
        }) => Err(client_error("Mempool is full")),
        Err(MempoolAddError::Rejected(rejection)) => {
            Err(client_error(format!("Rejected: {}", rejection.reason)))
        }
    }
    .map(Json)
}

pub fn extract_unvalidated_transaction(
    payload: &str,
) -> Result<NotarizedTransaction, ExtractionError> {
    let transaction_bytes = from_hex(payload)?;
    let notarized_transaction =
        UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
            &transaction_bytes,
        )?;
    Ok(notarized_transaction)
}
