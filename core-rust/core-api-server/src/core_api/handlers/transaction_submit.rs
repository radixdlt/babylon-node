use crate::prelude::*;
use models::transaction_submit_error_details::TransactionSubmitErrorDetails;

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
        RawNotarizedTransaction::from_vec(transaction_bytes),
        force_recalculate,
    );

    match result {
        Ok(_) => Ok(models::TransactionSubmitResponse::new(false)),
        Err(MempoolAddError::PriorityThresholdNotMet {
            min_tip_basis_points_required,
            tip_basis_points,
        }) => Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "The mempool is full and the submitted transaction's priority is not sufficient to replace any existing transactions. Try submitting with a larger tip to increase the transaction's priority.",
            TransactionSubmitErrorDetails::TransactionSubmitPriorityThresholdNotMetErrorDetails {
                tip_percentage: TipSpecifier::BasisPoints(tip_basis_points).truncate_to_percentage_u32() as i32, // Dividing by 100 means this is inbounds
                min_tip_percentage_required: min_tip_basis_points_required.map(|x| TipSpecifier::BasisPoints(x).truncate_to_percentage_u32() as i32),
            },
        )),
        Err(MempoolAddError::Duplicate(_)) => Ok(models::TransactionSubmitResponse::new(true)),
        Err(MempoolAddError::Rejected(rejection)) => {
            if rejection.is_rejected_because_intent_already_committed() {
                let already_committed_error = rejection
                    .reason
                    .already_committed_error()
                    .expect("Already committed rejections should have an already_committed_error");
                Err(detailed_error(
                    StatusCode::BAD_REQUEST,
                    "The transaction intent has already been committed",
                    TransactionSubmitErrorDetails::TransactionSubmitIntentAlreadyCommitted {
                        committed_as: Box::new(to_api_committed_intent_metadata(&mapping_context, already_committed_error)?)
                    }
                ))
            } else {
                Err(detailed_error(
                    StatusCode::BAD_REQUEST,
                    "The transaction execution resulted in a rejection",
                    TransactionSubmitErrorDetails::TransactionSubmitRejectedErrorDetails {
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

pub fn to_api_committed_intent_metadata(
    context: &MappingContext,
    error: &AlreadyCommittedError,
) -> Result<models::CommittedIntentMetadata, MappingError> {
    Ok(models::CommittedIntentMetadata {
        state_version: to_api_state_version(error.committed_state_version)?,
        payload_hash: to_api_notarized_transaction_hash(
            &error.committed_notarized_transaction_hash,
        ),
        payload_hash_bech32m: to_api_hash_bech32m(
            context,
            &error.committed_notarized_transaction_hash,
        )?,
        is_same_transaction: error.committed_notarized_transaction_hash
            == error.notarized_transaction_hash,
    })
}
