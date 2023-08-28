use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_account_state_substate(
    _context: &MappingContext,
    substate: &FieldSubstate<AccountSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        AccountFieldState,
        AccountSubstate {
            default_deposit_rule,
        },
        Value {
            default_deposit_rule: match default_deposit_rule {
                DefaultDepositRule::Accept => models::DefaultDepositRule::Accept,
                DefaultDepositRule::Reject => models::DefaultDepositRule::Reject,
                DefaultDepositRule::AllowExisting => models::DefaultDepositRule::AllowExisting,
            },
        }
    ))
}

pub fn to_api_account_vault_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<Vault>,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountVaultKey(
            resource_address
        ))
    );
    Ok(key_value_store_mandatory_substate!(
        substate,
        AccountVaultEntry,
        models::ResourceKey {
            resource_address: to_api_resource_address(context, resource_address)?,
        },
        value => {
            vault: Box::new(to_api_entity_reference(context, value.0.as_node_id())?),
        }
    ))
}

pub fn to_api_account_resource_preference_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<ResourcePreference>,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountResourcePreferenceKey(
            resource_address
        ))
    );
    Ok(key_value_store_optional_substate!(
        substate,
        AccountResourcePreferenceEntry,
        models::ResourceKey {
            resource_address: to_api_resource_address(context, resource_address)?,
        },
        resource_preference => {
            resource_preference: match resource_preference {
                ResourcePreference::Allowed => models::ResourcePreference::Allowed,
                ResourcePreference::Disallowed => models::ResourcePreference::Disallowed,
            },
        }
    ))
}

pub fn to_api_account_authorized_depositor_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<AccountAuthorizedDepositorEntryContents>,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountAuthorizedDepositorKey(
            authorized_depositor_badge
        ))
    );
    Ok(key_value_store_optional_substate!(
        substate,
        AccountAuthorizedDepositorEntry,
        models::AuthorizedDepositorKey {
            badge: Some(to_api_authorized_depositor_badge(context, authorized_depositor_badge)?),
        },
        () => {
            is_authorized: true,
        },
    ))
}

pub fn to_api_authorized_depositor_badge(
    context: &MappingContext,
    resource_or_non_fungible: &ResourceOrNonFungible,
) -> Result<models::AuthorizedDepositorBadge, MappingError> {
    Ok(match resource_or_non_fungible {
        ResourceOrNonFungible::Resource(resource_address) => {
            models::AuthorizedDepositorBadge::ResourceAuthorizedDepositorBadge {
                resource_address: to_api_resource_address(context, resource_address)?,
            }
        }
        ResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
            models::AuthorizedDepositorBadge::NonFungibleAuthorizedDepositorBadge {
                non_fungible_global_id: Box::new(to_api_non_fungible_global_id(
                    context,
                    non_fungible_global_id,
                )?),
            }
        }
    })
}
