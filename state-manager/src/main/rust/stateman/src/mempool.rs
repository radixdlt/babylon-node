use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum MempoolError {
    Full(usize, usize),
    Duplicate,
}

pub trait Mempool {
    fn add(&mut self, transaction: Vec<u8>) -> Result<Vec<u8>, MempoolError>;
    fn committed(&mut self, txns: &HashSet<Vec<u8>>);
    fn get_count(&self) -> usize;
    fn get_txns(&self, count: usize, seen: &HashSet<Vec<u8>>) -> HashSet<Vec<u8>>;
}

pub mod mock;
