mod entity_definitions;
mod ledger_transactions;
mod metadata;
mod role_assignment;

use std::error::Error;
use postgres::Transaction;
pub use entity_definitions::*;
pub use ledger_transactions::*;
pub use metadata::*;
pub use role_assignment::*;

pub trait DbIncrease {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>>;
}