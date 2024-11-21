use crate::prelude::*;

pub(crate) async fn handle_construction_submit(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionSubmitRequest>,
) -> Result<Json<models::TransactionIdentifierResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let (raw, intent_hash) = RawNotarizedTransaction::from_hex(&request.signed_transaction)
        .ok()
        .and_then(|raw| {
            let tx = raw.prepare(PreparationSettings::latest_ref());
            tx.map(|tx| (raw, tx)).ok()
        })
        .and_then(|(raw, tx)| Some((raw, tx.hashes().transaction_intent_hash)))
        .ok_or(
            ResponseError::from(ApiError::InvalidTransaction).with_details(format!(
                "Invalid transaction: {}",
                request.signed_transaction
            )),
        )?;

    let mempool_add_result = match state.state_manager.mempool_manager.add_and_trigger_relay(
        MempoolAddSource::MeshApi,
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
        return Err(ResponseError::from(ApiError::SubmitTransactionError)
            .with_details(format!(
                "Failed to submit transaction to mempool: {}",
                message
            ))
            .retryable(true));
    };
    let transaction_identifier = to_mesh_api_transaction_identifier_from_hash(
        to_api_transaction_hash_bech32m(&MappingContext::new(&state.network), &intent_hash)?,
    );

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionsubmitresponse for field
    // definitions
    Ok(Json(models::TransactionIdentifierResponse {
        transaction_identifier: Box::new(transaction_identifier),
        metadata: None,
    }))
}
