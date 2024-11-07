use crate::prelude::*;

pub fn to_api_transaction_tree_hash(transaction_tree_hash: &TransactionTreeHash) -> String {
    to_hex(transaction_tree_hash)
}
