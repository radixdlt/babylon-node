use radix_engine::{
    transaction::BalanceChange,
    types::{Decimal, GlobalAddress, IndexMap, ResourceAddress, RADIX_TOKEN},
};
use state_manager::{
    transaction::LedgerTransaction, CommittedTransactionIdentifiers, LedgerTransactionOutcome,
    LocalTransactionReceipt, SubstateChange,
};
use transaction::prelude::*;

use crate::core_api::*;

#[tracing::instrument(skip_all)]
pub fn to_api_lts_committed_transaction_outcome(
    context: &MappingContext,
    transaction: LedgerTransaction,
    receipt: LocalTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::LtsCommittedTransactionOutcome, MappingError> {
    let status = match receipt.on_ledger.outcome {
        LedgerTransactionOutcome::Success => models::LtsCommittedTransactionStatus::Success,
        LedgerTransactionOutcome::Failure => models::LtsCommittedTransactionStatus::Failure,
    };

    // TODO: add total tip payed to validator when it is implemented
    let total_fee = receipt.local_execution.fee_summary.total_royalty_cost_xrd
        + receipt.local_execution.fee_summary.total_execution_cost_xrd;

    Ok(models::LtsCommittedTransactionOutcome {
        state_version: to_api_state_version(identifiers.at_commit.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.at_commit.accumulator_hash),
        user_transaction_identifiers: identifiers.payload.typed.user().map(|hashes| {
            Box::new(models::TransactionIdentifiers {
                intent_hash: to_api_intent_hash(&hashes.0),
                signed_intent_hash: to_api_signed_intent_hash(&hashes.1),
                payload_hash: to_api_notarized_transaction_hash(&hashes.2),
            })
        }),
        status,
        fungible_entity_balance_changes: to_api_lts_fungible_balance_changes(
            context,
            total_fee,
            &receipt.local_execution.state_update_summary.balance_changes,
        )?,
        resultant_account_fungible_balances: to_api_lts_resultant_account_fungible_balances(
            context,
            &receipt.local_execution.state_update_summary.balance_changes,
            &receipt.on_ledger.substate_changes,
        ),
        total_fee: to_api_decimal(&total_fee),
    })
}

pub fn to_api_lts_fungible_balance_changes(
    context: &MappingContext,
    total_fee: Decimal,
    balance_changes: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
) -> Result<Vec<models::LtsEntityFungibleBalanceChanges>, MappingError> {
    // TODO - until we have the proper information from the engine, we need to do some guessing here about which entities
    // actually paid the fee. Ideally our guessing will align with 99% of transactions - and we can make clear in the docs
    // it's not fully accurate for now but will be for launch.

    // For now let's assume: The fee is paid by a single entity, and that entity is either:
    // - The first entity to be debitted the exact fee, or if that doesn't exist:
    // - The entity with the largest XRD debit.

    let total_fee_balance_change = -total_fee;

    let mut net_xrd_balance_change = Decimal::ZERO;
    let mut exact_fee_debit = Option::<GlobalAddress>::None;
    let mut biggest_xrd_debit = Option::<(Decimal, GlobalAddress)>::None;

    for (entity_address, resource_changes) in balance_changes.iter() {
        for (resource_address, balance_change) in resource_changes {
            if *resource_address == RADIX_TOKEN {
                let balance_change = get_fungible_balance(balance_change).unwrap();
                if balance_change == total_fee_balance_change && exact_fee_debit.is_none() {
                    exact_fee_debit = Some(*entity_address);
                }
                if biggest_xrd_debit.is_none() || balance_change < biggest_xrd_debit.unwrap().0 {
                    biggest_xrd_debit = Some((balance_change, *entity_address));
                }
                net_xrd_balance_change += balance_change;
            }
        }
    }

    let (assumed_fee_payer, assumed_fee_balance_change) = match (
        exact_fee_debit,
        biggest_xrd_debit,
        total_fee_balance_change == net_xrd_balance_change,
    ) {
        // If an entity debited the exact fee - it's probably that entity
        // - This covers the case where entity X paid the fee but didn't otherwise transfer XRD
        (Some(entity_address), _, true) => (Some(entity_address), total_fee_balance_change),
        // Else use the entity that debitted the most XRD - who is most likely to be the fee payer
        // - This is accurate in the case where someone transferred XRD from their account and paid the fee
        (None, Some((_, entity_address)), true) => (Some(entity_address), total_fee_balance_change),
        // If there's been no XRD debit, ot it doesn't equal the total fee, then we should be at genesis or in an end of
        // epoch scenario without a fee payer
        _ => (None, Decimal::ZERO),
    };

    balance_changes
        .iter()
        .map(|(entity_address, resource_changes)| {
            let changes = if assumed_fee_payer == Some(*entity_address) {
                models::LtsEntityFungibleBalanceChanges {
                    entity_address: to_api_global_address(context, entity_address)?,
                    fee_balance_change: Some(Box::new(models::LtsFungibleResourceBalanceChange {
                        resource_address: to_api_resource_address(context, &RADIX_TOKEN)?,
                        balance_change: to_api_decimal(&assumed_fee_balance_change),
                    })),
                    non_fee_balance_changes: resource_changes
                        .iter()
                        .filter_map(|(resource_address, balance_change)| {
                            if *resource_address == RADIX_TOKEN {
                                let fungible_balance_change =
                                    get_fungible_balance(balance_change).unwrap();
                                let non_fee_balance_change =
                                    fungible_balance_change - assumed_fee_balance_change;
                                if non_fee_balance_change == Decimal::ZERO {
                                    return None;
                                }
                                return Some(to_api_lts_fungible_resource_balance_change(
                                    context,
                                    resource_address,
                                    &non_fee_balance_change,
                                ));
                            }
                            match balance_change {
                                BalanceChange::Fungible(balance_change) => {
                                    Some(to_api_lts_fungible_resource_balance_change(
                                        context,
                                        resource_address,
                                        balance_change,
                                    ))
                                }
                                BalanceChange::NonFungible { .. } => None,
                            }
                        })
                        .collect::<Result<_, MappingError>>()?,
                }
            } else {
                models::LtsEntityFungibleBalanceChanges {
                    entity_address: to_api_global_address(context, entity_address)?,
                    fee_balance_change: None,
                    non_fee_balance_changes: resource_changes
                        .iter()
                        .filter_map(|(resource_address, balance_change)| match balance_change {
                            BalanceChange::Fungible(balance_change) => {
                                Some(to_api_lts_fungible_resource_balance_change(
                                    context,
                                    resource_address,
                                    balance_change,
                                ))
                            }
                            BalanceChange::NonFungible { .. } => None,
                        })
                        .collect::<Result<_, MappingError>>()?,
                }
            };
            Ok(changes)
        })
        .collect::<Result<Vec<_>, MappingError>>()
}

pub fn to_api_lts_fungible_resource_balance_change(
    context: &MappingContext,
    resource_address: &ResourceAddress,
    balance_change: &Decimal,
) -> Result<models::LtsFungibleResourceBalanceChange, MappingError> {
    Ok(models::LtsFungibleResourceBalanceChange {
        resource_address: to_api_resource_address(context, resource_address)?,
        balance_change: to_api_decimal(balance_change),
    })
}

pub fn to_api_lts_resultant_account_fungible_balances(
    _context: &MappingContext,
    _balance_changes: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
    _substate_changes: &[SubstateChange],
) -> Vec<models::LtsResultantAccountFungibleBalances> {
    // TODO - until we have the proper information from the engine, we need to do some guessing here about
    // how to match up vault changes with balance changes.
    // Also, for release/rcnet-v1 compatibility, we don't save _old_ state when we update substates.
    // So we can't even compare diffs.
    // So let's just give up and say in the docs that it'll be coming later.
    vec![]
}

pub fn get_fungible_balance(balance_change: &BalanceChange) -> Option<Decimal> {
    match balance_change {
        BalanceChange::Fungible(balance_change) => Some(*balance_change),
        BalanceChange::NonFungible { .. } => None,
    }
}
