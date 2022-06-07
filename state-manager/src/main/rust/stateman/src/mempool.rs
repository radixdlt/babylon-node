pub use crate::result::ToStateManagerError;

use crate::result::{StateManagerError, ERRCODE_MEMPOOL_DUPLICATE, ERRCODE_MEMPOOL_FULL};
use crate::types::Transaction;
use std::collections::HashSet;
use std::string::ToString;

#[derive(Debug, PartialEq)]
pub enum MempoolError {
    Full(usize, usize),
    Duplicate,
}

impl ToString for MempoolError {
    fn to_string(&self) -> String {
        match self {
            MempoolError::Full(a, b) => format!("Mempool Full [{} - {}]", a, b),
            MempoolError::Duplicate => "Duplicate Entry".to_string(),
        }
    }
}

impl ToStateManagerError for MempoolError {
    fn to_state_manager_error(&self) -> StateManagerError {
        let message = self.to_string();
        match self {
            MempoolError::Full(_, _) => StateManagerError::create(ERRCODE_MEMPOOL_FULL, message),
            MempoolError::Duplicate => {
                StateManagerError::create(ERRCODE_MEMPOOL_DUPLICATE, message)
            }
        }
    }
}

pub trait Mempool {
    fn add(&mut self, transaction: Transaction) -> Result<(), MempoolError>;
    fn committed(&mut self, txns: &HashSet<Transaction>);
    fn get_count(&self) -> usize;
    fn get_txns(&self, count: usize, seen: &HashSet<Transaction>) -> HashSet<Transaction>;
}

pub mod mock;
