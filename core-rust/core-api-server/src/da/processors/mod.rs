mod entity_definitions;
mod ledger_transactions;
mod metadata;

use std::error::Error;
use postgres::Transaction;
pub use entity_definitions::*;
pub use ledger_transactions::*;
pub use metadata::*;

pub trait DbIncrease {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>>;
}