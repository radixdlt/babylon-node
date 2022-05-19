use std::collections::BTreeMap;

pub struct TransactionStore {
    in_memory_store: BTreeMap<u64, Vec<u8>>
}

impl TransactionStore {
    pub fn new() -> TransactionStore {
        TransactionStore { in_memory_store: BTreeMap::new() }
    }

    pub fn insert_transaction(&mut self, state_version: u64, transaction_data: Vec<u8>) {
        self.in_memory_store.insert(state_version, transaction_data);
    }

    pub fn get_transaction(&self, state_version: u64) -> &Vec<u8> {
        self.get_transactions_in_range(state_version, state_version + 1)[0]
    }

    pub fn get_transactions_in_range(&self, start_state_version: u64, end_state_version: u64) -> Vec<&Vec<u8>> {
        let mut txs: Vec<&Vec<u8>> = Vec::new();
        for state_version in start_state_version..end_state_version {
            let tx_data = self.in_memory_store.get(&state_version).unwrap();
            txs.push(tx_data);
        }
        txs
    }
}
