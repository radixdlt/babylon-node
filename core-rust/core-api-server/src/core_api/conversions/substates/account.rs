use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_account_state_substate(
    _context: &MappingContext,
    substate: &AccountDepositRuleFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
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
    substate: &AccountResourceVaultEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::ResourceVaultKeyValueEntry(AccountResourceVaultKeyPayload {
                content: resource_address
            })
        ))
    );
    Ok(key_value_store_mandatory_substate_versioned!(
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
    substate: &AccountResourcePreferenceEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::ResourcePreferenceKeyValueEntry(
                AccountResourcePreferenceKeyPayload {
                    content: resource_address
                }
            )
        ))
    );
    Ok(key_value_store_optional_substate_versioned!(
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
    substate: &AccountAuthorizedDepositorEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::AuthorizedDepositorKeyValueEntry(
                AccountAuthorizedDepositorKeyPayload {
                    content: authorized_depositor_badge
                }
            )
        ))
    );
    Ok(key_value_store_optional_substate_versioned!(
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
