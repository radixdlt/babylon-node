use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_account_locker_account_claim_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &AccountLockerAccountClaimsEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountLocker(
            AccountLockerTypedSubstateKey::AccountClaimsKeyValueEntry(
                AccountLockerAccountClaimsKeyPayload {
                    content: global_account
                }
            )
        ))
    );
    Ok(key_value_store_mandatory_substate_single_versioned!(
        substate,
        AccountLockerAccountClaimsEntry,
        models::AccountAddressKey {
            account_address: to_api_component_address(context, &global_account.0)?,
        },
        Own(resource_vaults) => {
            resource_vaults: Box::new(to_api_entity_reference(context, resource_vaults)?),
        }
    ))
}
