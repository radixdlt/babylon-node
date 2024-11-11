use crate::prelude::*;

use crate::core_api::handlers::to_api_committed_intent_metadata;
use models::lts_transaction_submit_error_details::LtsTransactionSubmitErrorDetails;

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

    let result = state.state_manager.mempool_manager.add_and_trigger_relay(
        MempoolAddSource::CoreApi,
        RawNotarizedTransaction::from_vec(transaction_bytes),
        force_recalculate,
    );

    match result {
        Ok(_) => Ok(models::LtsTransactionSubmitResponse::new(false)),
        Err(MempoolAddError::PriorityThresholdNotMet {
            min_tip_basis_points_required,
            tip_basis_points,
        }) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "The mempool is full and the submitted transaction's priority is not sufficient to replace any existing transactions. Try submitting with a larger tip to increase the transaction's priority.",
            LtsTransactionSubmitErrorDetails::LtsTransactionSubmitPriorityThresholdNotMetErrorDetails {
                tip_percentage: TipSpecifier::BasisPoints(tip_basis_points).truncate_to_percentage_u32() as i32, // Dividing by 100 means this is inbounds
                min_tip_percentage_required: min_tip_basis_points_required.map(|x| TipSpecifier::BasisPoints(x).truncate_to_percentage_u32() as i32),
            },
        )),
        Err(MempoolAddError::Duplicate(_)) => Ok(models::LtsTransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Rejected(rejection, notarized_transaction_hash)) => {
            if let Some(already_committed_error) = rejection.transaction_intent_already_committed_error() {
                let is_same_transaction = Some(already_committed_error.committed_notarized_transaction_hash) == notarized_transaction_hash;
                Err(detailed_error(
                    StatusCode::BAD_REQUEST,
                    "The transaction intent has already been committed",
                    LtsTransactionSubmitErrorDetails::LtsTransactionSubmitIntentAlreadyCommitted {
                        committed_as: Box::new(to_api_committed_intent_metadata(&mapping_context, already_committed_error, is_same_transaction)?)
                    }
                ))
            } else {
                Err(detailed_error(
                    StatusCode::BAD_REQUEST,
                    "Transaction was rejected",
                    LtsTransactionSubmitErrorDetails::LtsTransactionSubmitRejectedErrorDetails {
                        error_message: format!("{}", rejection.reason),
                        is_fresh: !rejection.was_cached,
                        is_payload_rejection_permanent: rejection.is_permanent_for_payload(),
                        is_intent_rejection_permanent: rejection.is_permanent_for_intent(),
                        // TODO - Add `result_validity_substate_criteria` once track / mempool is improved
                        retry_from_timestamp: match rejection.retry_from {
                            RetryFrom::Never => None,
                            RetryFrom::FromTime(time) => Some(Box::new(
                                to_api_clamped_instant_from_epoch_milli(to_unix_timestamp_ms(time)?),
                            )),
                            RetryFrom::FromEpoch(_) => None,
                            RetryFrom::Whenever => {
                                Some(Box::new(to_api_clamped_instant_from_epoch_milli(
                                    to_unix_timestamp_ms(std::time::SystemTime::now())?,
                                )))
                            }
                        },
                        retry_from_epoch: match rejection.retry_from {
                            RetryFrom::FromEpoch(epoch) => {
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
                ))
            }
        }
    }
    .map(Json)
}
