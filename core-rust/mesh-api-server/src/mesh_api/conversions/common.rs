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

    let account = to_mesh_api_acount_from_address(mapping_context, account_address)?;
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
