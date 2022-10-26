use super::addressing::*;
use crate::core_api::models::*;
use crate::core_api::*;
use radix_engine::{
    fee::FeeSummary as EngineFeeSummary,
    ledger::{OutputId, OutputValue},
    state_manager::VirtualSubstateId,
    types::{hash, scrypto_encode, SubstateId},
};
use scrypto::address::Bech32Encoder;

use state_manager::{CommittedTransactionStatus, LedgerTransactionReceipt};

pub fn to_api_receipt(
    bech32_encoder: &Bech32Encoder,
    receipt: LedgerTransactionReceipt,
) -> Result<TransactionReceipt, MappingError> {
    let fee_summary = receipt.fee_summary;

    let (status, output, error_message) = match receipt.status {
        CommittedTransactionStatus::Success(output) => {
            (TransactionStatus::Succeeded, Some(output), None)
        }
        CommittedTransactionStatus::Failure(error) => {
            (TransactionStatus::Failed, None, Some(error))
        }
    };

    let state_updates = receipt.state_updates;

    let up_substates = state_updates
        .up_substates
        .into_iter()
        .map(|substate_kv| to_api_up_substate(bech32_encoder, substate_kv))
        .collect::<Result<Vec<_>, _>>()?;

    let down_substates = state_updates
        .down_substates
        .into_iter()
        .map(to_api_down_substate)
        .collect::<Result<Vec<_>, _>>()?;

    let down_virtual_substates = state_updates
        .down_virtual_substates
        .into_iter()
        .map(to_api_down_virtual_substate)
        .collect::<Result<Vec<_>, _>>()?;

    let new_global_entities: Vec<GlobalEntityId> = up_substates
        .iter()
        .filter_map(|substate| {
            substate.substate_data.as_ref().and_then(|data| match data {
                Substate::GlobalSubstate {
                    entity_type: _,
                    target_entity,
                } => Some(target_entity.as_ref().clone()),
                _ => None,
            })
        })
        .collect();

    let api_state_updates = StateUpdates {
        up_substates,
        down_substates,
        down_virtual_substates,
        new_global_entities,
    };

    let api_fee_summary = to_api_fee_summary(fee_summary);

    let api_output = match output {
        Some(output) => Some(
            output
                .into_iter()
                .map(|line_output| scrypto_bytes_to_api_sbor_data(bech32_encoder, &line_output))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };

    Ok(TransactionReceipt {
        status,
        fee_summary: Box::new(api_fee_summary),
        state_updates: Box::new(api_state_updates),
        output: api_output,
        error_message,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_up_substate(
    bech32_encoder: &Bech32Encoder,
    (substate_id, output_value): (SubstateId, OutputValue),
) -> Result<UpSubstate, MappingError> {
    let substate_bytes = scrypto_encode(&output_value.substate);
    let hash = to_hex(hash(&substate_bytes));

    let api_substate_data = Some(to_api_substate(
        &substate_id,
        &output_value.substate,
        bech32_encoder,
    )?);

    Ok(UpSubstate {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        version: to_api_substate_version(output_value.version)?,
        substate_hex: to_hex(substate_bytes),
        substate_data_hash: hash,
        substate_data: api_substate_data,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_down_substate(output_id: OutputId) -> Result<DownSubstate, MappingError> {
    Ok(DownSubstate {
        substate_id: Box::new(to_api_substate_id(output_id.substate_id)?),
        substate_data_hash: to_hex(output_id.substate_hash),
        version: to_api_substate_version(output_id.version)?,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_down_virtual_substate(
    VirtualSubstateId(root_substate_id, key): VirtualSubstateId,
) -> Result<models::SubstateId, MappingError> {
    to_api_virtual_substate_id(root_substate_id, key)
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_summary(fee_summary: EngineFeeSummary) -> FeeSummary {
    FeeSummary {
        loan_fully_repaid: fee_summary.loan_fully_repaid,
        cost_unit_limit: to_api_u32_as_i64(fee_summary.cost_unit_limit),
        cost_unit_consumed: to_api_u32_as_i64(fee_summary.cost_unit_consumed),
        cost_unit_price_attos: to_api_decimal_attos(&fee_summary.cost_unit_price),
        tip_percentage: to_api_u32_as_i64(fee_summary.tip_percentage),
        xrd_burned_attos: to_api_decimal_attos(&fee_summary.burned),
        xrd_tipped_attos: to_api_decimal_attos(&fee_summary.tipped),
    }
}
