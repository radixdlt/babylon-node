use crate::prelude::*;
use hyper::StatusCode;
use models::TransactionIdentifier;

pub(crate) async fn handle_construction_submit(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionSubmitRequest>,
) -> Result<Json<models::TransactionIdentifierResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let (raw, hash) = RawNotarizedTransaction::from_hex(&request.signed_transaction)
        .ok()
        .and_then(|raw| {
            raw.prepare(&PreparationSettingsV1::latest())
                .ok()
                .map(|x| (raw, x.transaction_intent_hash()))
        })
        .ok_or(client_error(
            format!(
                "Invalid signed transaction: {}",
                &request.signed_transaction
            ),
            false,
        ))?;

    let mempool_add_result = match state.state_manager.mempool_manager.add_and_trigger_relay(
        MempoolAddSource::CoreApi,
        raw,
        false,
    ) {
        Ok(_) => Ok(()),
        Err(e) => match e {
            e @ MempoolAddError::PriorityThresholdNotMet { .. } => Err(format!("{:?}", e)),
            MempoolAddError::Duplicate(_) => Ok(()),
            MempoolAddError::Rejected(rejection, _) => match rejection.reason {
                MempoolRejectionReason::SubintentAlreadyFinalized(_)
                | MempoolRejectionReason::TransactionIntentAlreadyCommitted(_) => Ok(()),
                MempoolRejectionReason::FromExecution(rejection_reason) => {
                    Err(format!("Execution failure: {:?}", rejection_reason))
                }
                MempoolRejectionReason::ValidationError(validation_error) => {
                    Err(format!("Validation failure: {:?}", validation_error))
                }
            },
        },
    };

    if let Err(message) = mempool_add_result {
        return Err(ResponseError::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            format!("Failed to submit transaction to mempool: {}", message),
            true,
        ));
    };

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionsubmitresponse for field
    // definitions
    Ok(Json(models::TransactionIdentifierResponse {
        transaction_identifier: Box::new(TransactionIdentifier {
            hash: state.hash_encoder().encode(&hash).unwrap(),
        }),
        metadata: None,
    }))
}
