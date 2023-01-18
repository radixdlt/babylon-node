use crate::core_api::*;

use hyper::StatusCode;
use models::transaction_submit_error_details::TransactionSubmitErrorDetails;
use state_manager::transaction::UserTransactionValidator;
use state_manager::{MempoolAddError, MempoolAddSource};
use transaction::model::NotarizedTransaction;

impl ErrorDetails for TransactionSubmitErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::TransactionSubmitErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

#[tracing::instrument(level = "debug", skip(state), err(Debug))]
pub(crate) async fn handle_transaction_submit(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::TransactionSubmitRequest>,
) -> Result<Json<models::TransactionSubmitResponse>, ResponseError<TransactionSubmitErrorDetails>> {
    let mut state_manager = state.state_manager.write();

    assert_matching_network(&request.network, &state_manager.network)?;

    let notarized_transaction = extract_unvalidated_transaction(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager
        .check_for_rejection_and_add_to_mempool(MempoolAddSource::CoreApi, notarized_transaction);

    match result {
        Ok(_) => Ok(models::TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::Duplicate) => Ok(models::TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Full { max_size, .. }) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "Mempool is full",
            TransactionSubmitErrorDetails::TransactionSubmitMempoolFullErrorDetails {
                mempool_capacity: max_size as i32,
            },
        )),
        Err(MempoolAddError::Rejected(rejection)) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "Transaction was rejected",
            TransactionSubmitErrorDetails::TransactionSubmitRejectedErrorDetails {
                error_message: format!("{}", rejection.reason),
                is_fresh: !rejection.was_cached,
                is_payload_rejection_permanent: rejection.is_permanent_for_payload(),
                is_intent_rejection_permanent: rejection.is_permanent_for_intent(),
                is_rejected_because_intent_already_committed: rejection
                    .is_rejected_because_intent_already_committed(),
                recalculation_due: match rejection.recalculation_due {
                    state_manager::RecalculationDue::Never => None,
                    state_manager::RecalculationDue::From(time) => {
                        Some(to_api_unix_timestamp_ms(time)?)
                    }
                    state_manager::RecalculationDue::Whenever => {
                        Some(to_api_unix_timestamp_ms(std::time::SystemTime::now())?)
                    }
                },
                invalid_from_epoch: if rejection.is_permanent_for_payload() {
                    None
                } else {
                    // If it's a temporary rejection, then the transaction must have passed validation...
                    // So the epoch quoted below must be in a valid range to pass the `to_api_epoch` mapping
                    Some(to_api_epoch(rejection.invalid_from_epoch)?)
                },
            },
        )),
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
