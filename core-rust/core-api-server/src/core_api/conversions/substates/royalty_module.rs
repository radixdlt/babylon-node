use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_component_royalty_substate(
    context: &MappingContext,
    substate: &ComponentRoyaltySubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let ComponentRoyaltySubstate {
        enabled,
        royalty_vault,
    } = substate;
    Ok(field_substate!(
        substate,
        RoyaltyModuleFieldState,
        {
            is_enabled: *enabled,
            vault_entity: Box::new(to_api_entity_reference(
                context,
                royalty_vault.0.as_node_id(),
            )?),
        }
    ))
}

pub fn to_api_component_method_royalty_substate(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &ComponentMethodRoyaltySubstate,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::RoyaltyModule(TypedRoyaltyModuleSubstateKey::RoyaltyMethodRoyaltyEntryKey(method_name)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "RoyaltyMethodRoyaltyEntryKey".to_string() });
    };
    Ok(key_value_store_optional_substate!(
        substate,
        RoyaltyModuleMethodRoyaltyEntry,
        models::MainMethodKey {
            method_name: method_name.to_string(),
        },
        value => {
            royalty_amount: to_api_royalty_amount(value).map(Box::new),
        }
    ))
}

pub fn to_api_royalty_amount(royalty_amount: &RoyaltyAmount) -> Option<models::RoyaltyAmount> {
    match royalty_amount {
        RoyaltyAmount::Free => None,
        RoyaltyAmount::Xrd(amount) => Some(models::RoyaltyAmount::new(
            to_api_decimal(amount),
            models::royalty_amount::Unit::XRD,
        )),
        RoyaltyAmount::Usd(amount) => Some(models::RoyaltyAmount::new(
            to_api_decimal(amount),
            models::royalty_amount::Unit::USD,
        )),
    }
}
