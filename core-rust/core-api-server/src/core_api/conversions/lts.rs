use radix_engine::{
    transaction::BalanceChange,
    types::{Decimal, GlobalAddress, IndexMap, ResourceAddress, RADIX_TOKEN},
};
use state_manager::store::traits::SubstateNodeAncestryStore;
use state_manager::store::StateManagerDatabase;
use state_manager::{
    CommittedTransactionIdentifiers, LedgerTransactionOutcome, LocalTransactionReceipt,
    StateVersion, SubstateChange, TransactionTreeHash,
};
use std::ops::SubAssign;
use transaction::prelude::*;

use crate::core_api::*;

#[tracing::instrument(skip_all)]
pub fn to_api_lts_committed_transaction_outcome(
    database: &StateManagerDatabase,
    context: &MappingContext,
    state_version: StateVersion,
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
        state_version: to_api_state_version(state_version)?,
        accumulator_hash: to_lts_api_accumulator_hash(
            &identifiers.resultant_ledger_hashes.transaction_root,
        ),
        user_transaction_identifiers: identifiers.payload.typed.user().map(|hashes| {
            Box::new(models::TransactionIdentifiers {
                intent_hash: to_api_intent_hash(hashes.intent_hash),
                signed_intent_hash: to_api_signed_intent_hash(hashes.signed_intent_hash),
                payload_hash: to_api_notarized_transaction_hash(hashes.notarized_transaction_hash),
            })
        }),
        status,
        fungible_entity_balance_changes: to_api_lts_fungible_balance_changes(
            database,
            context,
            &receipt.local_execution.fee_payments,
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
    database: &StateManagerDatabase,
    context: &MappingContext,
    fee_payments: &IndexMap<NodeId, Decimal>,
    balance_changes: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
) -> Result<Vec<models::LtsEntityFungibleBalanceChanges>, MappingError> {
    let mut fee_balance_changes = index_map_new();
    let records = database.batch_get_ancestry(fee_payments.keys());
    for (paid_fee_amount_xrd, ancestry) in fee_payments.values().zip(records) {
        let ancestry = ancestry.expect("a vault must be owned by an account");
        let account_address = GlobalAddress::new_or_panic(ancestry.root.0 .0);
        fee_balance_changes
            .entry(account_address)
            .or_insert_with(Decimal::zero)
            .sub_assign(*paid_fee_amount_xrd);
    }

    Ok(balance_changes
        .into_iter()
        .map(|(account_address, changes_by_resource)| {
            let fee_balance_change = fee_balance_changes.get(account_address);
            Ok(models::LtsEntityFungibleBalanceChanges {
                entity_address: to_api_global_address(context, account_address)?,
                fee_balance_change: fee_balance_change
                    .map(|amount_xrd| {
                        Ok(Box::new(models::LtsFungibleResourceBalanceChange {
                            resource_address: to_api_resource_address(context, &RADIX_TOKEN)?,
                            balance_change: to_api_decimal(amount_xrd),
                        }))
                    })
                    .transpose()?,
                non_fee_balance_changes: changes_by_resource
                    .iter()
                    .filter_map(|(resource_address, balance_change)| {
                        get_fungible_balance(balance_change)
                            .map(|balance_change| {
                                let fee_balance_change = *fee_balance_change
                                    .filter(|_| resource_address == &RADIX_TOKEN)
                                    .unwrap_or(&Decimal::ZERO);
                                balance_change - fee_balance_change
                            })
                            .filter(|maybe_zeroed_change| maybe_zeroed_change != &Decimal::ZERO)
                            .map(|non_fee_balance_change| {
                                to_api_lts_fungible_resource_balance_change(
                                    context,
                                    resource_address,
                                    &non_fee_balance_change,
                                )
                            })
                    })
                    .collect::<Result<_, MappingError>>()?,
            })
        })
        .collect::<Result<Vec<_>, MappingError>>())?
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

/// Retrofits the given transaction root, pretending it is an accumulator hash (for LTS purposes).
/// The transaction root and accumulator hash encode the same information and have the same
/// properties - only their computation differs.
fn to_lts_api_accumulator_hash(transaction_root: &TransactionTreeHash) -> String {
    to_hex(transaction_root)
}
