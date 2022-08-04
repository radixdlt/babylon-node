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

use crate::jni::dtos::*;
use crate::types::{
    EpochId, LedgerProof, ProvedTransactions, Transaction, TransactionStateVersion,
    TransactionStateVersionTrait,
};

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum TransactionStoreStoreError {
    ExhaustedStateVersions,
}
impl JavaStructure for TransactionStoreStoreError {}

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum FirstProvedTransactionsError {
    FirstProofNotFound,
    TransactionNotFound(TransactionStateVersion),
}

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum NextProvedTransactionsError {
    InvalidStateVersion(TransactionStateVersion),
    NextProofNotFound(TransactionStateVersion),
    TransactionNotFound(TransactionStateVersion),
}
impl JavaStructure for NextProvedTransactionsError {}

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum EpochProofError {
    EpochProofNotFound(EpochId),
}
impl JavaStructure for EpochProofError {}

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum LastProofError {
    ProofNotFound,
}
impl JavaStructure for LastProofError {}

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum StoreProofError {
    NoTransactionBeforeProof,
    ProofStateVersionMismatch(TransactionStateVersion, TransactionStateVersion),
}
impl JavaStructure for StoreProofError {}

#[derive(Debug, PartialEq, Encode, Decode, TypeId)]
pub enum VertexStateError {
    VertexStateNotFound,
}

pub trait TransactionStore {
    /// Begin a transaction in the underlying TransactionStore database.
    fn store_begin(&mut self);

    /// Store Transaction passed as argument in the TransactionStore database.
    fn store_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<TransactionStateVersion, TransactionStoreStoreError>;

    /// Store Proof in the TransactionStore database.
    fn store_proof(&mut self, proof: LedgerProof) -> Result<(), StoreProofError>;

    /// Store the latest Vertex Store State in the TransactionStore database.
    fn store_vertex_state(&mut self, vertex_state: Vec<u8>);

    /// Commit all writes in the underlying TransactionStore database.
    fn store_commit(&mut self);

    /// Get proof for epoch 'epoch'
    fn epoch_proof(&self, epoch: EpochId) -> Result<LedgerProof, EpochProofError>;

    /// Get last proof stored.
    fn last_proof(&self) -> Result<LedgerProof, LastProofError>;

    /// Get the first proof and associated transactions.
    fn first_proved_transactions(&self)
        -> Result<ProvedTransactions, FirstProvedTransactionsError>;

    /// Get next proof and associated transactions after the transaction at state version 'state_version'.
    fn next_proved_transactions(
        &self,
        state_version: TransactionStateVersion,
    ) -> Result<ProvedTransactions, NextProvedTransactionsError>;

    /// Get Vertex State
    fn vertex_state(&self) -> Result<Vec<u8>, VertexStateError>;
}

#[derive(Debug, TypeId, Encode, Decode, Clone)]
pub struct TransactionStoreConfig {
    pub minimum_block_size: u32,
}

pub mod in_memory;

#[cfg(test)]
mod tests {
    use crate::transaction_store::in_memory::InMemoryTransactionStore;
    use crate::transaction_store::*;
    use crate::types::*;

    #[test]
    fn test_in_memory() {
        let payload1 = vec![1u8; 32];
        let payload2 = vec![2u8; 32];
        let payload3 = vec![3u8; 32];

        let t1 = Transaction {
            payload: payload1.clone(),
            id: TId {
                bytes: payload1.clone(),
            },
        };
        let t2 = Transaction {
            payload: payload2.clone(),
            id: TId { bytes: payload2 },
        };
        let t3 = Transaction {
            payload: payload3.clone(),
            id: TId { bytes: payload3 },
        };

        let config = TransactionStoreConfig {
            minimum_block_size: 5,
        };
        let mut store = InMemoryTransactionStore::new(config);

        // Store first transaction.
        let ret = store.store_transaction(t1.clone());
        assert_eq!(ret, Ok(0));

        // Attempt storing a proof with the wrong last transaction state version.
        let proof = LedgerProof {
            state_version: 1,
            new_epoch: None,
            serialized: payload1.clone(),
        };
        let ret = store.store_proof(proof);
        assert_eq!(ret, Err(StoreProofError::ProofStateVersionMismatch(1, 0)));

        // Store a proof for the first transaction.
        let proof1 = LedgerProof{
            state_version: 0,
            new_epoch: Some(0),
            serialized: payload1.clone(),
        };
        let ret = store.store_proof(proof1.clone());
        assert_eq!(ret, Ok(()));
        let ret = store.last_proof();
        assert_eq!(ret, Ok(proof1.clone()));

        // Store second transaction.
        let ret = store.store_transaction(t2.clone());
        assert_eq!(ret, Ok(1));

        // Store a proof for the second transaction. Start new Epoch 1.
        let proof2 = LedgerProof {
            state_version: 1,
            new_epoch: Some(1),
            serialized: payload1.clone(),
        };
        let ret = store.store_proof(proof2.clone());
        assert_eq!(ret, Ok(()));
        let ret = store.last_proof();
        assert_eq!(ret, Ok(proof2.clone()));

        // Store third transaction.
        let ret = store.store_transaction(t3.clone());
        assert_eq!(ret, Ok(2));

        // Store a proof for the third transaction.
        let proof3 = LedgerProof {
            state_version: 2,
            new_epoch: Some(2),
            serialized: payload1.clone(),
        };
        let ret = store.store_proof(proof3.clone());
        assert_eq!(ret, Ok(()));
        let ret = store.last_proof();
        assert_eq!(ret, Ok(proof3.clone()));

        // Get proof for epoch id 0.
        let ret = store.epoch_proof(0);
        assert_eq!(Ok(proof1.clone()), ret);

        // Get proof for epoch id 1.
        let ret = store.epoch_proof(1);
        assert_eq!(Ok(proof2.clone()), ret);

        // Get proof for epoch id 2.
        let ret = store.epoch_proof(2);
        assert_eq!(Ok(proof3.clone()), ret);

        // Get first proved transaction.
        let proved0 = ProvedTransactions::new(proof1, vec![t1.clone()]);
        let ret = store.first_proved_transactions();
        assert_eq!(ret, Ok(proved0.clone()));

        // Get second proved transaction.
        let proved1 = ProvedTransactions::new(proof2, vec![t2.clone()]);
        let ret = store.next_proved_transactions(proved0.clone().proof.state_version);
        assert_eq!(ret, Ok(proved1.clone()));

        // Get third proved transaction.
        let proved2 = ProvedTransactions::new(proof3, vec![t3.clone()]);
        let ret = store.next_proved_transactions(proved1.clone().proof.state_version);
        assert_eq!(ret, Ok(proved2));


        // Test Vertex State Store
        let ret = store.vertex_state();
        assert_eq!(ret, Err(VertexStateError::VertexStateNotFound));

        store.store_vertex_state(payload1.clone());
        let ret = store.vertex_state();
        assert_eq!(ret, Ok(payload1.clone()));
    }
}
