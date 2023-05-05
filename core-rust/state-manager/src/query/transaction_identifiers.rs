use enum_dispatch::enum_dispatch;

use crate::CommittedTransactionIdentifiers;

#[enum_dispatch]
pub trait TransactionIdentifierLoader {
    fn get_top_transaction_identifiers(&self) -> CommittedTransactionIdentifiers;
}
