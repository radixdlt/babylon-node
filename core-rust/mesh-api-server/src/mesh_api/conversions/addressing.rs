use crate::prelude::*;

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

pub(crate) fn extract_resource_address_from_currency(
    extraction_context: &ExtractionContext,
    database: &impl SubstateDatabase,
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

pub fn to_api_resource_address(
    context: &MappingContext,
    resource_address: &ResourceAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, resource_address.as_node_id())
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

pub fn to_api_account_identifier_from_global_address(
    mapping_context: &MappingContext,
    address: impl AsRef<NodeId>,
) -> Result<models::AccountIdentifier, MappingError> {
    let node_id: &NodeId = address.as_ref();
    let address = to_api_entity_address(mapping_context, node_id)?;

    if !node_id.is_global_account() {
        return Err(MappingError::InvalidAccount {
            message: format!("address {} is not an account", address),
        });
    }

    // See https://docs.cdp.coinbase.com/mesh/docs/models#accountidentifier for field
    // definitions
    Ok(models::AccountIdentifier {
        address,
        sub_account: None,
        metadata: None,
    })
}

pub fn to_api_account_identifier_from_public_key(
    mapping_context: &MappingContext,
    public_key: PublicKey,
) -> Result<models::AccountIdentifier, MappingError> {
    let address = mapping_context
        .address_encoder
        .encode(ComponentAddress::preallocated_account_from_public_key(&public_key).as_bytes())
        .map_err(|err| MappingError::InvalidEntityAddress { encode_error: err })?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#accountidentifier for field
    // definitions
    Ok(models::AccountIdentifier {
        address,
        sub_account: None,
        metadata: None,
    })
}
pub fn extract_radix_account_address_from_account_identifier(
    extraction_context: &ExtractionContext,
    account_identifier: &models::AccountIdentifier,
) -> Result<ComponentAddress, ExtractionError> {
    if account_identifier.sub_account.is_some() {
        return Err(ExtractionError::InvalidAccount {
            message: format!("Sub accounts not supported"),
        });
    }
    let component_address =
        extract_component_address(extraction_context, &account_identifier.address)?;

    if component_address.as_node_id().is_global_account() {
        Ok(component_address)
    } else {
        Err(ExtractionError::InvalidAccount {
            message: format!("Whilst this API returns balance changes with an `AccountIdentifier` representing any global entity to allow full reconciliation, only Radix account addresses starting `account_` are accepted for the construction and account balance endpoints. {} is not a Radix account.", account_identifier.address),
        })
    }
}
