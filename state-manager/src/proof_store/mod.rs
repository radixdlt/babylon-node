use crate::types::*;

#[derive(Debug)]
pub enum NextProofError {
    InvalidStateVersion(TransactionStateVersion),
    NotFound(TransactionStateVersion),
}

pub trait ProofStore {
    fn store_proof(&mut self, proof: LedgerProof);
    fn epoch_proof(&self, epoch: EpochId) -> Option<LedgerProof>;
    fn next_proof(
        &self,
        state_version: TransactionStateVersion,
    ) -> Result<LedgerProof, NextProofError>;
    fn last_proof(&self) -> Option<LedgerProof>;
}

mod in_memory;
