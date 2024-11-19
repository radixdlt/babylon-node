use crate::prelude::*;

pub fn to_mesh_api_currency_from_resource_address(
    mapping_context: &MappingContext,
    database: &dyn SubstateDatabase,
    resource_address: &ResourceAddress,
) -> Result<models::Currency, MappingError> {
    let resource_node_id = resource_address.as_node_id();
    // currency.symbol field keeps bech32-encoded resource address
    let symbol = to_api_resource_address(mapping_context, resource_address)?;

    if resource_node_id.entity_type() != Some(EntityType::GlobalFungibleResourceManager) {
        return Err(MappingError::InvalidResource {
            message: format!("resource {} is not fungible type", symbol),
        });
    }
    let divisibility: FungibleResourceManagerDivisibilityFieldSubstate =
        read_optional_main_field_substate(
            database,
            resource_node_id,
            &FungibleResourceManagerField::Divisibility.into(),
        )
        .ok_or_else(|| MappingError::InvalidResource {
            message: format!("currency {} not found", symbol),
        })?;
    let decimals = *divisibility.payload().as_unique_version() as i32;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#currency for field
    // definitions
    Ok(models::Currency {
        symbol,
        decimals,
        metadata: None,
    })
}
