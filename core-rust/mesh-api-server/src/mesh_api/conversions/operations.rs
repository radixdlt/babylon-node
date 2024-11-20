use crate::engine_prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, EnumIter, Display, EnumString)]
pub(crate) enum MeshApiOperationType {
    Withdraw,
    Deposit,
}

#[derive(Debug, Clone, Copy, EnumIter, Display)]
pub(crate) enum MeshApiOperationStatus {
    #[strum(serialize = "Success")]
    Success,
    #[strum(serialize = "Failure")]
    Failure,
}

// TODO:MESH This one might be confusing. Failed transaction will still have successful FeePayment
// operation
impl From<DetailedTransactionOutcome> for MeshApiOperationStatus {
    fn from(value: DetailedTransactionOutcome) -> Self {
        match value {
            DetailedTransactionOutcome::Success(..) => Self::Success,
            DetailedTransactionOutcome::Failure(..) => Self::Failure,
        }
    }
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
    database: &StateManagerDatabase<impl ReadableRocks>,
    index: i64,
    status: Option<MeshApiOperationStatus>,
    account_address: &GlobalAddress,
    resource_address: &ResourceAddress,
    amount: Decimal,
) -> Result<models::Operation, MappingError> {
    // TODO:MESH what about fee locking, burning, minting?
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

pub fn to_mesh_api_operations(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
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
    let status = MeshApiOperationStatus::from(local_execution.outcome);
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
            if let Some(non_fee_balance_changes) =
                fee_payment_computation.non_fee_balance_changes.get(&entity)
            {
                for (resource_address, amount) in non_fee_balance_changes {
                    let operation = to_mesh_api_operation_no_fee(
                        mapping_context,
                        database,
                        output.len() as i64,
                        Some(status),
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
    database: &StateManagerDatabase<impl ReadableRocks>,
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
