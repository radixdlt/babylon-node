use crate::prelude::*;

pub(crate) async fn handle_mempool(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkRequest>,
) -> Result<Json<models::MempoolResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let mempool = state.state_manager.mempool.read();

    Ok(Json(models::MempoolResponse::new(
        mempool
            .all_hashes_iter()
            .map(|(intent_hash, _)| {
                Ok(models::TransactionIdentifier {
                    hash: to_api_transaction_hash_bech32m(&mapping_context, intent_hash)?,
                })
            })
            .collect::<Result<Vec<_>, MappingError>>()?,
    )))
}

pub(crate) async fn handle_mempool_transaction(
    state: State<MeshApiState>,
    Json(request): Json<models::MempoolTransactionRequest>,
) -> Result<Json<models::MempoolTransactionResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let extraction_context = ExtractionContext::new(&state.network);
    let mapping_context = MappingContext::new(&state.network);
    let mempool = state.state_manager.mempool.read();

    let intent_hash = extract_transaction_intent_hash(
        &extraction_context,
        request.transaction_identifier.hash.clone(),
    )
    .map_err(|err| err.into_response_error("intent_hash"))?;

    if mempool
        .get_notarized_transaction_hashes_for_intent(&intent_hash)
        .is_empty()
    {
        return Err(
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "transaction {} not found in mempool transactions",
                &request.transaction_identifier.hash
            )),
        );
    }

    // TODO:MESH prepare transaction estimates
    let transaction = models::Transaction::new(
        models::TransactionIdentifier::new(to_api_transaction_hash_bech32m(
            &mapping_context,
            &intent_hash,
        )?),
        vec![],
    );

    Ok(Json(models::MempoolTransactionResponse::new(transaction)))
}
