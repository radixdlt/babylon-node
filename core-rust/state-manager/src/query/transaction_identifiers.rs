use crate::prelude::*;

pub trait TransactionIdentifierLoader {
    fn get_top_transaction_identifiers(
        &self,
    ) -> Option<(StateVersion, CommittedTransactionIdentifiers)>;

    fn get_top_ledger_hashes(&self) -> (StateVersion, LedgerHashes) {
        self.get_top_transaction_identifiers()
            .map(|(state_version, ids)| (state_version, ids.resultant_ledger_hashes))
            .unwrap_or_else(|| (StateVersion::pre_genesis(), LedgerHashes::pre_genesis()))
    }
}
