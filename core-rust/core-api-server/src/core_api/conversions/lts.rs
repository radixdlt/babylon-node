use radix_engine::transaction::BalanceChange;
use state_manager::{
    CommittedTransactionIdentifiers, LedgerTransactionOutcome, LedgerTransactionReceipt,
};

use crate::core_api::*;

#[tracing::instrument(skip_all)]
pub fn to_api_lts_comitted_transaction_basic_outcome(
    context: &MappingContext,
    receipt: LedgerTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::LtsCommittedTransactionOutcome, MappingError> {
    let status = match receipt.outcome {
        LedgerTransactionOutcome::Success(_) => models::LtsCommittedTransactionStatus::Succeeded,
        LedgerTransactionOutcome::Failure(_) => models::LtsCommittedTransactionStatus::Failed,
    };

    let fungible_entity_balance_changes = receipt
        .state_update_summary
        .balance_changes
        .iter()
        .map(
            |(address, resource_changes)| models::LtsEntityFungibleBalanceChanges {
                address: to_api_address(context, address),
                fungible_resource_balance_changes: resource_changes
                    .iter()
                    .filter_map(|(resource_address, balance_change)| match balance_change {
                        BalanceChange::Fungible(balance_change) => {
                            Some(models::LtsFungibleResourceBalanceChange {
                                fungible_resource_address: to_api_resource_address(
                                    context,
                                    resource_address,
                                ),
                                balance_change: to_api_decimal(balance_change),
                            })
                        }
                        BalanceChange::NonFungible { .. } => None,
                    })
                    .collect(),
            },
        )
        .collect();

    // TODO: add total tip payed to validator when it is implemented
    let fee =
        receipt.fee_summary.total_royalty_cost_xrd + receipt.fee_summary.total_execution_cost_xrd;

    Ok(models::LtsCommittedTransactionOutcome {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
        status,
        fungible_entity_balance_changes,
        fee: to_api_decimal(&fee),
    })
}
