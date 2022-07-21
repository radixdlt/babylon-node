use crate::transaction_store::*;
use crate::types::*;

mod in_memory_proofdb;
mod in_memory_transactiondb;

use in_memory_proofdb::*;
use in_memory_transactiondb::*;

pub struct InMemoryTransactionStore {
    transaction_db: InMemoryTransactionDatabase,
    proof_db: InMemoryProofDatabase,
}

impl InMemoryTransactionStore {
    pub fn new(minimum_block_size: u64) -> InMemoryTransactionStore {
        InMemoryTransactionStore {
            transaction_db: InMemoryTransactionDatabase::new(),
            proof_db: InMemoryProofDatabase::new(minimum_block_size),
        }
    }
}

impl TransactionStore for InMemoryTransactionStore {
    fn store_begin(&mut self) {
        // Nop in memory store.
    }

    fn store_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<TransactionStateVersion, TransactionStoreStoreError> {
        self.transaction_db.store(transaction)
    }

    fn store_proof(&mut self, proof: LedgerProof) -> Result<(), StoreProofError> {
        let last_version: TransactionStateVersion = match self.transaction_db.last_version() {
            Some(version) => version,
            None => {
                return Err(StoreProofError::NoTransactionBeforeProof);
            }
        };

        let proof_version = proof.state_version();

        if last_version != proof_version {
            return Err(StoreProofError::ProofStateVersionMismatch(
                proof_version,
                last_version,
            ));
        }

        self.proof_db.store(proof);
        Ok(())
    }

    fn store_vertex_state(&mut self, vertex_state: Vec<u8>) {
        // TODO: Implement me. And decide on name.
    }

    fn store_commit(&mut self) {
        // Nop in memory store.
    }

    fn epoch_proof(&self, epoch: EpochId) -> Result<LedgerProof, EpochProofError> {
        self.proof_db
            .epoch_proof(epoch)
            .ok_or(EpochProofError::EpochProofNotFound(epoch))
    }

    fn last_proof(&self) -> Result<LedgerProof, LastProofError> {
        self.proof_db
            .last_proof()
            .ok_or(LastProofError::ProofNotFound)
    }

    fn next_proved_transactions(
        &self,
        state_version: TransactionStateVersion,
    ) -> Result<ProvedTransactions, NextProvedTransactionsError> {
        let next_proof = self.proof_db.next_proof(state_version)?;

        let mut transactions = Vec::new();
        for i in state_version + 1..next_proof.state_version {
            let t = self
                .transaction_db
                .get(i)
                .ok_or(NextProvedTransactionsError::TransactionNotFound(i))?;
            transactions.push(t);
        }

        Ok(ProvedTransactions::new(next_proof, transactions))
    }
}
