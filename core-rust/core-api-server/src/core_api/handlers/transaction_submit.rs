use crate::core_api::*;

use hyper::StatusCode;
use models::transaction_submit_error_details::TransactionSubmitErrorDetails;
use state_manager::MempoolAddError;

#[tracing::instrument(level = "debug", skip(state), err(Debug))]
pub(crate) async fn handle_transaction_submit(
    State(state): State<CoreApiState>,
    Json(request): Json<models::TransactionSubmitRequest>,
) -> Result<Json<models::TransactionSubmitResponse>, ResponseError<TransactionSubmitErrorDetails>> {
    let mut state_manager = state.state_manager.write();

    let mapping_context = MappingContext::new_for_uncommitted_data(&state_manager.network);

    assert_matching_network(&request.network, &state_manager.network)?;

    let notarized_transaction = extract_unvalidated_transaction(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

    let result = state_manager.add_to_mempool_and_trigger_relay(notarized_transaction);

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
                // TODO - Add `result_validity_substate_criteria` once track / mempool is improved
                retry_from_timestamp: match rejection.retry_from {
                    state_manager::RetryFrom::Never => None,
                    state_manager::RetryFrom::FromTime(time) => Some(Box::new(
                        to_api_instant_from_safe_timestamp(to_unix_timestamp_ms(time)?)?,
                    )),
                    state_manager::RetryFrom::FromEpoch(_) => None,
                    state_manager::RetryFrom::Whenever => {
                        Some(Box::new(to_api_instant_from_safe_timestamp(
                            to_unix_timestamp_ms(std::time::SystemTime::now())?,
                        )?))
                    }
                },
                retry_from_epoch: match rejection.retry_from {
                    state_manager::RetryFrom::FromEpoch(epoch) => {
                        Some(to_api_epoch(&mapping_context, epoch)?)
                    }
                    _ => None,
                },
                invalid_from_epoch: if rejection.is_permanent_for_payload() {
                    None
                } else {
                    Some(to_api_epoch(
                        &mapping_context,
                        rejection.invalid_from_epoch,
                    )?)
                },
            },
        )),
    }
    .map(Json)
}
