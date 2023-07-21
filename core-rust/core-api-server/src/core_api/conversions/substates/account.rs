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
                AccountDefaultDepositRule::Accept => models::DefaultDepositRule::Accept,
                AccountDefaultDepositRule::Reject => models::DefaultDepositRule::Reject,
                AccountDefaultDepositRule::AllowExisting =>
                    models::DefaultDepositRule::AllowExisting,
            },
        }
    ))
}

pub fn to_api_account_vault_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<Own>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountVaultIndexKey(resource_address)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Account Vault Key".to_string() });
    };
    Ok(key_value_store_mandatory_substate!(
        substate,
        AccountVaultIndexEntry,
        models::ResourceKey {
            resource_address: to_api_resource_address(context, resource_address)?,
        },
        value => {
            vault: Box::new(to_api_entity_reference(context, value.as_node_id())?),
        }
    ))
}

pub fn to_api_account_deposit_rule_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<AccountResourceDepositRuleEntry>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountResourceDepositRuleIndexKey(resource_address)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Account Deposit Rule Key".to_string() });
    };
    Ok(key_value_store_optional_substate!(
        substate,
        AccountDepositRuleIndexEntry,
        models::ResourceKey {
            resource_address: to_api_resource_address(context, resource_address)?,
        },
        value => {
            deposit_rule: value.map(|rule| match rule {
                ResourceDepositRule::Neither => models::DepositRule::Neither,
                ResourceDepositRule::Allowed => models::DepositRule::Allowed,
                ResourceDepositRule::Disallowed => models::DepositRule::Disallowed,
            }),
        }
    ))
}
