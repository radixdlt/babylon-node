use crate::transaction_store::*;
use crate::types::{EpochId, TransactionStateVersion};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct InMemoryProofDatabase {
    minimum_block_size: u64,
    last_block_state_version: Option<TransactionStateVersion>,
    proof_map: BTreeMap<TransactionStateVersion, LedgerProof>,
    epoch_proof_map: BTreeMap<EpochId, LedgerProof>,
}

impl InMemoryProofDatabase {
    pub fn new(minimum_block_size: u64) -> InMemoryProofDatabase {
        InMemoryProofDatabase {
            minimum_block_size,
            last_block_state_version: None,
            proof_map: BTreeMap::new(),
            epoch_proof_map: BTreeMap::new(),
        }
    }

    fn is_new_block_needed(&self, proof: &LedgerProof) -> bool {
        if let Some(last_block_version) = self.last_block_state_version {
            let proof_version = proof.state_version();
            // Assert might seem harsh. Could use checked_sub and
            // unwrap but asserts makes it more explicit.
            assert!(proof_version > last_block_version);
            let version_diff = proof_version - last_block_version;
            version_diff > self.minimum_block_size
        } else {
            // First block.
            true
        }
    }

    fn last_state_version(&self) -> Option<TransactionStateVersion> {
        if let Some((state_version, _)) = self.proof_map.iter().next_back() {
            Some(*state_version)
        } else {
            None
        }
    }

    pub fn store(&mut self, proof: LedgerProof) {
        if let Some(epoch_id) = proof.new_epoch() {
            self.epoch_proof_map.insert(epoch_id, proof.clone());
        }

        if proof.new_epoch().is_some() || self.is_new_block_needed(&proof) {
            // Create new block.
            self.last_block_state_version = self.last_state_version();
        } else {
            // Delete last entry
            let last_entry = self.last_state_version();

            if let Some(state_version) = last_entry {
                self.proof_map.remove(&state_version);
            }
        }
        self.proof_map.insert(proof.state_version(), proof);
    }

    pub fn epoch_proof(&self, epoch: EpochId) -> Option<LedgerProof> {
        self.epoch_proof_map.get(&epoch).cloned()
    }

    pub fn next_proof(
        &self,
        previous_state_version: TransactionStateVersion,
    ) -> Result<LedgerProof, NextProvedTransactionsError> {
        let minimum_state_version = previous_state_version.next().ok_or(
            NextProvedTransactionsError::InvalidStateVersion(previous_state_version),
        )?;
        let (_, next_proof) = self.proof_map.range(minimum_state_version..).next().ok_or(
            NextProvedTransactionsError::NextProofNotFound(minimum_state_version),
        )?;
        Ok(next_proof.clone())
    }

    pub fn last_proof(&self) -> Option<LedgerProof> {
        if let Some((_, proof)) = self.proof_map.iter().next_back() {
            Some(proof.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_store::in_memory::*;

    struct TestProofCreator {
        state_version: TransactionStateVersion,
        epoch_id: EpochId,
    }

    impl TestProofCreator {
        fn new() -> TestProofCreator {
            TestProofCreator {
                state_version: 0,
                epoch_id: 0,
            }
        }

        fn new_proof(&mut self, new_epoch: bool) -> LedgerProof {
            let payload = vec![1u8; 1];
            let state_version = self.state_version;
            self.state_version = state_version.next().unwrap();

            let epoch = if new_epoch {
                let e = self.epoch_id;
                self.epoch_id = e + 1;
                Some(e)
            } else {
                None
            };
            LedgerProof::new(state_version, epoch, payload)
        }
    }

    #[test]
    fn store_block_test() {
        // Test with 1 transaction per block.
        let mut proofstore = InMemoryProofDatabase::new(1);

        let mut pc = TestProofCreator::new();
        let proof0 = pc.new_proof(false);
        let proof1 = pc.new_proof(false);
        let proof2 = pc.new_proof(false);
        let proof3 = pc.new_proof(false);
        let proof4 = pc.new_proof(false);

        // Store 4 transactions.
        assert_eq!(true, proofstore.is_new_block_needed(&proof0));
        proofstore.store(proof0.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof0.state_version())
        );
        proofstore.store(proof1.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof1.state_version())
        );
        proofstore.store(proof2.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof2.state_version())
        );
        proofstore.store(proof3.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof3.state_version())
        );
        proofstore.store(proof4.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof4.state_version())
        );

        // Check blocks
        assert_eq!(proofstore.next_proof(0), Ok(proof1.clone()));
        assert_eq!(proofstore.next_proof(1), Ok(proof2.clone()));
        assert_eq!(proofstore.next_proof(2), Ok(proof3.clone()));
        assert_eq!(proofstore.next_proof(3), Ok(proof4.clone()));
        // Check final store state
        assert_eq!(proofstore.last_proof(), Some(proof4.clone()));

        // Test with 3 transactions per block.

        let mut proofstore = InMemoryProofDatabase::new(3);

        let mut pc = TestProofCreator::new();
        let proof0 = pc.new_proof(false);
        let proof1 = pc.new_proof(false);
        let proof2 = pc.new_proof(false);
        let proof3 = pc.new_proof(false);
        let proof4 = pc.new_proof(false);

        // Store 4 transactions.
        assert_eq!(true, proofstore.is_new_block_needed(&proof0));
        proofstore.store(proof0.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof0.state_version())
        );
        proofstore.store(proof1.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof1.state_version())
        );
        proofstore.store(proof2.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof2.state_version())
        );
        proofstore.store(proof3.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof3.state_version())
        );
        proofstore.store(proof4.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof4.state_version())
        );

        // Check blocks
        assert_eq!(proofstore.next_proof(0), Ok(proof3.clone()));
        assert_eq!(proofstore.next_proof(1), Ok(proof3.clone()));
        assert_eq!(proofstore.next_proof(2), Ok(proof3.clone()));
        assert_eq!(proofstore.next_proof(3), Ok(proof4.clone()));
        // Check final store state
        assert_eq!(proofstore.last_proof(), Some(proof4.clone()));
    }

    #[test]
    fn store_epoch_test() {
        // Test epochs.

        // 5 transactions per block, but the epoch change comes before.
        let mut proofstore = InMemoryProofDatabase::new(5);

        let mut pc = TestProofCreator::new();
        let proof0 = pc.new_proof(false);
        let proof1 = pc.new_proof(false);
        let proof2 = pc.new_proof(false); // New epoch
        let proof3 = pc.new_proof(true);
        let proof4 = pc.new_proof(false);

        // Store 4 transactions.
        assert_eq!(true, proofstore.is_new_block_needed(&proof0));
        proofstore.store(proof0.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof0.state_version())
        );
        proofstore.store(proof1.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof1.state_version())
        );
        proofstore.store(proof2.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof2.state_version())
        );
        proofstore.store(proof3.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof3.state_version())
        );
        proofstore.store(proof4.clone());
        assert_eq!(
            proofstore.last_state_version(),
            Some(proof4.state_version())
        );

        // Check blocks
        // Epoch starts at third proof. Blocks should be:
        //
        //     [proof0], [proof2], [proof4]
        //
        assert_eq!(proofstore.next_proof(0), Ok(proof2.clone()));
        assert_eq!(proofstore.next_proof(1), Ok(proof2.clone()));
        assert_eq!(proofstore.next_proof(2), Ok(proof4.clone()));
        assert_eq!(proofstore.next_proof(3), Ok(proof4.clone()));
        // Check final store state
        assert_eq!(proofstore.last_proof(), Some(proof4.clone()));
    }
}
