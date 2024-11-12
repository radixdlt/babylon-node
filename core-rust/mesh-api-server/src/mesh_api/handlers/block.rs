use models::transaction_identifier;

use crate::prelude::*;

pub(crate) async fn handle_block(
    state: State<MeshApiState>,
    Json(request): Json<models::BlockRequest>,
) -> Result<Json<models::BlockResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();
    let mapping_context = MappingContext::new(&state.network);

    let state_version =
        extract_state_version_from_mesh_api_partial_block_identifier(&request.block_identifier)
            .map_err(|err| err.into_response_error("block_identifier"))?
            .unwrap_or_else(|| database.max_state_version());

    let previous_state_version =
        state_version
            .previous()
            .map_err(|_| MappingError::IntegerError {
                message: "Error getting parent block".to_string(),
            })?;

    let transaction_identifiers = database
        .get_committed_transaction_identifiers(state_version)
        .ok_or_else(|| {
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "Failed fetching transaction identifiers for state version {}",
                state_version.number()
            ))
        })?;

    let operations = to_mesh_api_operations(&mapping_context, database.deref(), state_version)?;

    let transaction_identifier = to_mesh_api_transaction_identifier(
        &mapping_context,
        &transaction_identifiers,
        state_version,
    )?;

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
        )?),
        parent_block_identifier: Box::new(to_mesh_api_block_identifier_from_state_version(
            previous_state_version,
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

pub(crate) async fn handle_block_transaction(
    state: State<MeshApiState>,
    Json(request): Json<models::BlockTransactionRequest>,
) -> Result<Json<models::BlockTransactionResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();
    let mapping_context = MappingContext::new(&state.network);

    let state_version =
        extract_state_version_from_mesh_api_block_identifier(&request.block_identifier)
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
        return Err(MappingError::InvalidTransactionIdentifier {
            message: format!("transaction_identifier does not match with block_identifier"),
        }
        .into());
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
