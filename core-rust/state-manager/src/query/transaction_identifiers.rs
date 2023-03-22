use crate::CommittedTransactionIdentifiers;

pub trait TransactionIdentifierLoader {
    fn get_top_transaction_identifiers(&self) -> CommittedTransactionIdentifiers;
}
