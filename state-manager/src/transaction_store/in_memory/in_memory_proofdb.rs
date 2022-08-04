/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

use crate::transaction_store::*;
use crate::types::{EpochId, TransactionStateVersion};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct InMemoryProofDatabase {
    minimum_block_size: u32,
    last_block_state_version: Option<TransactionStateVersion>,
    proof_map: BTreeMap<TransactionStateVersion, LedgerProof>,
    epoch_proof_map: BTreeMap<EpochId, LedgerProof>,
}

impl InMemoryProofDatabase {
    pub fn new(minimum_block_size: u32) -> InMemoryProofDatabase {
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
            version_diff > (self.minimum_block_size as u64)
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

    pub fn first_proof(&self) -> Option<LedgerProof> {
        if let Some((_, proof)) = self.proof_map.iter().next() {
            Some(proof.clone())
        } else {
            None
        }
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

        fn new_proof(&mut self, start_epoch: bool) -> LedgerProof {
            let serialized = vec![1u8; 1];
            let state_version = self.state_version;
            self.state_version = state_version.next().unwrap();

            let new_epoch = if start_epoch {
                let e = self.epoch_id;
                self.epoch_id = e + 1;
                Some(e)
            } else {
                None
            };
            LedgerProof {
                state_version,
                new_epoch,
                serialized,
            }
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
        assert!(proofstore.is_new_block_needed(&proof0));
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
        assert_eq!(proofstore.next_proof(0), Ok(proof1));
        assert_eq!(proofstore.next_proof(1), Ok(proof2));
        assert_eq!(proofstore.next_proof(2), Ok(proof3));
        assert_eq!(proofstore.next_proof(3), Ok(proof4.clone()));
        // Check final store state
        assert_eq!(proofstore.last_proof(), Some(proof4));

        // Test with 3 transactions per block.

        let mut proofstore = InMemoryProofDatabase::new(3);

        let mut pc = TestProofCreator::new();
        let proof0 = pc.new_proof(false);
        let proof1 = pc.new_proof(false);
        let proof2 = pc.new_proof(false);
        let proof3 = pc.new_proof(false);
        let proof4 = pc.new_proof(false);

        // Store 4 transactions.
        assert!(proofstore.is_new_block_needed(&proof0));
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
        assert_eq!(proofstore.next_proof(2), Ok(proof3));
        assert_eq!(proofstore.next_proof(3), Ok(proof4.clone()));
        // Check final store state
        assert_eq!(proofstore.last_proof(), Some(proof4));
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
        assert!(proofstore.is_new_block_needed(&proof0));
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
        assert_eq!(proofstore.next_proof(1), Ok(proof2));
        assert_eq!(proofstore.next_proof(2), Ok(proof4.clone()));
        assert_eq!(proofstore.next_proof(3), Ok(proof4.clone()));
        // Check final store state
        assert_eq!(proofstore.last_proof(), Some(proof4));
    }
}
