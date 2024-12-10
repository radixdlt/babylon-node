use crate::prelude::*;
pub use radix_engine::system::system_modules::costing::*;
pub use radix_engine::transaction::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FeePaymentBalanceChangeType {
    FeePayment,
    FeeDistributed,
    TipDistributed,
    RoyaltyDistributed,
}

pub struct FeePaymentComputer<'a> {
    inputs: FeePaymentComputationInputs<'a>,
    computation: FeePaymentComputation,
}

pub struct FeePaymentComputationInputs<'a> {
    /// The balance changes caused by [`FeeSource#paying_vaults`] (resolved to global ancestors).
    /// Note: this information is logically of the same type as [`balance_changes`], but the actual
    /// signature is simpler, since all fees are necessarily XRD and thus have fungible balances.
    pub fee_balance_changes: IndexMap<GlobalAddress, Decimal>,
    pub fee_summary: &'a TransactionFeeSummary,
    pub fee_destination: &'a FeeDestination,
    /// The total balance changes (i.e. including the [`fee_balance_changes`]).
    pub balance_changes: &'a IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
}

pub struct FeePaymentComputation {
    pub relevant_entities: IndexSet<GlobalAddress>,
    pub fee_balance_changes: NonIterMap<GlobalAddress, Vec<(FeePaymentBalanceChangeType, Decimal)>>,
    pub non_fee_balance_changes: NonIterMap<GlobalAddress, IndexMap<ResourceAddress, Decimal>>,
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
                FeePaymentBalanceChangeType::FeePayment,
            );
        }
    }

    fn track_fee_distributions(&mut self) {
        self.record_fee_balance_change_if_non_zero(
            CONSENSUS_MANAGER.into(),
            self.inputs
                .fee_summary
                .network_fees()
                .sub_or_panic(self.inputs.fee_destination.to_burn),
            FeePaymentBalanceChangeType::FeeDistributed,
        );
        self.record_fee_balance_change_if_non_zero(
            CONSENSUS_MANAGER.into(),
            self.inputs.fee_summary.total_tipping_cost_in_xrd,
            FeePaymentBalanceChangeType::TipDistributed,
        );
    }

    fn track_royalty_distributions(&mut self) {
        for (recipient, amount) in &self.inputs.fee_destination.to_royalty_recipients {
            let recipient: GlobalAddress = match recipient {
                RoyaltyRecipient::Package(address, _) => (*address).into(),
                RoyaltyRecipient::Component(address, _) => (*address).into(),
            };
            self.record_fee_balance_change_if_non_zero(
                recipient,
                *amount,
                FeePaymentBalanceChangeType::RoyaltyDistributed,
            );
        }
    }

    fn record_fee_balance_change_if_non_zero(
        &mut self,
        address: GlobalAddress,
        balance_change: Decimal,
        fee_type: FeePaymentBalanceChangeType,
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
            .or_default()
            .push((fee_type, balance_change));
    }

    fn finalize_non_fee_balance_changes(&mut self) {
        for (entity, changes) in self.inputs.balance_changes {
            let total_fee_balance_changes: Decimal = self
                .computation
                .fee_balance_changes
                .get(entity)
                .map(|fee_payments| fee_payments.iter().map(|p| p.1).sum_or_panic())
                .unwrap_or_default();
            let mut non_fee_balance_changes: IndexMap<ResourceAddress, Decimal> = changes
                .iter()
                .filter_map(|(resource, balance_change)| {
                    if resource == &XRD {
                        let total_balance_change = get_fungible_balance(balance_change)
                            .expect("Expected XRD to be fungible");
                        let total_non_fee_balance_change =
                            total_balance_change.sub_or_panic(total_fee_balance_changes);
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
                non_fee_balance_changes.insert(XRD, total_fee_balance_changes.neg_or_panic());
            }
            self.computation
                .non_fee_balance_changes
                .insert(*entity, non_fee_balance_changes);
        }
    }
}

pub fn get_fungible_balance(balance_change: &BalanceChange) -> Option<Decimal> {
    match balance_change {
        BalanceChange::Fungible(balance_change) => Some(*balance_change),
        BalanceChange::NonFungible { .. } => None,
    }
}
