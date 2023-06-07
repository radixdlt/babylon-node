use enum_dispatch::enum_dispatch;

use crate::{CommittedTransactionIdentifiers, LedgerHashes};

#[enum_dispatch]
pub trait TransactionIdentifierLoader {
    // TODO(no-accu-hash): it's worth having `pub struct StateVersion(u64);` at this point
    fn get_top_transaction_identifiers(&self) -> Option<(u64, CommittedTransactionIdentifiers)>;

    fn get_top_ledger_hashes(&self) -> (u64, LedgerHashes) {
        self.get_top_transaction_identifiers()
            .map(|(state_version, ids)| (state_version, ids.resultant_ledger_hashes))
            .unwrap_or_else(|| (0, LedgerHashes::pre_genesis()))
    }
}
