use enum_dispatch::enum_dispatch;

use crate::{AccumulatorState, CommittedTransactionIdentifiers};

#[enum_dispatch]
pub trait TransactionIdentifierLoader {
    fn get_top_transaction_identifiers(&self) -> Option<CommittedTransactionIdentifiers>;
    fn get_top_accumulator_state(&self) -> AccumulatorState {
        self.get_top_transaction_identifiers()
            .map(|ids| ids.resultant_accumulator_state)
            .unwrap_or_else(AccumulatorState::pre_genesis)
    }
}
