use crate::prelude::*;

pub(crate) async fn handle_block_transaction(
    state: State<MeshApiState>,
    Json(request): Json<models::BlockTransactionRequest>,
) -> Result<Json<models::BlockTransactionResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();

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
            if !request
                .transaction_identifier
                .hash
                .eq(&transaction_identifier)
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

            let mapping_context = MappingContext::new(&state.network);

            let transaction_identifier =
                to_api_hash_bech32m(&mapping_context, &user_hashes.transaction_intent_hash)?;

            if !request
                .transaction_identifier
                .hash
                .eq(&transaction_identifier)
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
                                &mapping_context,
                                database.deref(),
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
