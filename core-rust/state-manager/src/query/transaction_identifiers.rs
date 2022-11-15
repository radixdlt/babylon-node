use crate::store::traits::*;
use crate::CommittedTransactionIdentifiers;

pub trait TransactionIdentifierLoader {
    fn get_top_of_ledger_transaction_identifiers_unwrap(&self) -> CommittedTransactionIdentifiers;
    fn get_top_of_ledger_transaction_identifiers(&self) -> Option<CommittedTransactionIdentifiers>;
    fn get_transaction_identifiers(
        &self,
        state_version: u64,
    ) -> Option<CommittedTransactionIdentifiers>;
}

impl<T: QueryableProofStore + TransactionIndex<u64>> TransactionIdentifierLoader for T {
    fn get_top_of_ledger_transaction_identifiers_unwrap(&self) -> CommittedTransactionIdentifiers {
        let top_state_version = self.max_state_version();
        self.get_transaction_identifiers(top_state_version)
            .unwrap_or_else(|| {
                panic!(
                    "No transactions found in database at top-of-ledger state version: {}",
                    top_state_version
                )
            })
    }

    fn get_top_of_ledger_transaction_identifiers(&self) -> Option<CommittedTransactionIdentifiers> {
        let top_state_version = self.max_state_version();
        self.get_transaction_identifiers(top_state_version)
    }

    fn get_transaction_identifiers(
        &self,
        state_version: u64,
    ) -> Option<CommittedTransactionIdentifiers> {
        let payload_hash = self.get_payload_hash(state_version)?;

        // TODO: This is rather wasteful, particularly with a big genesis transaction(!)
        //       when we refactor the DB, we should be able to get this information much more cheaply
        let (_, _, identifiers) = self
            .get_committed_transaction(&payload_hash)
            .unwrap_or_else(|| {
                panic!(
                    "A transaction is missing at state version {}",
                    state_version
                )
            });
        Some(identifiers)
    }
}
