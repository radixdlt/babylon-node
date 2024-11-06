use crate::prelude::*;

pub(crate) async fn handle_block(
    state: State<MeshApiState>,
    Json(request): Json<models::BlockRequest>,
) -> Result<Json<models::BlockResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();

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
                "No transaction at given index {}",
                state_version.number()
            ))
        })?;

    let (operations, transaction_identifier) =
        match transaction_identifiers.transaction_hashes.as_user() {
            // In case of non-user transaction we return empty transaction,
            // otherwise mesh-cli returns error.
            // Unfortunately non-user transactions don't have txid,
            // let's use state_version as transaction_identifier.
            None => (vec![], format!("state_version_{})", state_version)),
            Some(h) => {
                let local_transaction_execution = database
                    .get_committed_local_transaction_execution(state_version)
                    .ok_or_else(|| {
                        ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                            "Local transaction execution not available {}",
                            state_version.number()
                        ))
                    })?;

                let mut operations = vec![];

                let mapping_context = MappingContext::new(&state.network);

                let transaction_identifier =
                    to_api_hash_bech32m(&mapping_context, &h.transaction_intent_hash)?;
                let mut index = 0_i64;
                for (address, balance_changes) in local_transaction_execution
                    .global_balance_summary
                    .global_balance_changes
                {
                    let api_address =
                        to_api_entity_address(&mapping_context, address.as_node_id())?;
                    println!(
                        "address {} {} {} balance_changes = {:#?}",
                        api_address,
                        address.is_account(),
                        address.as_node_id().is_global_account(),
                        balance_changes
                    );
                    if address.is_account() {
                        for (resource_address, balance_change) in balance_changes {
                            if let BalanceChange::Fungible(amount) = balance_change {
                                operations.push(to_mesh_api_operation(
                                    &mapping_context,
                                    database.deref(),
                                    index,
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

    Ok(Json(models::BlockResponse {
        block: Some(Box::new(block)),
        other_transactions: None,
    }))
}

fn to_mesh_api_amount(
    amount: Decimal,
    currency: models::Currency,
) -> Result<models::Amount, MappingError> {
    let value = amount
        / Decimal::TEN
            .checked_powi((Decimal::SCALE as i32 - currency.decimals) as i64)
            .ok_or_else(|| MappingError::IntegerError {
                message: "Integer overflow".to_string(),
            })?;

    Ok(models::Amount::new(value.attos().to_string(), currency))
}

fn to_mesh_api_operation(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
    index: i64,
    account_address: &GlobalAddress,
    resource_address: &ResourceAddress,
    amount: Decimal,
) -> Result<models::Operation, MappingError> {
    // TODO:MESH what about fee locking, burning, minting?
    let op_type = if amount.is_positive() {
        OperationTypes::Deposit
    } else {
        OperationTypes::Withdraw
    };

    let currency =
        to_mesh_api_currency_from_resource_address(mapping_context, database, resource_address)?;

    let account = to_mesh_api_acount_from_address(mapping_context, account_address)?;
    Ok(models::Operation {
        operation_identifier: Box::new(models::OperationIdentifier::new(index)),
        related_operations: None,
        _type: op_type.to_string(),
        status: Some("Success".to_string()),
        account: Some(Box::new(account)),
        amount: Some(Box::new(to_mesh_api_amount(amount, currency)?)),
        coin_change: None,
        metadata: None,
    })
}
