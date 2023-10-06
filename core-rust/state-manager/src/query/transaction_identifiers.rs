use enum_dispatch::enum_dispatch;

use crate::{CommittedTransactionIdentifiers, LedgerHashes, StateVersion};

#[enum_dispatch]
pub trait TransactionIdentifierLoader {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)>;

    fn get_top_ledger_hashes(&self) -> (StateVersion, LedgerHashes) {
        self.get_top_transaction_identifiers()
            .map(|(state_version, ids)| (state_version, ids.resultant_ledger_hashes))
            .unwrap_or_else(|| (StateVersion::pre_genesis(), LedgerHashes::pre_genesis()))
    }

    fn get_state_computer_lite_latest_state_version(
        &self,
    ) -> Option<StateVersion>;
}
