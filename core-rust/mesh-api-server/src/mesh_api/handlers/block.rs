use crate::prelude::*;

pub(crate) async fn handle_block(
    state: State<MeshApiState>,
    Json(request): Json<models::BlockRequest>,
) -> Result<Json<models::BlockResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.access_direct();
    let mapping_context = MappingContext::new(&state.network);

    let state_version = extract_state_version_from_mesh_api_partial_block_identifier(
        database.deref(),
        &request.block_identifier,
    )
    .map_err(|err| err.into_response_error("block_identifier"))?
    .unwrap_or_else(|| database.max_state_version());

    let previous_state_version = state_version.previous().map_err(|_| {
        ResponseError::from(ApiError::ParentBlockNotAvailable).with_details(format!(
            "Parent block not found for state version {}",
            state_version.number()
        ))
    })?;

    let transaction_identifiers = database
        .get_committed_transaction_identifiers(state_version)
        .ok_or_else(|| {
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "Failed fetching transaction identifiers for state version {}",
                state_version.number()
            ))
        })?;
    let previous_transaction_identifiers = database
        .get_committed_transaction_identifiers(previous_state_version)
        .ok_or_else(|| {
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "Failed fetching transaction identifiers for state version {}",
                previous_state_version.number()
            ))
        })?;

    let operations = to_mesh_api_operations(&mapping_context, database.deref(), state_version)?;

    let transaction_identifier =
        to_mesh_api_transaction_identifier(&mapping_context, &transaction_identifiers)?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#transaction
    let transaction = models::Transaction {
        transaction_identifier: Box::new(transaction_identifier),
        operations,
        related_transactions: None,
        metadata: None,
    };

    // see https://docs.cdp.coinbase.com/mesh/docs/models#block
    let block = models::Block {
        block_identifier: Box::new(to_mesh_api_block_identifier_from_state_version(
            state_version,
            &transaction_identifiers
                .resultant_ledger_hashes
                .transaction_root,
            &transaction_identifiers.resultant_ledger_hashes.receipt_root,
        )?),
        parent_block_identifier: Box::new(to_mesh_api_block_identifier_from_state_version(
            previous_state_version,
            &previous_transaction_identifiers
                .resultant_ledger_hashes
                .transaction_root,
            &previous_transaction_identifiers
                .resultant_ledger_hashes
                .receipt_root,
        )?),
        timestamp: transaction_identifiers.proposer_timestamp_ms,
        transactions: vec![transaction],
        metadata: None,
    };

    // see https://docs.cdp.coinbase.com/mesh/docs/models#blockresponse
    Ok(Json(models::BlockResponse {
        block: Some(Box::new(block)),
        other_transactions: None,
    }))
}
