use std::error::Error;
use std::rc::Rc;
use std::time::SystemTime;

use postgres::Transaction;

use crate::da::scan_result::Tx;
use crate::da::db::*;
use crate::da::processors::DbIncrease;

pub struct LedgerTransactionProcessor {
    ledger_transactions: Vec<Rc<DbLedgerTransaction>>,
}

impl LedgerTransactionProcessor {
    pub fn new() -> Self {
        Self {
            ledger_transactions: vec![],
        }
    }

    pub fn process_tx(
        &mut self,
        tx: &Tx,
        _: &DbSequences
    ) -> Result<(), Box<dyn Error>> {
        let new_ledger_transaction = DbLedgerTransaction {
            state_version: tx.state_version,
            created_at: SystemTime::now(),
        };

        self.ledger_transactions.push(Rc::new(new_ledger_transaction));

        Ok(())
    }
}

impl DbIncrease for LedgerTransactionProcessor {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>> {
        let mut cnt = 0;

        cnt += persist_ledger_transactions(client, &self.ledger_transactions)?;

        Ok(cnt)
    }
}
