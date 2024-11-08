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

    let (operations, transaction_identifier) = get_operations_and_transaction_identifier(
        &mapping_context,
        database.deref(),
        state_version,
        &transaction_identifiers,
        None,
    )?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#transaction
    let transaction = models::Transaction {
        transaction_identifier: Box::new(models::TransactionIdentifier::new(
            transaction_identifier,
        )),
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

    let (operations, transaction_identifier) = get_operations_and_transaction_identifier(
        &mapping_context,
        database.deref(),
        state_version,
        &transaction_identifiers,
        Some(&request.transaction_identifier.hash),
    )?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#transaction
    let transaction = models::Transaction {
        transaction_identifier: Box::new(models::TransactionIdentifier::new(
            transaction_identifier,
        )),
        operations,
        related_transactions: None,
        metadata: None,
    };

    // see https://docs.cdp.coinbase.com/mesh/docs/models#blocktransactionresponse
    Ok(Json(models::BlockTransactionResponse {
        transaction: Box::new(transaction),
    }))
}

fn get_operations_and_transaction_identifier(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
    state_version: StateVersion,
    transaction_identifiers: &CommittedTransactionIdentifiers,
    requested_transaction_identifier: Option<&str>,
) -> Result<(Vec<models::Operation>, String), ResponseError> {
    let (operations, transaction_identifier) = match transaction_identifiers
        .transaction_hashes
        .as_user()
    {
        // In case of non-user transaction we return empty transaction,
        // otherwise mesh-cli returns error.
        // Unfortunately non-user transactions don't have txid,
        // let's use state_version as transaction_identifier.
        None => {
            let transaction_identifier = format!("state_version_{}", state_version);
            if requested_transaction_identifier.is_some_and(|tx_id| tx_id != transaction_identifier)
            {
                return Err(ResponseError::from(ApiError::InvalidRequest)
                    .with_details("transaction_identifier does not match with block_identifier"));
            }

            (vec![], transaction_identifier)
        }
        Some(user_hashes) => {
            let local_transaction_execution = database
                .get_committed_local_transaction_execution(state_version)
                .ok_or_else(|| {
                    ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                        "Failed fetching transaction execution for state version {}",
                        state_version.number()
                    ))
                })?;

            let transaction_identifier =
                to_api_hash_bech32m(mapping_context, &user_hashes.transaction_intent_hash)?;

            if requested_transaction_identifier.is_some_and(|tx_id| tx_id != transaction_identifier)
            {
                return Err(ResponseError::from(ApiError::InvalidRequest)
                    .with_details("transaction_identifier does not match with block_identifier"));
            }

            let status = MeshApiOperationStatus::from(local_transaction_execution.outcome);

            let mut index = 0_i64;
            let mut operations = vec![];
            for (address, balance_changes) in local_transaction_execution
                .global_balance_summary
                .global_balance_changes
            {
                if address.is_account() {
                    for (resource_address, balance_change) in balance_changes {
                        if let BalanceChange::Fungible(amount) = balance_change {
                            operations.push(to_mesh_api_operation(
                                mapping_context,
                                database,
                                index,
                                status.clone(),
                                &address,
                                &resource_address,
                                amount,
                            )?);
                            index += 1;
                        }
                    }
                }
            }
            (operations, transaction_identifier)
        }
    };
    Ok((operations, transaction_identifier))
}
