use crate::prelude::*;

pub fn to_mesh_api_operation(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
    index: i64,
    status: MeshApiOperationStatus,
    account_address: &GlobalAddress,
    resource_address: &ResourceAddress,
    amount: Decimal,
) -> Result<models::Operation, MappingError> {
    // TODO:MESH what about fee locking, burning, minting?
    let op_type = if amount.is_positive() {
        MeshApiOperationTypes::Deposit
    } else {
        MeshApiOperationTypes::Withdraw
    };
    let currency =
        to_mesh_api_currency_from_resource_address(mapping_context, database, resource_address)?;
    let account = to_mesh_api_account_from_address(mapping_context, account_address)?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#operation
    Ok(models::Operation {
        operation_identifier: Box::new(models::OperationIdentifier::new(index)),
        related_operations: None,
        _type: op_type.to_string(),
        status: Some(status.to_string()),
        account: Some(Box::new(account)),
        amount: Some(Box::new(to_mesh_api_amount(amount, currency)?)),
        coin_change: None,
        metadata: None,
    })
}

pub fn to_mesh_api_transaction_identifier(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
    state_version: StateVersion,
    transaction_identifiers: &CommittedTransactionIdentifiers,
    requested_transaction_identifier: Option<&str>,
) -> Result<(Vec<models::Operation>, models::TransactionIdentifier), MappingError> {
    let (operations, transaction_identifier) = match transaction_identifiers
        .transaction_hashes
        .as_user()
    {
        // TODO:MESH Support non-user transactions.
        // For now we take into account only user transactions.
        // For non-user we return empty operations vector and artificial transaction identifier
        // (unfortunately non-user transactions don't have txid, let's use state_version as
        // transaction_identifier).
        None => {
            let transaction_identifier = format!("state_version_{}", state_version);
            if requested_transaction_identifier.is_some_and(|tx_id| tx_id != transaction_identifier)
            {
                return Err(MappingError::InvalidTransactionIdentifier {
                    message: format!("transaction_identifier does not match with block_identifier"),
                });
            }

            (vec![], transaction_identifier)
        }
        Some(user_hashes) => {
            let transaction_identifier = to_api_transaction_hash_bech32m(
                mapping_context,
                &user_hashes.transaction_intent_hash,
            )?;

            if requested_transaction_identifier.is_some_and(|tx_id| tx_id != transaction_identifier)
            {
                return Err(MappingError::InvalidTransactionIdentifier {
                    message: format!("transaction_identifier does not match with block_identifier"),
                });
            }
            let local_transaction_execution = database
                .get_committed_local_transaction_execution(state_version)
                .ok_or_else(|| MappingError::InvalidTransactionIdentifier {
                    message: format!(
                        "No transaction found at state version {}",
                        state_version.number()
                    ),
                })?;

            let status = MeshApiOperationStatus::from(local_transaction_execution.outcome);

            let mut index = 0_i64;
            let mut operations = vec![];
            for (address, balance_changes) in local_transaction_execution
                .global_balance_summary
                .global_balance_changes
            {
                // TODO:MESH support LockFee, Mint, Burn
                // see https://github.com/radixdlt/babylon-node/pull/1018#discussion_r1834905560
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
    Ok((
        operations,
        models::TransactionIdentifier::new(transaction_identifier),
    ))
}
