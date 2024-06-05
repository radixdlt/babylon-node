use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_component_royalty_substate(
    context: &MappingContext,
    substate: &ComponentRoyaltyAccumulatorFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_single_versioned!(
        substate,
        RoyaltyModuleFieldState,
        ComponentRoyaltySubstate { royalty_vault },
        Value {
            is_enabled: true,
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
    substate: &KeyValueEntrySubstate<ComponentRoyaltyMethodAmountEntryPayload>,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::RoyaltyModule(
            TypedRoyaltyModuleSubstateKey::RoyaltyMethodRoyaltyEntryKey(method_name)
        )
    );
    Ok(key_value_store_optional_substate_single_versioned!(
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
