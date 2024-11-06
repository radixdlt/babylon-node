use crate::prelude::*;

pub fn to_api_component_address(
    context: &MappingContext,
    component_address: &ComponentAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, component_address.as_node_id())
}

pub fn to_api_resource_address(
    context: &MappingContext,
    resource_address: &ResourceAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, resource_address.as_node_id())
}

pub fn to_api_package_address(
    context: &MappingContext,
    package_address: &PackageAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, package_address.as_node_id())
}

pub fn to_api_entity_address(
    context: &MappingContext,
    node_id: &NodeId,
) -> Result<String, MappingError> {
    context
        .address_encoder
        .encode(node_id.as_ref())
        .map_err(|err| MappingError::InvalidEntityAddress { encode_error: err })
}

pub fn extract_resource_address(
    extraction_context: &ExtractionContext,
    resource_address: &str,
) -> Result<ResourceAddress, ExtractionError> {
    ResourceAddress::try_from_bech32(&extraction_context.address_decoder, resource_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_component_address(
    extraction_context: &ExtractionContext,
    component_address: &str,
) -> Result<ComponentAddress, ExtractionError> {
    ComponentAddress::try_from_bech32(&extraction_context.address_decoder, component_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn to_mesh_api_currency_from_resource_address(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
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

pub(crate) fn extract_resource_address_from_mesh_api_currency(
    extraction_context: &ExtractionContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
    currency: &models::Currency,
) -> Result<ResourceAddress, ExtractionError> {
    // currency.symbol field keeps bech32-encoded resource address
    let resource_address = extract_resource_address(extraction_context, &currency.symbol)?;
    let resource_node_id = resource_address.as_node_id();

    if resource_node_id.entity_type() != Some(EntityType::GlobalFungibleResourceManager) {
        return Err(ExtractionError::InvalidCurrency {
            message: format!("currency {} is not fungible type", currency.symbol),
        });
    }

    let divisibility: FungibleResourceManagerDivisibilityFieldSubstate =
        read_optional_main_field_substate(
            database,
            resource_node_id,
            &FungibleResourceManagerField::Divisibility.into(),
        )
        .ok_or_else(|| ExtractionError::InvalidCurrency {
            message: format!("currency {} not found", currency.symbol),
        })?;
    let divisibility = *divisibility.payload().as_unique_version() as i32;

    if divisibility != currency.decimals {
        return Err(ExtractionError::InvalidCurrency {
            message: format!(
                "currency {} decimals mismatch, specified: {}, current: {}",
                &currency.symbol, &currency.decimals, divisibility
            ),
        });
    }
    Ok(resource_address)
}
