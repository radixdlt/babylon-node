use crate::core_api::*;

use state_manager::jni::state_manager::ActualStateManager;

use state_manager::transaction::UserTransactionValidator;
use state_manager::MempoolAddError;
use transaction::model::NotarizedTransaction;

pub(crate) async fn handle_transaction_submit(
    state: Extension<CoreApiState>,
    request: Json<models::TransactionSubmitRequest>,
) -> Result<Json<models::TransactionSubmitResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_transaction_submit_internal)
}

fn handle_transaction_submit_internal(
    state_manager: &mut ActualStateManager,
    request: models::TransactionSubmitRequest,
) -> Result<models::TransactionSubmitResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let notarized_transaction = extract_unvalidated_transaction(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager.check_for_rejection_and_add_to_mempool(notarized_transaction);

    match result {
        Ok(_) => Ok(models::TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::Duplicate) => Ok(models::TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Full {
            current_size: _,
            max_size: _,
        }) => Err(client_error("Mempool is full")),
        Err(MempoolAddError::Rejected(reason)) => {
            Err(client_error(&format!("Rejected: {}", reason)))
        }
    }
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
