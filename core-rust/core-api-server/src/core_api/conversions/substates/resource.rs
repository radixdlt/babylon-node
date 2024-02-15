use super::super::*;
use super::*;
use crate::core_api::models;
use crate::scrypto_prelude::*;

pub fn to_api_fungible_vault_balance_substate(
    _context: &MappingContext,
    substate: &FungibleVaultBalanceFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        FungibleVaultFieldBalance,
        balance,
        Value {
            amount: to_api_decimal(&balance.amount()),
        }
    ))
}

pub fn to_api_fungible_vault_frozen_status_substate(
    _context: &MappingContext,
    substate: &FungibleVaultFreezeStatusFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        FungibleVaultFieldFrozenStatus,
        VaultFrozenFlag { frozen },
        Value {
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
    substate: &NonFungibleVaultBalanceFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        NonFungibleVaultFieldBalance,
        NonFungibleVaultBalance { amount },
        Value {
            amount: to_api_decimal(amount),
        }
    ))
}

pub fn to_api_non_fungible_vault_frozen_status_substate(
    _context: &MappingContext,
    substate: &NonFungibleVaultFreezeStatusFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        FungibleVaultFieldFrozenStatus,
        VaultFrozenFlag { frozen },
        Value {
            frozen_status: Box::new(to_api_frozen_status(frozen)),
        }
    ))
}

pub fn to_api_non_fungible_vault_contents_entry_substate(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    _substate: &NonFungibleVaultNonFungibleEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultTypedSubstateKey::NonFungibleIndexEntry(
                NonFungibleVaultNonFungibleKeyPayload {
                    content: non_fungible_local_id
                }
            )
        ))
    );
    Ok(index_substate!(
        substate,
        NonFungibleVaultContentsIndexEntry,
        models::LocalNonFungibleKey {
            non_fungible_local_id: Box::new(to_api_non_fungible_local_id(non_fungible_local_id)),
        },
        {
            is_present: true,
        }
    ))
}

pub fn to_api_fungible_resource_manager_divisibility_substate(
    substate: &FungibleResourceManagerDivisibilityFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        FungibleResourceManagerFieldDivisibility,
        divisibility,
        Value {
            divisibility: to_api_u8_as_i32(*divisibility),
        }
    ))
}

pub fn to_api_fungible_resource_manager_total_supply_substate(
    substate: &FungibleResourceManagerTotalSupplyFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        FungibleResourceManagerFieldTotalSupply,
        total_supply,
        Value {
            total_supply: to_api_decimal(total_supply),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_id_type_substate(
    substate: &NonFungibleResourceManagerIdTypeFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        NonFungibleResourceManagerFieldIdType,
        non_fungible_id_type,
        Value {
            non_fungible_id_type: to_api_non_fungible_id_type(non_fungible_id_type),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_total_supply_substate(
    substate: &NonFungibleResourceManagerTotalSupplyFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        NonFungibleResourceManagerFieldTotalSupply,
        total_supply,
        Value {
            total_supply: to_api_decimal(total_supply),
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_mutable_fields_substate(
    context: &MappingContext,
    substate: &NonFungibleResourceManagerMutableFieldsFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        NonFungibleResourceManagerFieldMutableFields,
        NonFungibleResourceManagerMutableFields {
            mutable_field_index
        },
        Value {
            mutable_fields: mutable_field_index
                .iter()
                .map(|(name, index)| to_api_mutable_field(context, name.as_str(), *index))
                .collect::<Result<Vec<_>, _>>()?
        }
    ))
}

pub fn to_api_non_fungible_resource_manager_data_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &NonFungibleResourceManagerDataEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceManager(
            NonFungibleResourceManagerTypedSubstateKey::DataKeyValueEntry(
                NonFungibleResourceManagerDataKeyPayload {
                    content: non_fungible_local_id
                }
            )
        ))
    );
    Ok(key_value_store_optional_substate!(
        substate,
        NonFungibleResourceManagerDataEntry,
        models::LocalNonFungibleKey {
            non_fungible_local_id: Box::new(to_api_non_fungible_local_id(non_fungible_local_id)),
        },
        NonFungibleResourceManagerDataEntryPayload { content: value } => {
            data_struct: Box::new(to_api_data_struct_from_scrypto_value(context, value)?),
        }
    ))
}

pub fn to_api_mutable_field(
    _context: &MappingContext,
    name: &str,
    index: usize,
) -> Result<models::MutableField, MappingError> {
    Ok(models::MutableField {
        name: name.to_owned(),
        index: to_api_index_as_i64(index)?,
    })
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

pub fn extract_resource_or_non_fungible(
    extraction_context: &ExtractionContext,
    badge: &models::PresentedBadge,
) -> Result<ResourceOrNonFungible, ExtractionError> {
    Ok(match badge {
        models::PresentedBadge::ResourcePresentedBadge { resource_address } => {
            ResourceOrNonFungible::from(extract_resource_address(
                extraction_context,
                resource_address,
            )?)
        }
        models::PresentedBadge::NonFungiblePresentedBadge {
            resource_address,
            local_id,
        } => ResourceOrNonFungible::from(NonFungibleGlobalId::new(
            extract_resource_address(extraction_context, resource_address)?,
            extract_non_fungible_id_from_simple_representation(local_id)?,
        )),
    })
}
