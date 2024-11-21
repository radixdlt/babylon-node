use crate::prelude::*;

pub(crate) async fn handle_mempool_transaction(
    state: State<MeshApiState>,
    Json(request): Json<models::MempoolTransactionRequest>,
) -> Result<Json<models::MempoolTransactionResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let extraction_context = ExtractionContext::new(&state.network);
    let mapping_context = MappingContext::new(&state.network);
    let mempool = &state.state_manager.mempool_manager;

    // Only user transactions might be present in mempool.
    // So it is safe to assume that transaction_identifier includes
    // `transaction intent_hash` and not `ledger_transaction_hash`
    let intent_hash = extract_transaction_intent_hash(
        &extraction_context,
        request.transaction_identifier.hash.clone(),
    )
    .map_err(|err| err.into_response_error("intent_hash"))?;

    if mempool
        .get_mempool_payload_hashes_for_intent(&intent_hash)
        .is_empty()
    {
        return Err(
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "transaction {} not found in mempool transactions",
                &request.transaction_identifier.hash
            )),
        );
    }

    let transaction_identifier = Box::new(models::TransactionIdentifier {
        hash: to_api_transaction_hash_bech32m(&mapping_context, &intent_hash)?,
    });

    // TODO:MESH prepare transaction estimates
    let transaction = Box::new(models::Transaction {
        transaction_identifier,
        // TODO:MESH Use the same approach as in `construction_parse`?
        operations: vec![],
        related_transactions: None,
        metadata: None,
    });

    Ok(Json(models::MempoolTransactionResponse {
        transaction,
        metadata: None,
    }))
}
