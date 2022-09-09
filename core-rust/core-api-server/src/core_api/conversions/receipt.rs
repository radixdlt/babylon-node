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
    let entity_changes = receipt.entity_changes;

    let (status, output, error_message) = match receipt.status {
        CommittedTransactionStatus::Success(output) => {
            let output_hex: Vec<String> = output.into_iter().map(to_hex).collect();
            (TransactionStatus::Succeeded, Some(output_hex), None)
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

    // These should be entity ids, not substate ids
    let new_global_entities = state_updates
        .new_roots
        .into_iter()
        .map(|x| to_api_global_entity_id(bech32_encoder, x))
        .collect::<Result<Vec<_>, _>>()?;

    let api_state_updates = StateUpdates {
        up_substates,
        down_substates,
        down_virtual_substates,
        new_global_entities,
    };

    let api_fee_summary = to_api_fee_summary(fee_summary);

    Ok(TransactionReceipt {
        status,
        fee_summary: Box::new(api_fee_summary),
        state_updates: Box::new(api_state_updates),
        new_package_addresses: entity_changes
            .new_package_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_package_address(&v))
            .collect(),
        new_component_addresses: entity_changes
            .new_component_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_component_address(&v))
            .collect(),
        new_resource_addresses: entity_changes
            .new_resource_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_resource_address(&v))
            .collect(),
        output,
        error_message,
    })
}

pub fn to_api_up_substate(
    bech32_encoder: &Bech32Encoder,
    (substate_id, output_value): (SubstateId, OutputValue),
) -> Result<UpSubstate, MappingError> {
    let substate_bytes = scrypto_encode(&output_value.substate);
    let hash = to_hex(hash(&substate_bytes));
    Ok(UpSubstate {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        version: output_value.version.to_string(),
        substate_bytes: to_hex(substate_bytes),
        substate_data_hash: hash,
        substate_data: Option::Some(to_api_substate(&output_value.substate, bech32_encoder)?),
    })
}

pub fn to_api_down_substate(output_id: OutputId) -> Result<DownSubstate, MappingError> {
    Ok(DownSubstate {
        substate_id: Box::new(to_api_substate_id(output_id.substate_id)?),
        substate_data_hash: to_hex(output_id.substate_hash),
        version: output_id
            .version
            .try_into()
            .map_err(|err| MappingError::Integer {
                message: "Substate version could not be mapped to i32".to_owned(),
                error: err,
            })?,
    })
}

pub fn to_api_down_virtual_substate(
    VirtualSubstateId(root_substate_id, key): VirtualSubstateId,
) -> Result<models::SubstateId, MappingError> {
    to_api_virtual_substate_id(root_substate_id, key)
}

pub fn to_api_fee_summary(fee_summary: EngineFeeSummary) -> FeeSummary {
    FeeSummary {
        loan_fully_repaid: fee_summary.loan_fully_repaid,
        cost_unit_limit: fee_summary.cost_unit_limit.to_string(),
        cost_unit_consumed: fee_summary.cost_unit_consumed.to_string(),
        cost_unit_price: fee_summary.cost_unit_price.to_string(),
        tip_percentage: fee_summary.tip_percentage.to_string(),
        xrd_burned: fee_summary.burned.to_string(),
        xrd_tipped: fee_summary.tipped.to_string(),
    }
}
