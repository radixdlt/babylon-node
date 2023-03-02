use super::addressing::*;
use crate::core_api::*;
use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine::system::kernel_modules::costing::{FeeSummary, RoyaltyReceiver};
use radix_engine::{
    ledger::OutputValue,
    types::{hash, scrypto_encode, Decimal, RENodeId, SubstateId},
};
use radix_engine_interface::api::component::ComponentAddress;
use radix_engine_interface::api::types::{Address, NodeModuleId};
use std::collections::BTreeMap;

use state_manager::{DeletedSubstateVersion, LedgerTransactionOutcome, LedgerTransactionReceipt};

pub fn to_api_receipt(
    context: &MappingContext,
    receipt: LedgerTransactionReceipt,
) -> Result<models::TransactionReceipt, MappingError> {
    let fee_summary = receipt.fee_summary;

    let (status, output, error_message) = match receipt.outcome {
        LedgerTransactionOutcome::Success(output) => {
            (models::TransactionStatus::Succeeded, Some(output), None)
        }
        LedgerTransactionOutcome::Failure(error) => (
            models::TransactionStatus::Failed,
            None,
            Some(format!("{:?}", error)),
        ),
    };

    let substate_changes = receipt.substate_changes;

    let mut new_global_entities = Vec::new();
    let mut created = Vec::new();

    for (id, output) in substate_changes.created {
        match id {
            SubstateId(RENodeId::GlobalPackage(package_address), NodeModuleId::TypeInfo, ..) => {
                new_global_entities
                    .push(to_global_entity_reference(context, &package_address.into()));
            }
            SubstateId(
                RENodeId::GlobalComponent(component_address),
                NodeModuleId::TypeInfo,
                ..,
            ) => {
                new_global_entities.push(to_global_entity_reference(
                    context,
                    &component_address.into(),
                ));
            }
            SubstateId(
                RENodeId::GlobalResourceManager(resource_address),
                NodeModuleId::TypeInfo,
                ..,
            ) => {
                new_global_entities.push(to_global_entity_reference(
                    context,
                    &resource_address.into(),
                ));
            }
            _ => {}
        }
        let substate_version = to_api_new_substate_version(context, (id.clone(), output))?;
        created.push(substate_version)
    }

    let updated = substate_changes
        .updated
        .into_iter()
        .map(|substate_kv| to_api_new_substate_version(context, substate_kv))
        .collect::<Result<Vec<_>, _>>()?;

    let deleted = substate_changes
        .deleted
        .into_iter()
        .map(to_api_deleted_substate)
        .collect::<Result<Vec<_>, _>>()?;

    let api_state_updates = models::StateUpdates {
        created_substates: created,
        updated_substates: updated,
        deleted_substates: deleted,
        new_global_entities,
    };

    let api_fee_summary = to_api_fee_summary(context, fee_summary)?;

    let api_output = match output {
        Some(output) => Some(
            output
                .into_iter()
                .map(|line_output| scrypto_bytes_to_api_sbor_data(context, &line_output))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };

    let next_epoch = if let Some(next_epoch) = receipt.next_epoch {
        Some(Box::new(to_api_next_epoch(context, next_epoch)?))
    } else {
        None
    };

    Ok(models::TransactionReceipt {
        status,
        fee_summary: Box::new(api_fee_summary),
        state_updates: Box::new(api_state_updates),
        output: api_output,
        next_epoch,
        error_message,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_new_substate_version(
    context: &MappingContext,
    (substate_id, output_value): (SubstateId, OutputValue),
) -> Result<models::NewSubstateVersion, MappingError> {
    let substate_bytes =
        scrypto_encode(&output_value.substate).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Substate bytes could not be encoded".to_string(),
        })?;
    let hash = to_hex(hash(&substate_bytes));

    let api_substate_data = Some(to_api_substate(
        context,
        &substate_id,
        &output_value.substate,
    )?);

    Ok(models::NewSubstateVersion {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        version: to_api_substate_version(output_value.version)?,
        substate_hex: to_hex(substate_bytes),
        substate_data_hash: hash,
        substate_data: api_substate_data,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_deleted_substate(
    (substate_id, deleted_substate): (SubstateId, DeletedSubstateVersion),
) -> Result<models::DeletedSubstateVersionRef, MappingError> {
    Ok(models::DeletedSubstateVersionRef {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        substate_data_hash: to_hex(deleted_substate.substate_hash),
        version: to_api_substate_version(deleted_substate.version)?,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_next_epoch(
    context: &MappingContext,
    next_epoch: (BTreeMap<ComponentAddress, Validator>, u64),
) -> Result<models::NextEpoch, MappingError> {
    let mut sorted_validators: Vec<(ComponentAddress, Validator)> =
        next_epoch.0.into_iter().map(|e| (e.0, e.1)).collect();
    sorted_validators.sort_by(|a, b| b.1.stake.cmp(&a.1.stake));

    let mut validators = Vec::new();
    for (address, validator) in sorted_validators {
        let api_validator = to_api_active_validator(context, &address, &validator);
        validators.push(api_validator);
    }

    let next_epoch = models::NextEpoch {
        epoch: to_api_epoch(context, next_epoch.1)?,
        validators,
    };

    Ok(next_epoch)
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_summary(
    context: &MappingContext,
    fee_summary: FeeSummary,
) -> Result<models::FeeSummary, MappingError> {
    Ok(models::FeeSummary {
        cost_unit_price: to_api_decimal(&fee_summary.cost_unit_price),
        tip_percentage: to_api_u16_as_i32(fee_summary.tip_percentage),
        cost_unit_limit: to_api_u32_as_i64(fee_summary.cost_unit_limit),
        cost_units_consumed: to_api_u32_as_i64(fee_summary.total_cost_units_consumed),
        xrd_total_execution_cost: to_api_decimal(&fee_summary.total_execution_cost_xrd),
        xrd_total_royalty_cost: to_api_decimal(&fee_summary.total_royalty_cost_xrd),
        xrd_total_tipped: to_api_decimal(&Decimal::ZERO),
        xrd_vault_payments: fee_summary
            .vault_payments_xrd
            .map(|vault_payments| {
                vault_payments
                    .into_iter()
                    .map(|(vault_id, amount)| {
                        Ok(models::VaultPayment {
                            vault_entity: Box::new(to_api_entity_reference(RENodeId::Vault(
                                vault_id,
                            ))?),
                            xrd_amount: to_api_decimal(&amount),
                        })
                    })
                    .collect::<Result<_, _>>()
            })
            .transpose()?,
        cost_unit_execution_breakdown: fee_summary
            .execution_cost_unit_breakdown
            .into_iter()
            .map(|(key, cost_unit_amount)| (key.to_string(), to_api_u32_as_i64(cost_unit_amount)))
            .collect(),
        cost_unit_royalty_breakdown: fee_summary
            .royalty_cost_unit_breakdown
            .into_iter()
            .filter_map(|(receiver, cost_unit_amount)| {
                let global_address = match receiver {
                    RoyaltyReceiver::Package(address) => Address::Package(address),
                    RoyaltyReceiver::Component(RENodeId::GlobalComponent(address)) => {
                        Address::Component(address)
                    }
                    _ => return None,
                };
                let payment = models::RoyaltyPayment {
                    royalty_receiver: Box::new(to_global_entity_reference(
                        context,
                        &global_address,
                    )),
                    cost_unit_amount: to_api_u32_as_i64(cost_unit_amount),
                };
                Some(payment)
            })
            .collect(),
    })
}
