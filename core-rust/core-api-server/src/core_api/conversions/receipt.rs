use crate::core_api::conversions::common::{to_hex, to_sbor_hex};
use crate::core_api::conversions::to_api_substate;
use crate::core_api::generated::models;
use radix_engine::fee::FeeSummary;
use scrypto::address::Bech32Encoder;
use state_manager::{CommittedTransactionStatus, LedgerTransactionReceipt};

pub fn to_api_receipt(
    bech32_encoder: &Bech32Encoder,
    receipt: LedgerTransactionReceipt,
) -> models::TransactionReceipt {
    let fee_summary = receipt.fee_summary;
    let entity_changes = receipt.entity_changes;

    let (status, output, error_message) = match receipt.status {
        CommittedTransactionStatus::Success(output) => {
            let output_hex: Vec<String> = output.into_iter().map(to_hex).collect();
            (models::TransactionStatus::SUCCEEDED, Some(output_hex), None)
        }
        CommittedTransactionStatus::Failure(error) => {
            (models::TransactionStatus::FAILED, None, Some(error))
        }
    };

    let state_updates = receipt.state_updates;

    let api_state_updates = models::StateUpdates {
        down_virtual_substates: state_updates
            .down_virtual_substates
            .into_iter()
            .map(|v| to_sbor_hex(&v).into())
            .collect(),
        up_substates: state_updates
            .up_substates
            .into_iter()
            .map(|(substate_id, output_value)| models::UpSubstate {
                substate_id: to_sbor_hex(&substate_id),
                version: output_value.version.to_string(),
                substate_bytes: to_sbor_hex(&output_value.substate),
                substate: to_api_substate(&output_value.substate, bech32_encoder),
            })
            .collect(),
        down_substates: state_updates
            .down_substates
            .into_iter()
            .map(|v| models::DownSubstate {
                substate_id: to_sbor_hex(&v.substate_id),
                substate_hash: v.substate_hash.to_string(),
                version: v.version.to_string(),
            })
            .collect(),
        new_roots: state_updates
            .new_roots
            .into_iter()
            .map(|v| to_sbor_hex(&v).into())
            .collect(),
    };

    let api_fee_summary = to_api_fee_summary(fee_summary);

    models::TransactionReceipt {
        status,
        fee_summary: api_fee_summary,
        state_updates: api_state_updates,
        new_package_addresses: entity_changes
            .new_package_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_package_address(&v).into())
            .collect(),
        new_component_addresses: entity_changes
            .new_component_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_component_address(&v).into())
            .collect(),
        new_resource_addresses: entity_changes
            .new_resource_addresses
            .into_iter()
            .map(|v| bech32_encoder.encode_resource_address(&v).into())
            .collect(),
        output,
        error_message,
    }
}

pub fn to_api_fee_summary(fee_summary: FeeSummary) -> models::FeeSummary {
    models::FeeSummary {
        loan_fully_repaid: fee_summary.loan_fully_repaid,
        cost_unit_limit: fee_summary.cost_unit_limit.to_string(),
        cost_unit_consumed: fee_summary.cost_unit_consumed.to_string(),
        cost_unit_price: fee_summary.cost_unit_price.to_string(),
        tip_percentage: fee_summary.tip_percentage.to_string(),
        xrd_burned: fee_summary.burned.to_string(),
        xrd_tipped: fee_summary.tipped.to_string(),
    }
}
