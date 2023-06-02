use enum_dispatch::enum_dispatch;

use crate::{CommitBasedIdentifiers, CommittedTransactionIdentifiers};

#[enum_dispatch]
pub trait TransactionIdentifierLoader {
    fn get_top_transaction_identifiers(&self) -> Option<CommittedTransactionIdentifiers>;
    fn get_top_commit_identifiers(&self) -> CommitBasedIdentifiers {
        self.get_top_transaction_identifiers()
            .map(|ids| ids.at_commit)
            .unwrap_or_else(CommitBasedIdentifiers::pre_genesis)
    }
}
