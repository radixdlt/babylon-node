use crate::store::traits::*;
use crate::CommittedTransactionIdentifiers;

pub trait TransactionIdentifierLoader {
    fn get_top_of_ledger_transaction_identifiers(&self) -> Option<CommittedTransactionIdentifiers>;
}

impl<T: QueryableProofStore + QueryableTransactionStore> TransactionIdentifierLoader for T {
    fn get_top_of_ledger_transaction_identifiers(&self) -> Option<CommittedTransactionIdentifiers> {
        let top_state_version = self.max_state_version();
        self.get_committed_transaction_identifiers(top_state_version)
    }
}
