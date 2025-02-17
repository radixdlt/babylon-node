use crate::prelude::*;

#[tracing::instrument(skip_all)]
pub fn to_api_lts_committed_transaction_outcome(
    database: &StateManagerDatabase<impl ReadableRocks>,
    context: &MappingContext,
    state_version: StateVersion,
    receipt: LocalTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::LtsCommittedTransactionOutcome, MappingError> {
    let status = match receipt.on_ledger.outcome {
        LedgerTransactionOutcome::Success => models::LtsCommittedTransactionStatus::Success,
        LedgerTransactionOutcome::Failure => models::LtsCommittedTransactionStatus::Failure,
    };

    let local_execution = &receipt.local_execution;
    Ok(models::LtsCommittedTransactionOutcome {
        state_version: to_api_state_version(state_version)?,
        proposer_timestamp_ms: identifiers.proposer_timestamp_ms,
        accumulator_hash: to_lts_api_accumulator_hash(
            &identifiers.resultant_ledger_hashes.transaction_root,
        ),
        user_transaction_identifiers: identifiers
            .transaction_hashes
            .as_user()
            .map(|hashes| {
                Ok(Box::new(models::TransactionIdentifiers {
                    intent_hash: to_api_transaction_intent_hash(&hashes.transaction_intent_hash),
                    intent_hash_bech32m: to_api_hash_bech32m(
                        context,
                        &hashes.transaction_intent_hash,
                    )?,
                    signed_intent_hash: to_api_signed_transaction_intent_hash(
                        &hashes.signed_transaction_intent_hash,
                    ),
                    signed_intent_hash_bech32m: to_api_hash_bech32m(
                        context,
                        &hashes.signed_transaction_intent_hash,
                    )?,
                    payload_hash: to_api_notarized_transaction_hash(
                        &hashes.notarized_transaction_hash,
                    ),
                    payload_hash_bech32m: to_api_hash_bech32m(
                        context,
                        &hashes.notarized_transaction_hash,
                    )?,
                }))
            })
            .transpose()?,
        status,
        fungible_entity_balance_changes: to_api_lts_fungible_balance_changes(
            database,
            context,
            &local_execution.fee_summary,
            &local_execution.fee_source,
            &local_execution.fee_destination,
            &local_execution
                .global_balance_summary
                .global_balance_changes,
        )?,
        non_fungible_entity_balance_changes: to_api_lts_entity_non_fungible_balance_changes(
            context,
            &local_execution
                .global_balance_summary
                .global_balance_changes,
        )?,
        resultant_account_fungible_balances: to_api_lts_resultant_account_fungible_balances(
            context,
            &local_execution
                .global_balance_summary
                .resultant_fungible_account_balances,
        )?,
        total_fee: to_api_decimal(&local_execution.fee_summary.total_cost()),
    })
}

pub fn to_api_lts_entity_non_fungible_balance_changes(
    context: &MappingContext,
    global_balance_summary: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
) -> Result<Vec<models::LtsEntityNonFungibleBalanceChanges>, MappingError> {
    let mut changes = Vec::new();
    for (address, balance_changes) in global_balance_summary.iter() {
        for (resource, balance_change) in balance_changes.iter() {
            match balance_change {
                BalanceChange::Fungible(_) => {}
                BalanceChange::NonFungible { added, removed } => {
                    changes.push(models::LtsEntityNonFungibleBalanceChanges {
                        entity_address: to_api_global_address(context, address)?,
                        resource_address: to_api_resource_address(context, resource)?,
                        added: added
                            .iter()
                            .map(|non_fungible_id| non_fungible_id.to_string())
                            .collect(),
                        removed: removed
                            .iter()
                            .map(|non_fungible_id| non_fungible_id.to_string())
                            .collect(),
                    });
                }
            }
        }
    }
    Ok(changes)
}

pub fn to_api_lts_fungible_balance_changes(
    database: &StateManagerDatabase<impl ReadableRocks>,
    context: &MappingContext,
    fee_summary: &TransactionFeeSummary,
    fee_source: &FeeSource,
    fee_destination: &FeeDestination,
    balance_changes: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
) -> Result<Vec<models::LtsEntityFungibleBalanceChanges>, MappingError> {
    let fee_balance_changes = resolve_global_fee_balance_changes(database, fee_source)?;
    let fee_payment_computations = FeePaymentComputer::compute(FeePaymentComputationInputs {
        fee_balance_changes,
        fee_summary,
        fee_destination,
        balance_changes,
    });
    resolve_fungible_balance_changes(&fee_payment_computations, context)
}

/// Uses the [`SubstateNodeAncestryStore`] (from the given DB) to transform the input
/// `vault ID -> payment` map into a `global address -> balance change` map.
fn resolve_global_fee_balance_changes(
    database: &StateManagerDatabase<impl ReadableRocks>,
    fee_source: &FeeSource,
) -> Result<IndexMap<GlobalAddress, Decimal>, MappingError> {
    let paying_vaults = &fee_source.paying_vaults;
    let ancestries = database.batch_get_ancestry(paying_vaults.keys());
    let mut fee_balance_changes = index_map_new();
    for ((vault_id, paid_fee_amount_xrd), ancestry) in paying_vaults.iter().zip(ancestries) {
        let ancestry = ancestry.ok_or_else(|| MappingError::InternalIndexDataMismatch {
            message: format!("no ancestry record for vault {}", vault_id.to_hex()),
        })?;
        let global_ancestor_address = GlobalAddress::new_or_panic(ancestry.root.0.into());
        let fee_balance_change = fee_balance_changes
            .entry(global_ancestor_address)
            .or_insert_with(Decimal::zero);
        *fee_balance_change = fee_balance_change.sub_or_panic(*paid_fee_amount_xrd);
    }
    Ok(fee_balance_changes)
}

fn to_api_lts_fee_fungible_resource_balance_change_type(
    fee_type: &FeePaymentBalanceChangeType,
) -> models::LtsFeeFungibleResourceBalanceChangeType {
    match fee_type {
        FeePaymentBalanceChangeType::FeePayment => {
            models::LtsFeeFungibleResourceBalanceChangeType::FeePayment
        }
        FeePaymentBalanceChangeType::FeeDistributed => {
            models::LtsFeeFungibleResourceBalanceChangeType::FeeDistributed
        }
        FeePaymentBalanceChangeType::TipDistributed => {
            models::LtsFeeFungibleResourceBalanceChangeType::TipDistributed
        }
        FeePaymentBalanceChangeType::RoyaltyDistributed => {
            models::LtsFeeFungibleResourceBalanceChangeType::RoyaltyDistributed
        }
    }
}

fn resolve_fungible_balance_changes(
    fee_payment_computation: &FeePaymentComputation,
    context: &MappingContext,
) -> Result<Vec<models::LtsEntityFungibleBalanceChanges>, MappingError> {
    let mut output = Vec::with_capacity(fee_payment_computation.relevant_entities.len());
    for entity in &fee_payment_computation.relevant_entities {
        // First - calculate the deprecated/duplicated total balance change
        let deprecated_fee_payment_balance_change = fee_payment_computation
            .fee_balance_changes
            .get(entity)
            .map(|fee_changes| {
                let total_fee_payment_balance_change = fee_changes
                    .iter()
                    .filter_map(|fee_balance_change| match &fee_balance_change.0 {
                        FeePaymentBalanceChangeType::FeePayment => Some(fee_balance_change.1),
                        _ => None,
                    })
                    .sum_or_panic();
                let output = if total_fee_payment_balance_change.is_zero() {
                    None
                } else {
                    Some(Box::new(to_api_lts_fungible_resource_balance_change(
                        context,
                        &XRD,
                        &total_fee_payment_balance_change,
                    )?))
                };
                Ok(output)
            })
            .transpose()?
            .flatten();
        output.push(models::LtsEntityFungibleBalanceChanges {
            entity_address: to_api_global_address(context, entity)?,
            fee_balance_change: deprecated_fee_payment_balance_change,
            fee_balance_changes: fee_payment_computation
                .fee_balance_changes
                .get(entity)
                .map(|fee_balance_changes| {
                    fee_balance_changes
                        .iter()
                        .map(
                            |(fee_change_type, balance_change)| -> Result<_, MappingError> {
                                Ok(models::LtsFeeFungibleResourceBalanceChange {
                                    resource_address: to_api_resource_address(context, &XRD)?,
                                    balance_change: to_api_decimal(balance_change),
                                    _type: to_api_lts_fee_fungible_resource_balance_change_type(
                                        fee_change_type,
                                    ),
                                })
                            },
                        )
                        .collect::<Result<_, _>>()
                })
                .transpose()?
                .unwrap_or_default(),
            non_fee_balance_changes: fee_payment_computation
                .non_fee_balance_changes
                .get(entity)
                .map(|non_fee_balance_changes| {
                    non_fee_balance_changes
                        .iter()
                        .map(|(resource_address, amount)| {
                            to_api_lts_fungible_resource_balance_change(
                                context,
                                resource_address,
                                amount,
                            )
                        })
                        .collect::<Result<_, _>>()
                })
                .transpose()?
                .unwrap_or_default(),
        });
    }
    Ok(output)
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
    context: &MappingContext,
    fungible_account_balances: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, Decimal>>,
) -> Result<Vec<models::LtsResultantAccountFungibleBalances>, MappingError> {
    fungible_account_balances
        .iter()
        .map(|(account_address, resource_balances)| {
            Ok(models::LtsResultantAccountFungibleBalances {
                account_address: to_api_global_address(context, account_address)?,
                resultant_balances: resource_balances
                    .iter()
                    .map(|(resource_address, resultant_balance)| {
                        Ok(models::LtsResultantFungibleBalance {
                            resource_address: to_api_resource_address(context, resource_address)?,
                            resultant_balance: to_api_decimal(resultant_balance),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect()
}

/// Retrofits the given transaction root, pretending it is an accumulator hash (for LTS purposes).
/// The transaction root and accumulator hash encode the same information and have the same
/// properties - only their computation differs.
fn to_lts_api_accumulator_hash(transaction_root: &TransactionTreeHash) -> String {
    to_hex(transaction_root)
}
