use crate::proof_store::*;
use crate::types::{EpochId, TransactionStateVersion};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct InMemoryProofStore {
    minimum_block_size: u64,
    last_block_state_version: Option<TransactionStateVersion>,
    proof_map: BTreeMap<TransactionStateVersion, LedgerProof>,
    epoch_proof_map: BTreeMap<EpochId, LedgerProof>,
}

impl InMemoryProofStore {
    pub fn new(minimum_block_size: u64) -> InMemoryProofStore {
        InMemoryProofStore {
            minimum_block_size,
            last_block_state_version: None,
            proof_map: BTreeMap::new(),
            epoch_proof_map: BTreeMap::new(),
        }
    }

    fn is_new_block_needed(&self, proof: &LedgerProof) -> bool {
        if let Some(last_block_version) = self.last_block_state_version {
            let proof_version = proof.state_version();
            println!(
                "Proof state version: {:?} > {:?} (last_block)",
                proof_version, last_block_version
            );
            // Assert might seem harsh. Could use checked_sub and
            // unwrap but asserts makes it more explicit.
            assert!(proof_version > last_block_version);
            let version_diff = proof_version - last_block_version;
            println!(
                "version_diff: {} block_size: {}, cmp: {}",
                version_diff,
                self.minimum_block_size,
                version_diff > self.minimum_block_size
            );
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
}

impl ProofStore for InMemoryProofStore {
    fn store_proof(&mut self, proof: LedgerProof) {
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

    fn epoch_proof(&self, epoch: EpochId) -> Option<LedgerProof> {
        self.epoch_proof_map.get(&epoch).cloned()
    }

    fn next_proof(
        &self,
        previous_state_version: TransactionStateVersion,
    ) -> Result<LedgerProof, NextProofError> {
        let minimum_state_version = previous_state_version
            .next()
            .ok_or(NextProofError::InvalidStateVersion(previous_state_version))?;
        let (_, next_proof) = self
            .proof_map
            .range(minimum_state_version..)
            .next()
            .ok_or(NextProofError::NotFound(minimum_state_version))?;
        Ok(next_proof.clone())
    }

    fn last_proof(&self) -> Option<LedgerProof> {
        if let Some((_, proof)) = self.proof_map.iter().next_back() {
            Some(proof.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proof_store::in_memory::*;

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
    fn store_test() {
        let mut pc = TestProofCreator::new();

        let proof0 = pc.new_proof(false);
        let proof1 = pc.new_proof(false);
        let proof2 = pc.new_proof(false);
        let proof3 = pc.new_proof(false);
        let proof4 = pc.new_proof(false);

        let mut proofstore = InMemoryProofStore::new(3);
        assert_eq!(true, proofstore.is_new_block_needed(&proof0));
        println!("{:?}", proofstore);
        proofstore.store_proof(proof0.clone());
        //	assert_eq!(proofstore.last_block_state_version, None);
        println!("{:?}", proofstore);
        proofstore.store_proof(proof1.clone());
        //	assert_eq!(proofstore.last_block_state_version, Some(0));
        println!("{:?}", proofstore);
        proofstore.store_proof(proof2.clone());
        //	assert_eq!(proofstore.last_block_state_version, Some(1));
        println!("{:?}", proofstore);
        proofstore.store_proof(proof3.clone());
        //	assert_eq!(proofstore.last_block_state_version, Some(2));
        println!("{:?}", proofstore);
        proofstore.store_proof(proof4.clone());
        //	assert_eq!(proofstore.last_block_state_version, Some(2));
        println!("{:?}", proofstore);

        println!("{:?}", proofstore.epoch_proof(0));
        println!("{:?}", proofstore.epoch_proof(1));
        println!("{:?}", proofstore.epoch_proof(2));

        println!("Next Proof");
        println!("{:?}", proofstore.next_proof(0));
        println!("{:?}", proofstore.next_proof(1));
        println!("{:?}", proofstore.next_proof(2));
        println!("{:?}", proofstore.next_proof(3));
    }
}
