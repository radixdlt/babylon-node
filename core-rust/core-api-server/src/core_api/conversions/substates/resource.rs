use super::*;
use super::super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_fungible_vault_balance_substate(
    _context: &MappingContext,
    balance: &FungibleVaultBalanceSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        FungibleVaultFieldBalance,
        {
            amount: to_api_decimal(&balance.amount()),
        }
    ))
}

pub fn to_api_fungible_vault_frozen_status_substate(
    _context: &MappingContext,
    substate: &VaultFrozenFlag,
) -> Result<models::Substate, MappingError> {
    let VaultFrozenFlag { frozen } = substate;
    Ok(field_substate!(
        substate,
        FungibleVaultFieldFrozenStatus,
        {
            frozen_status: Box::new(to_api_frozen_status(frozen)),
        }
    ))
}

pub fn to_api_frozen_status(vault_freeze_flags: &VaultFreezeFlags) -> models::FrozenStatus {
    let is_withdraw_frozen = vault_freeze_flags.intersects(VaultFreezeFlags::WITHDRAW);
    let is_deposit_frozen = vault_freeze_flags.intersects(VaultFreezeFlags::DEPOSIT);
    let is_burn_frozen = vault_freeze_flags.intersects(VaultFreezeFlags::BURN);
    models::FrozenStatus {
        is_withdraw_frozen,
        is_deposit_frozen,
        is_burn_frozen,
    }
}

pub fn to_api_non_fungible_vault_balance_substate(
    _context: &MappingContext,
    substate: &NonFungibleVaultBalanceSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        NonFungibleVaultFieldBalance,
        {
            amount: to_api_decimal(&substate.amount),
        }
    ))
}

pub fn to_api_non_fungible_vault_frozen_status_substate(
    _context: &MappingContext,
    substate: &VaultFrozenFlag,
) -> Result<models::Substate, MappingError> {
    let VaultFrozenFlag { frozen } = substate;
    Ok(field_substate!(
        substate,
        FungibleVaultFieldFrozenStatus,
        {
            frozen_status: Box::new(to_api_frozen_status(frozen)),
        }
    ))
}

pub fn to_api_non_fungible_vault_contents_entry_substate(
    _context: &MappingContext,
    non_fungible_id: &NonFungibleLocalId,
) -> Result<models::Substate, MappingError> {
    Ok(index_substate!(
        substate,
        NonFungibleVaultContentsIndexEntry,
        models::LocalNonFungibleKey {
            non_fungible_local_id: Box::new(to_api_non_fungible_local_id(non_fungible_id)),
        },
        { }
    ))
}

pub fn to_api_fungible_resource_manager_divisibility_substate(
    substate: &FungibleResourceManagerDivisibilitySubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        FungibleResourceManagerFieldDivisibility,
        {
            divisibility: to_api_u8_as_i32(*substate),
        }
    ))
}

pub fn to_api_fungible_resource_manager_total_supply_substate(
    substate: &FungibleResourceManagerTotalSupplySubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        FungibleResourceManagerFieldTotalSupply,
        {
            total_supply: to_api_decimal(substate),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_id_type_substate(
    substate: &NonFungibleResourceManagerIdTypeSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        NonFungibleResourceManagerFieldIdType,
        {
            non_fungible_id_type: to_api_non_fungible_id_type(substate),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_total_supply_substate(
    substate: &NonFungibleResourceManagerTotalSupplySubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        NonFungibleResourceManagerFieldTotalSupply,
        {
            total_supply: to_api_decimal(substate),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_mutable_fields_substate(
    _context: &MappingContext,
    substate: &NonFungibleResourceManagerMutableFieldsSubstate,
) -> Result<models::Substate, MappingError> {
    let NonFungibleResourceManagerMutableFieldsSubstate { mutable_fields } = substate;
    Ok(field_substate!(
        substate,
        NonFungibleResourceManagerFieldMutableFields,
        {
            mutable_fields: mutable_fields.iter().cloned().collect(),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_data_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<ScryptoRawValue<'_>>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceData(non_fungible_local_id)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "NonFungibleResourceData".to_string() });
    };
    Ok(key_value_store_substate!(
        substate,
        NonFungibleResourceManagerDataEntry,
        models::LocalNonFungibleKey {
            non_fungible_local_id: Box::new(to_api_non_fungible_local_id(non_fungible_local_id)),
        },
        {
            data_struct: substate
                .get_optional_value()
                .map(|value| -> Result<_, MappingError> {
                    Ok(Box::new(to_api_data_struct_from_scrypto_raw_value(context, value)?))
                })
                .transpose()?,
        }
    ))
}

pub fn to_api_fungible_resource_amount(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    amount: &Decimal,
) -> Result<models::ResourceAmount, MappingError> {
    Ok(models::ResourceAmount::FungibleResourceAmount {
        resource_address: to_api_resource_address(context, resource_address)?,
        amount: to_api_decimal(amount),
    })
}

pub fn to_api_non_fungible_resource_amount(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    amount: &Decimal,
    ids: &BTreeSet<NonFungibleLocalId>,
) -> Result<models::ResourceAmount, MappingError> {
    let non_fungible_ids = ids.iter().map(to_api_non_fungible_local_id).collect();
    Ok(models::ResourceAmount::NonFungibleResourceAmount {
        resource_address: to_api_resource_address(context, resource_address)?,
        amount: to_api_decimal(amount),
        non_fungible_ids,
    })
}

pub fn to_api_non_fungible_id_type(id_type: &NonFungibleIdType) -> models::NonFungibleIdType {
    match id_type {
        NonFungibleIdType::String => models::NonFungibleIdType::String,
        NonFungibleIdType::Integer => models::NonFungibleIdType::Integer,
        NonFungibleIdType::Bytes => models::NonFungibleIdType::Bytes,
        NonFungibleIdType::RUID => models::NonFungibleIdType::RUID,
    }
}

pub fn to_api_non_fungible_local_id(
    non_fungible_id: &NonFungibleLocalId,
) -> models::NonFungibleLocalId {
    models::NonFungibleLocalId {
        simple_rep: non_fungible_id.to_string(),
        id_type: to_api_non_fungible_id_type(&non_fungible_id.id_type()),
        sbor_hex: to_hex(scrypto_encode(non_fungible_id).unwrap()),
    }
}

pub fn to_api_non_fungible_global_id(
    context: &MappingContext,
    non_fungible_global_id: &NonFungibleGlobalId,
) -> Result<models::NonFungibleGlobalId, MappingError> {
    Ok(models::NonFungibleGlobalId {
        resource_address: to_api_resource_address(
            context,
            &non_fungible_global_id.resource_address(),
        )?,
        local_id: Box::new(to_api_non_fungible_local_id(
            non_fungible_global_id.local_id(),
        )),
    })
}