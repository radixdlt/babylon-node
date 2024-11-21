use crate::prelude::*;

pub(crate) async fn handle_block_transaction(
    state: State<MeshApiState>,
    Json(request): Json<models::BlockTransactionRequest>,
) -> Result<Json<models::BlockTransactionResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();
    let mapping_context = MappingContext::new(&state.network);

    let state_version = extract_state_version_from_mesh_api_block_identifier(
        database.deref(),
        &request.block_identifier,
    )
    .map_err(|err| err.into_response_error("block_identifier"))?;

    let transaction_identifiers = database
        .get_committed_transaction_identifiers(state_version)
        .ok_or_else(|| {
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "Failed fetching transaction identifiers for state version {}",
                state_version.number()
            ))
        })?;

    let transaction_identifier = to_mesh_api_transaction_identifier(
        &mapping_context,
        &transaction_identifiers,
        state_version,
    )?;
    if !request
        .transaction_identifier
        .as_ref()
        .eq(&transaction_identifier)
    {
        return Err(
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "transaction_identifier {} not available in given block",
                request.transaction_identifier.hash
            )),
        );
    }

    let operations = to_mesh_api_operations(&mapping_context, database.deref(), state_version)?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#transaction
    let transaction = models::Transaction {
        transaction_identifier: Box::new(transaction_identifier),
        operations,
        related_transactions: None,
        metadata: None,
    };

    // see https://docs.cdp.coinbase.com/mesh/docs/models#blocktransactionresponse
    Ok(Json(models::BlockTransactionResponse {
        transaction: Box::new(transaction),
    }))
}
