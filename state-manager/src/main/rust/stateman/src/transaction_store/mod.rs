mod in_memory_transaction_store;
mod transaction_store_jni;

pub use in_memory_transaction_store::TransactionAndProofStore;

#[derive(PartialEq, Debug)]
pub struct Transaction {
    state_version: u64, 
    transaction_data: Vec<u8>
}

impl Transaction {
    pub fn new(state_version: &u64, transaction_data: &Vec<u8>) -> Transaction {
        Transaction { state_version: state_version.clone(), transaction_data: transaction_data.clone() }
    }
}