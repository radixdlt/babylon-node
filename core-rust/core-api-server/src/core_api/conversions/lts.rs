use radix_engine::{
    system::system_modules::costing::{FeeSummary, RoyaltyRecipient},
    transaction::BalanceChange,
    types::{Decimal, GlobalAddress, IndexMap, ResourceAddress},
};
use state_manager::store::traits::SubstateNodeAncestryStore;
use state_manager::store::StateManagerDatabase;
use state_manager::{
    BySubstate, ChangeAction, CommittedTransactionIdentifiers, LedgerTransactionOutcome,
    LocalTransactionReceipt, StateVersion, TransactionTreeHash,
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

    let total_fee = receipt.local_execution.fee_summary.total_royalty_cost_xrd
        + receipt.local_execution.fee_summary.total_execution_cost_xrd
        + receipt
            .local_execution
            .fee_summary
            .total_state_expansion_cost_xrd
        + receipt.local_execution.fee_summary.total_tipping_cost_xrd;

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
            &receipt.local_execution.fee_summary,
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
    fee_summary: &FeeSummary,
    balance_changes: &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
) -> Result<Vec<models::LtsEntityFungibleBalanceChanges>, MappingError> {
    let fee_balance_changes =
        resolve_global_fee_balance_changes(database, &fee_summary.fee_payments)?;
    FeePaymentComputer::compute(FeePaymentComputationInputs {
        fee_balance_changes,
        fee_summary,
        balance_changes,
    })
    .resolve_fungible_balance_changes(context)
}

/// Uses the [`SubstateNodeAncestryStore`] (from the given DB) to transform the input
/// `vault ID -> payment` map into a `global address -> balance change` map.
fn resolve_global_fee_balance_changes(
    database: &StateManagerDatabase,
    fee_payments: &IndexMap<NodeId, Decimal>,
) -> Result<IndexMap<GlobalAddress, Decimal>, MappingError> {
    let ancestries = database.batch_get_ancestry(fee_payments.keys());
    let mut fee_balance_changes = index_map_new();
    for ((vault_id, paid_fee_amount_xrd), ancestry) in fee_payments.iter().zip(ancestries) {
        let ancestry = ancestry.ok_or_else(|| MappingError::InternalIndexDataMismatch {
            message: format!("no ancestry record for vault {}", vault_id.to_hex()),
        })?;
        let global_ancestor_address = GlobalAddress::new_or_panic(ancestry.root.0.into());
        fee_balance_changes
            .entry(global_ancestor_address)
            .or_insert_with(Decimal::zero)
            .sub_assign(*paid_fee_amount_xrd);
    }
    Ok(fee_balance_changes)
}

struct FeePaymentComputer<'a> {
    inputs: FeePaymentComputationInputs<'a>,
    computation: FeePaymentComputation,
}

struct FeePaymentComputationInputs<'a> {
    /// The balance changes caused by [`FeeSummary#fee_payments`] (resolved to global ancestors).
    /// Note: this information is logically of the same type as [`balance_changes`], but the actual
    /// signature is simpler, since all fees are necessarily XRD and thus have fungible balances.
    fee_balance_changes: IndexMap<GlobalAddress, Decimal>,
    fee_summary: &'a FeeSummary,
    /// The total balance changes (i.e. including the [`fee_balance_changes`]).
    balance_changes: &'a IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
}

struct FeePaymentComputation {
    relevant_entities: IndexSet<GlobalAddress>,
    fee_balance_changes:
        NonIterMap<GlobalAddress, Vec<(models::LtsFeeFungibleResourceBalanceChangeType, Decimal)>>,
    non_fee_balance_changes: NonIterMap<GlobalAddress, IndexMap<ResourceAddress, Decimal>>,
}

impl<'a> FeePaymentComputer<'a> {
    pub fn compute(inputs: FeePaymentComputationInputs<'a>) -> FeePaymentComputation {
        Self {
            inputs,
            computation: FeePaymentComputation {
                relevant_entities: Default::default(),
                fee_balance_changes: Default::default(),
                non_fee_balance_changes: Default::default(),
            },
        }
        .compute_internal()
    }

    fn compute_internal(mut self) -> FeePaymentComputation {
        // Step 1 - Add initial relevant entities
        self.add_initial_ordered_relevant_entities();

        // Step 2 - Track fee balance changes
        self.track_fee_payments();
        self.track_fee_distributions();
        self.track_royalty_distributions();

        // Step 3 - Compute resultant non-fee balance changes
        self.finalize_non_fee_balance_changes();

        // And finally, return the outputs
        self.computation
    }

    fn add_initial_ordered_relevant_entities(&mut self) {
        for entity in self.inputs.balance_changes.keys() {
            self.computation.relevant_entities.insert(*entity);
        }
    }

    fn track_fee_payments(&mut self) {
        for (fee_payer, fee_balance_change) in self.inputs.fee_balance_changes.clone() {
            self.record_fee_balance_change_if_non_zero(
                fee_payer,
                fee_balance_change,
                models::LtsFeeFungibleResourceBalanceChangeType::FeePayment,
            );
        }
    }

    fn track_fee_distributions(&mut self) {
        // TODO: Improve the fee calculation once the right values are on the FeeSummary
        // - Because "fee distributed" isn't on the fee summary, we have to calculate it instead, by assuming the consensus manager
        //   has no other fee changes
        // - Once "fee distributed" is on the summary, we can use that directly instead
        let tip_distributed = self.inputs.fee_summary.total_tipping_cost_xrd;
        let consensus_manager_xrd_balance_change = self
            .inputs
            .balance_changes
            .get(&GlobalAddress::from(CONSENSUS_MANAGER))
            .and_then(|balance_changes| balance_changes.get(&XRD))
            .map(|balance_change| {
                get_fungible_balance(balance_change).expect("Expected XRD to be fungible")
            })
            .unwrap_or_default();
        let assumed_consensus_manager_non_fee_xrd_balance_change = Decimal::ZERO;
        let fee_distributed = consensus_manager_xrd_balance_change
            - assumed_consensus_manager_non_fee_xrd_balance_change
            - tip_distributed;

        self.record_fee_balance_change_if_non_zero(
            CONSENSUS_MANAGER.into(),
            fee_distributed,
            models::LtsFeeFungibleResourceBalanceChangeType::FeeDistributed,
        );
        self.record_fee_balance_change_if_non_zero(
            CONSENSUS_MANAGER.into(),
            tip_distributed,
            models::LtsFeeFungibleResourceBalanceChangeType::TipDistributed,
        );
    }

    fn track_royalty_distributions(&mut self) {
        for (recipient, (_, amount)) in &self.inputs.fee_summary.royalty_cost_breakdown {
            let recipient: GlobalAddress = match recipient {
                RoyaltyRecipient::Package(address) => (*address).into(),
                RoyaltyRecipient::Component(address) => (*address).into(),
            };
            self.record_fee_balance_change_if_non_zero(
                recipient,
                *amount,
                models::LtsFeeFungibleResourceBalanceChangeType::RoyaltyDistributed,
            );
        }
    }

    fn record_fee_balance_change_if_non_zero(
        &mut self,
        address: GlobalAddress,
        balance_change: Decimal,
        fee_type: models::LtsFeeFungibleResourceBalanceChangeType,
    ) {
        if balance_change == Decimal::ZERO {
            return;
        }
        // This handles the case that a relevant entity had 0 net balance change.
        // For example, if a component received a royalty and output XRD equal to that royalty in the same transaction then
        // it wouldn't be in the balance changes - but we'd still want to include it in our output.
        self.computation.relevant_entities.insert(address);
        self.computation
            .fee_balance_changes
            .entry(address)
            .or_insert_with(Vec::new)
            .push((fee_type, balance_change));
    }

    fn finalize_non_fee_balance_changes(&mut self) {
        for (entity, changes) in self.inputs.balance_changes {
            let total_fee_balance_changes: Decimal = self
                .computation
                .fee_balance_changes
                .get(entity)
                .map(|fee_payments| fee_payments.iter().map(|p| p.1).sum())
                .unwrap_or_default();
            let mut non_fee_balance_changes: IndexMap<ResourceAddress, Decimal> = changes
                .iter()
                .filter_map(|(resource, balance_change)| {
                    if resource == &XRD {
                        let total_balance_change = get_fungible_balance(balance_change)
                            .expect("Expected XRD to be fungible");
                        let total_non_fee_balance_change =
                            total_balance_change - total_fee_balance_changes;
                        if total_non_fee_balance_change == Decimal::ZERO {
                            None
                        } else {
                            Some((*resource, total_non_fee_balance_change))
                        }
                    } else {
                        match balance_change {
                            BalanceChange::Fungible(change) => Some((*resource, *change)),
                            BalanceChange::NonFungible { .. } => None,
                        }
                    }
                })
                .collect();
            if total_fee_balance_changes != Decimal::ZERO && !changes.contains_key(&XRD) {
                // If there were fee-related balance changes, but XRD is not in the balance change set,
                // then there must have been an equal-and-opposite non-fee balance change to offset it
                non_fee_balance_changes.insert(XRD, -total_fee_balance_changes);
            }
            self.computation
                .non_fee_balance_changes
                .insert(*entity, non_fee_balance_changes);
        }
    }
}

impl FeePaymentComputation {
    fn resolve_fungible_balance_changes(
        self,
        context: &MappingContext,
    ) -> Result<Vec<models::LtsEntityFungibleBalanceChanges>, MappingError> {
        let mut output = Vec::with_capacity(self.relevant_entities.len());
        for entity in self.relevant_entities {
            // First - calculate the deprecated/duplicated total balance change
            let deprecated_fee_payment_balance_change = self
                .fee_balance_changes
                .get(&entity)
                .map(|fee_changes| {
                    let total_fee_payment_balance_change = fee_changes
                        .iter()
                        .filter_map(|fee_balance_change| match &fee_balance_change.0 {
                            models::LtsFeeFungibleResourceBalanceChangeType::FeePayment => {
                                Some(fee_balance_change.1)
                            }
                            _ => None,
                        })
                        .sum::<Decimal>();
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
                entity_address: to_api_global_address(context, &entity)?,
                fee_balance_change: deprecated_fee_payment_balance_change,
                fee_balance_changes: self
                    .fee_balance_changes
                    .get(&entity)
                    .map(|fee_balance_changes| {
                        fee_balance_changes
                            .iter()
                            .map(
                                |(fee_change_type, balance_change)| -> Result<_, MappingError> {
                                    Ok(models::LtsFeeFungibleResourceBalanceChange {
                                        resource_address: to_api_resource_address(context, &XRD)?,
                                        balance_change: to_api_decimal(balance_change),
                                        _type: *fee_change_type,
                                    })
                                },
                            )
                            .collect::<Result<_, _>>()
                    })
                    .transpose()?
                    .unwrap_or_default(),
                non_fee_balance_changes: self
                    .non_fee_balance_changes
                    .get(&entity)
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
    _substate_changes: &BySubstate<ChangeAction>,
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
