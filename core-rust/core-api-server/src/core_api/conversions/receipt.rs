use super::addressing::*;
use crate::core_api::*;
use radix_engine::{
    fee::{FeeSummary, RoyaltyReceiver},
    ledger::OutputValue,
    transaction::TransactionOutcome,
    types::{hash, scrypto_encode, Bech32Encoder, Decimal, GlobalAddress, RENodeId, SubstateId},
};

use state_manager::{DeletedSubstateVersion, LedgerTransactionReceipt};

pub fn to_api_receipt(
    bech32_encoder: &Bech32Encoder,
    receipt: LedgerTransactionReceipt,
) -> Result<models::TransactionReceipt, MappingError> {
    let fee_summary = receipt.fee_summary;

    let (status, output, error_message) = match receipt.outcome {
        TransactionOutcome::Success(output) => {
            (models::TransactionStatus::Succeeded, Some(output), None)
        }
        TransactionOutcome::Failure(error) => (
            models::TransactionStatus::Failed,
            None,
            Some(format!("{:?}", error)),
        ),
    };

    let substate_changes = receipt.substate_changes;

    let created = substate_changes
        .created
        .into_iter()
        .map(|substate_kv| to_api_new_substate_version(bech32_encoder, substate_kv))
        .collect::<Result<Vec<_>, _>>()?;

    let updated = substate_changes
        .updated
        .into_iter()
        .map(|substate_kv| to_api_new_substate_version(bech32_encoder, substate_kv))
        .collect::<Result<Vec<_>, _>>()?;

    let deleted = substate_changes
        .deleted
        .into_iter()
        .map(to_api_deleted_substate)
        .collect::<Result<Vec<_>, _>>()?;

    let new_global_entities = created
        .iter()
        .filter_map(|substate| {
            substate.substate_data.as_ref().and_then(|data| match data {
                models::Substate::GlobalAddressSubstate { target_entity } => {
                    Some(target_entity.as_ref().clone())
                }
                _ => None,
            })
        })
        .collect::<Vec<_>>();

    let api_state_updates = models::StateUpdates {
        created_substates: created,
        updated_substates: updated,
        deleted_substates: deleted,
        new_global_entities,
    };

    let api_fee_summary = to_api_fee_summary(bech32_encoder, fee_summary)?;

    let api_output = match output {
        Some(output) => Some(
            output
                .into_iter()
                .map(|line_output| scrypto_bytes_to_api_sbor_data(bech32_encoder, &line_output))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };

    Ok(models::TransactionReceipt {
        status,
        fee_summary: Box::new(api_fee_summary),
        state_updates: Box::new(api_state_updates),
        output: api_output,
        error_message,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_new_substate_version(
    bech32_encoder: &Bech32Encoder,
    (substate_id, output_value): (SubstateId, OutputValue),
) -> Result<models::NewSubstateVersion, MappingError> {
    let substate_bytes =
        scrypto_encode(&output_value.substate).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Substate bytes could not be encoded".to_string(),
        })?;
    let hash = to_hex(hash(&substate_bytes));

    let api_substate_data = Some(to_api_substate(
        &substate_id,
        &output_value.substate,
        bech32_encoder,
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
pub fn to_api_fee_summary(
    bech32_encoder: &Bech32Encoder,
    fee_summary: FeeSummary,
) -> Result<models::FeeSummary, MappingError> {
    Ok(models::FeeSummary {
        cost_unit_price: to_api_decimal(&fee_summary.cost_unit_price),
        tip_percentage: to_api_u8_as_i32(fee_summary.tip_percentage),
        cost_unit_limit: to_api_u32_as_i64(fee_summary.cost_unit_limit),
        cost_units_consumed: to_api_u32_as_i64(fee_summary.cost_unit_consumed),
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
                            vault_entity: Box::new(to_entity_reference(RENodeId::Vault(vault_id))?),
                            xrd_amount: to_api_decimal(&amount),
                        })
                    })
                    .collect::<Result<_, _>>()
            })
            .transpose()?,
        cost_unit_execution_breakdown: fee_summary
            .execution_cost_unit_breakdown
            .into_iter()
            .map(|(key, cost_unit_amount)| (key, to_api_u32_as_i64(cost_unit_amount)))
            .collect(),
        cost_unit_royalty_breakdown: fee_summary
            .royalty_cost_unit_breakdown
            .into_iter()
            .map(|(receiver, cost_unit_amount)| {
                let global_address = match receiver {
                    RoyaltyReceiver::Package(address, _) => GlobalAddress::Package(address),
                    RoyaltyReceiver::Component(address, _) => GlobalAddress::Component(address),
                };
                models::RoyaltyPayment {
                    royalty_receiver: Box::new(to_global_entity_reference(
                        bech32_encoder,
                        &global_address,
                    )),
                    cost_unit_amount: to_api_u32_as_i64(cost_unit_amount),
                }
            })
            .collect(),
    })
}
