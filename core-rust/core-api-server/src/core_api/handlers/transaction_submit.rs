use crate::core_api::*;

use hyper::StatusCode;
use models::transaction_submit_error_details::TransactionSubmitErrorDetails;
use state_manager::{MempoolAddError, MempoolAddSource};
use transaction::prelude::*;

#[tracing::instrument(level = "debug", skip(state))]
pub(crate) async fn handle_transaction_submit(
    State(state): State<CoreApiState>,
    Json(request): Json<models::TransactionSubmitRequest>,
) -> Result<Json<models::TransactionSubmitResponse>, ResponseError<TransactionSubmitErrorDetails>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new_for_uncommitted_data(&state.network);

    let transaction_bytes = from_hex(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction_hex"))?;

    let force_recalculate = request.force_recalculate.unwrap_or(false);

    let result = state.state_manager.mempool_manager.add_and_trigger_relay(
        MempoolAddSource::CoreApi,
        RawNotarizedTransaction(transaction_bytes),
        force_recalculate,
    );

    match result {
        Ok(_) => Ok(models::TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::PriorityThresholdNotMet {
            min_tip_percentage_required,
            tip_percentage,
        }) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "The mempool is full and the submitted transaction's priority is not sufficient to replace any existing transactions. Try submitting with a larger tip to increase the transaction's priority.",
            TransactionSubmitErrorDetails::TransactionSubmitPriorityThresholdNotMetErrorDetails {
                tip_percentage: tip_percentage as i32,
                min_tip_percentage_required: min_tip_percentage_required.map(|x| x as i32),
            },
        )),
        Err(MempoolAddError::Duplicate(_)) => Ok(models::TransactionSubmitResponse::new(true)),
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
                    rejection
                        .invalid_from_epoch
                        .map(|epoch| to_api_epoch(&mapping_context, epoch))
                        .transpose()?
                },
            },
        )),
    }
    .map(Json)
}
