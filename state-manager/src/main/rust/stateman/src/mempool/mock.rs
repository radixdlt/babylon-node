use crate::mempool::*;
use std::collections::HashSet;

pub struct MockMempool {
    max_size: usize,
    data: HashSet<Transaction>,
}

impl MockMempool {
    pub fn new(max_size: usize) -> MockMempool {
        MockMempool {
            max_size,
            data: HashSet::new(),
        }
    }
}

impl Mempool for MockMempool {
    fn add(&mut self, transaction: Transaction) -> Result<(), MempoolError> {
        let len = self.data.len();

        if len >= self.max_size {
            return Err(MempoolError::Full(len, self.max_size));
        }

        if !self.data.insert(transaction) {
            return Err(MempoolError::Duplicate);
        }

        Ok(())
    }

    fn committed(&mut self, txns: &HashSet<Transaction>) {
        for t in txns {
            self.data.remove(t);
        }
    }

    fn get_count(&self) -> usize {
        self.data.len()
    }

    fn get_txns(&self, count: usize, seen: &HashSet<Transaction>) -> HashSet<Transaction> {
        self.data.difference(seen).take(count).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::mempool::mock::*;
    use crate::types::*;

    #[test]
    fn mock_test() {
        let pl1 = vec![1u8; 32];
        let pl2 = vec![2u8; 32];
        let pl3 = vec![3u8; 32];

        let tv1 = Transaction {
            payload: pl1.clone(),
            id: Aid { bytes: pl1 },
        };
        let tv2 = Transaction {
            payload: pl2.clone(),
            id: Aid { bytes: pl2 },
        };
        let tv3 = Transaction {
            payload: pl3.clone(),
            id: Aid { bytes: pl3 },
        };

        let mut mp = MockMempool::new(2);
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 0);
        let get = mp.get_txns(3, &HashSet::new());
        assert!(get.is_empty());

        let rc = mp.add(tv1.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains(&tv1));
        let get = mp.get_txns(3, &HashSet::new());
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let get = mp.get_txns(3, &HashSet::from([tv1.clone(), tv2.clone(), tv3.clone()]));
        assert!(get.is_empty());

        let get = mp.get_txns(3, &HashSet::from([tv2.clone(), tv3.clone()]));
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.add(tv1.clone());
        assert!(rc.is_err());
        assert_eq!(rc, Err(MempoolError::Duplicate));

        let rc = mp.add(tv2.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 2);
        assert!(mp.data.contains(&tv1));
        assert!(mp.data.contains(&tv2));

        let get = mp.get_txns(3, &HashSet::new());
        assert_eq!(get.len(), 2);
        assert!(get.contains(&tv1));
        assert!(get.contains(&tv2));

        let get = mp.get_txns(3, &HashSet::from([tv1.clone(), tv2.clone(), tv3.clone()]));
        assert!(get.is_empty());

        let get = mp.get_txns(3, &HashSet::from([tv2.clone(), tv3.clone()]));
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let get = mp.get_txns(3, &HashSet::from([tv1.clone(), tv3.clone()]));
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv2));

        mp.committed(&HashSet::from([tv1.clone()]));
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains(&tv2));
        assert!(!mp.data.contains(&tv1));

        mp.committed(&HashSet::from([tv2.clone()]));
        assert_eq!(mp.get_count(), 0);
        assert!(!mp.data.contains(&tv2));
        assert!(!mp.data.contains(&tv1));
    }
}
