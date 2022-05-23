use std::collections::BTreeMap;
use std::ops::Bound::Included;

use crate::transaction_store::Transaction; 

pub struct TransactionAndProofStore {
    in_memory_tx_store: BTreeMap<u64, Vec<u8>>,
    in_memory_proof_store: BTreeMap<u64, Vec<u8>>,
    in_memory_epoch_proof_store: BTreeMap<u64, Vec<u8>>
}

impl TransactionAndProofStore {
    pub fn new() -> TransactionAndProofStore {
        TransactionAndProofStore { 
            in_memory_tx_store: BTreeMap::new(),
            in_memory_proof_store: BTreeMap::new(),
            in_memory_epoch_proof_store: BTreeMap::new()
        }
    }

    pub fn insert_transaction(&mut self, state_version: u64, transaction_data: Vec<u8>) {
        self.in_memory_tx_store.insert(state_version, transaction_data);
    }

    pub fn get_transaction(&self, state_version: u64) -> &Vec<u8> {
        self.get_transactions_in_range(state_version, state_version + 1)[0]
    }

    pub fn get_transactions_in_range(&self, start_state_version: u64, end_state_version: u64) -> Vec<&Vec<u8>> {
        self.in_memory_tx_store.range((Included(&start_state_version), Included(&end_state_version)))
            .map(|range| range.1)
            .collect()
    }

    pub fn get_last_transaction_data(&self) -> Option<Transaction> {
        self.in_memory_tx_store.iter().next_back()
            .map(|e| Transaction::new(e.0, e.1) )
    }
}

#[cfg(test)]
mod tests {
    use crate::{TransactionAndProofStore, transaction_store::Transaction};

    #[test]
    fn it_works() {
        let mut tx_db = TransactionAndProofStore::new();
        let tx2_data = vec![1, 2, 3];
        tx_db.insert_transaction(2, tx2_data.clone());
        assert_eq!(tx_db.get_transaction(2).to_vec(), tx2_data.clone());

        let tx3_data = &vec![4, 5, 6];
        tx_db.insert_transaction(3, tx3_data.clone());
        assert_eq!(tx_db.get_transaction(3).to_vec(), tx3_data.clone());

        let txs = tx_db.get_transactions_in_range(2, 3);
        assert_eq!(txs.into_iter().cloned().collect::<Vec<Vec<u8>>>(), vec![tx2_data.clone(), tx3_data.clone()]);

        assert_eq!(tx_db.get_last_transaction_data(), Some(Transaction::new(&3, tx3_data)));

        tx_db.insert_transaction(1, vec![1]);

        assert_eq!(tx_db.in_memory_tx_store.range(..3).map(|r| r.1).next_back().unwrap().to_vec(), tx2_data.clone());
    }
}