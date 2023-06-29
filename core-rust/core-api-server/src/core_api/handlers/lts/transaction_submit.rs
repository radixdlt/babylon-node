use crate::core_api::*;

use hyper::StatusCode;
use models::lts_transaction_submit_error_details::LtsTransactionSubmitErrorDetails;
use state_manager::{MempoolAddError, MempoolAddSource};
use transaction::prelude::RawNotarizedTransaction;

#[tracing::instrument(level = "debug", skip(state))]
pub(crate) async fn handle_lts_transaction_submit(
    State(state): State<CoreApiState>,
    Json(request): Json<models::LtsTransactionSubmitRequest>,
) -> Result<
    Json<models::LtsTransactionSubmitResponse>,
    ResponseError<LtsTransactionSubmitErrorDetails>,
> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new_for_uncommitted_data(&state.network);

    let transaction_bytes = from_hex(&request.notarized_transaction_hex)
        .map_err(|err| err.into_response_error("notarized_transaction_hex"))?;

    let force_recalculate = request.force_recalculate.unwrap_or(false);

    let result = state.mempool_manager.add_and_trigger_relay(
        MempoolAddSource::CoreApi,
        RawNotarizedTransaction(transaction_bytes),
        force_recalculate,
    );

    match result {
        Ok(_) => Ok(models::LtsTransactionSubmitResponse::new(false)),
        Err(MempoolAddError::PriorityThresholdNotMet { min_tip_percentage_required, tip_percentage }) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "Mempool is full and priority threshold not met",
            LtsTransactionSubmitErrorDetails::LtsTransactionSubmitPriorityThresholdNotMetErrorDetails {
                tip_percentage: tip_percentage as i32,
                min_tip_percentage_required: min_tip_percentage_required as i32,
            },
        )),
        Err(MempoolAddError::Duplicate(_)) => Ok(models::LtsTransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Rejected(rejection)) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "Transaction was rejected",
            LtsTransactionSubmitErrorDetails::LtsTransactionSubmitRejectedErrorDetails {
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
