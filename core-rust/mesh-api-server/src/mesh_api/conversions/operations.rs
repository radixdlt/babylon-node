use crate::engine_prelude::*;
use crate::prelude::*;
use radix_engine_interface::blueprints::account::{
    AccountTryDepositBatchOrAbortManifestInput, AccountTryDepositOrAbortManifestInput,
    AccountWithdrawManifestInput,
};
use radix_transactions::manifest::{CallMethod, TakeFromWorktop};

#[derive(Debug, Clone, Copy, EnumIter, Display, EnumString)]
pub(crate) enum MeshApiOperationType {
    Withdraw,
    Deposit,
    FeePayment,
}

#[derive(Debug, Clone, Copy, EnumIter, Display)]
pub(crate) enum MeshApiOperationStatus {
    #[strum(serialize = "Success")]
    Success,
    #[strum(serialize = "Failure")]
    Failure,
}

impl From<MeshApiOperationStatus> for models::OperationStatus {
    fn from(value: MeshApiOperationStatus) -> Self {
        let successful = match value {
            MeshApiOperationStatus::Failure => false,
            MeshApiOperationStatus::Success => true,
        };
        Self::new(value.to_string(), successful)
    }
}

pub fn to_mesh_api_operation_no_fee(
    mapping_context: &MappingContext,
    database: &ActualStateManagerDatabase,
    index: i64,
    status: Option<MeshApiOperationStatus>,
    account_address: &GlobalAddress,
    resource_address: &ResourceAddress,
    amount: Decimal,
) -> Result<models::Operation, MappingError> {
    let op_type = if amount.is_positive() {
        MeshApiOperationType::Deposit
    } else {
        MeshApiOperationType::Withdraw
    };
    let currency =
        to_mesh_api_currency_from_resource_address(mapping_context, database, resource_address)?;
    let account = to_api_account_identifier_from_global_address(mapping_context, account_address)?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#operation
    Ok(models::Operation {
        operation_identifier: Box::new(models::OperationIdentifier::new(index)),
        related_operations: None,
        _type: op_type.to_string(),
        status: status.map(|s| s.to_string()),
        account: Some(Box::new(account)),
        amount: Some(Box::new(to_mesh_api_amount(amount, currency)?)),
        coin_change: None,
        metadata: None,
    })
}

pub fn to_mesh_api_operation_fee_payment(
    mapping_context: &MappingContext,
    database: &ActualStateManagerDatabase,
    index: i64,
    account_address: &GlobalAddress,
    amount: Decimal,
) -> Result<models::Operation, MappingError> {
    let currency = to_mesh_api_currency_from_resource_address(mapping_context, database, &XRD)?;
    let account = to_api_account_identifier_from_global_address(mapping_context, account_address)?;

    // see https://docs.cdp.coinbase.com/mesh/docs/models#operation
    Ok(models::Operation {
        operation_identifier: Box::new(models::OperationIdentifier::new(index)),
        related_operations: None,
        _type: MeshApiOperationType::FeePayment.to_string(),
        // Fee payment is always success, even if transaction failed
        status: Some(MeshApiOperationStatus::Success.to_string()),
        account: Some(Box::new(account)),
        amount: Some(Box::new(to_mesh_api_amount(amount, currency)?)),
        coin_change: None,
        metadata: None,
    })
}

pub fn to_mesh_api_operations(
    mapping_context: &MappingContext,
    database: &ActualStateManagerDatabase,
    state_version: StateVersion,
) -> Result<Vec<models::Operation>, MappingError> {
    let local_execution = database
        .get_committed_local_transaction_execution(state_version)
        .ok_or_else(|| MappingError::InvalidTransactionIdentifier {
            message: format!(
                "No transaction found at state version {}",
                state_version.number()
            ),
        })?;
    let fee_balance_changes =
        resolve_global_fee_balance_changes(database, &local_execution.fee_source)?;

    let fee_payment_computation = FeePaymentComputer::compute(FeePaymentComputationInputs {
        fee_balance_changes,
        fee_summary: &local_execution.fee_summary,
        fee_destination: &local_execution.fee_destination,
        balance_changes: &local_execution
            .global_balance_summary
            .global_balance_changes,
    });

    let mut output = Vec::with_capacity(fee_payment_computation.relevant_entities.len());
    for entity in &fee_payment_computation.relevant_entities {
        if entity.is_account() {
            if let Some(fee_balance_changes) =
                fee_payment_computation.fee_balance_changes.get(&entity)
            {
                for amount in fee_balance_changes
                    .iter()
                    .filter_map(|(fee_payment_type, amount)| {
                        if *fee_payment_type == FeePaymentBalanceChangeType::FeePayment {
                            Some(amount)
                        } else {
                            None
                        }
                    })
                {
                    let operation = to_mesh_api_operation_fee_payment(
                        mapping_context,
                        database,
                        output.len() as i64,
                        entity,
                        *amount,
                    )?;
                    output.push(operation)
                }
            }

            if let Some(non_fee_balance_changes) =
                fee_payment_computation.non_fee_balance_changes.get(&entity)
            {
                for (resource_address, amount) in non_fee_balance_changes {
                    let operation = to_mesh_api_operation_no_fee(
                        mapping_context,
                        database,
                        output.len() as i64,
                        // There are no non-fee balance changes for failed transactions
                        Some(MeshApiOperationStatus::Success),
                        entity,
                        resource_address,
                        *amount,
                    )?;
                    output.push(operation)
                }
            }
        }
    }

    Ok(output)
}

/// Uses the [`SubstateNodeAncestryStore`] (from the given DB) to transform the input
/// `vault ID -> payment` map into a `global address -> balance change` map.
fn resolve_global_fee_balance_changes(
    database: &ActualStateManagerDatabase,
    fee_source: &FeeSource,
) -> Result<IndexMap<GlobalAddress, Decimal>, MappingError> {
    let paying_vaults = &fee_source.paying_vaults;
    let ancestries = database.batch_get_ancestry(paying_vaults.keys());
    let mut fee_balance_changes = index_map_new();
    for ((vault_id, paid_fee_amount_xrd), ancestry) in paying_vaults.iter().zip(ancestries) {
        let ancestry = ancestry.ok_or_else(|| MappingError::InternalIndexDataMismatch {
            message: format!("no ancestry record for vault {}", vault_id.to_hex()),
        })?;
        let global_ancestor_address = GlobalAddress::new_or_panic(ancestry.root.0.into());
        let fee_balance_change = fee_balance_changes
            .entry(global_ancestor_address)
            .or_insert_with(Decimal::zero);
        *fee_balance_change = fee_balance_change.sub_or_panic(*paid_fee_amount_xrd);
    }
    Ok(fee_balance_changes)
}

/// This method converts Transaction V1 instructons to operations.
/// The parser is limited to the instructions generated by the Mesh construction API.
pub fn to_mesh_api_operations_from_instructions_v1(
    instructions: &[InstructionV1],
    mapping_context: &MappingContext,
    database: &ActualStateManagerDatabase,
) -> Result<Vec<models::Operation>, ResponseError> {
    let mut operations = Vec::new();
    let mut next_index = 0;
    let mut withdraw_input: Option<(ResourceAddress, Decimal)> = None;

    while next_index < instructions.len() {
        let mut instruction = &instructions[next_index];
        next_index = next_index + 1;
        match instruction {
            InstructionV1::CallMethod(CallMethod {
                address: DynamicGlobalAddress::Static(global_address),
                method_name,
                args,
            }) => {
                let args_bytes = manifest_encode(&args).unwrap();
                match method_name.as_str() {
                    "lock_fee" => (),
                    "withdraw" if global_address.is_account() => {
                        let input = manifest_decode::<AccountWithdrawManifestInput>(&args_bytes)
                            .map_err(|_| {
                                ResponseError::from(ApiError::InvalidManifestInstruction)
                                    .with_details("Invalid withdraw instruction")
                            })?;
                        let resource_adddress = &match input.resource_address {
                            ManifestResourceAddress::Static(resource_address) => resource_address,
                            ManifestResourceAddress::Named(_) => {
                                return Err(ResponseError::from(ApiError::NamedAddressNotSupported)
                                    .with_details("Named address is not supported"))
                            }
                        };

                        operations.push(to_mesh_api_operation_no_fee(
                            mapping_context,
                            database,
                            operations.len() as i64,
                            None,
                            global_address,
                            resource_adddress,
                            -input.amount.clone(),
                        )?);
                        withdraw_input = Some((*resource_adddress, input.amount));
                    }
                    // Below assumes that previous operation was Withdraw and whole withdraw amount
                    // shall be deposited to the global address
                    "try_deposit_batch_or_abort"
                        if global_address.is_account() && withdraw_input.is_some() =>
                    {
                        let (resource_address, amount) = withdraw_input.unwrap();
                        if let Ok(_input) = manifest_decode::<
                            AccountTryDepositBatchOrAbortManifestInput,
                        >(&args_bytes)
                        {
                            operations.push(to_mesh_api_operation_no_fee(
                                mapping_context,
                                database,
                                operations.len() as i64,
                                None,
                                global_address,
                                &resource_address,
                                amount,
                            )?);
                        } else {
                            return Err(ResponseError::from(ApiError::InvalidManifestInstruction)
                                .with_details("Invalid try_deposit_batch_or_abort instruction"));
                        }
                    }
                    _ => {
                        return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                            .with_details(format!("Unrecognized instruction: {:?}", instruction)));
                    }
                }
            }
            InstructionV1::TakeFromWorktop(TakeFromWorktop {
                resource_address,
                amount,
            }) if next_index < instructions.len() => {
                instruction = &instructions[next_index];
                next_index = next_index + 1;

                match instruction {
                    InstructionV1::CallMethod(CallMethod {
                        address: DynamicGlobalAddress::Static(global_address),
                        method_name,
                        args,
                    }) if method_name.eq("try_deposit_or_abort") && global_address.is_account() => {
                        if let Ok(_input) = manifest_decode::<AccountTryDepositOrAbortManifestInput>(
                            &manifest_encode(&args).unwrap(),
                        ) {
                            operations.push(to_mesh_api_operation_no_fee(
                                mapping_context,
                                database,
                                operations.len() as i64,
                                None,
                                global_address,
                                resource_address,
                                *amount,
                            )?);
                        } else {
                            return Err(ResponseError::from(ApiError::InvalidManifestInstruction)
                                .with_details("Invalid try_deposit_or_abort instruction"));
                        }
                    }
                    _ => {
                        return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                            .with_details(format!("Unrecognized instruction: {:?}", instruction)));
                    }
                }
                withdraw_input = None;
            }
            _ => {
                return Err(ResponseError::from(ApiError::UnrecognizedInstruction)
                    .with_details(format!("Unrecognized instruction: {:?}", instruction)));
            }
        }
    }

    Ok(operations)
}
