use node_common::locks::StateLock;
use std::ops::Deref;
use std::sync::Arc;

use crate::scrypto_prelude::*;

use crate::protocol::*;

use crate::StateManagerDatabase;

pub trait ProtocolUpdater {
    /// Executes these state updates associated with the given protocol version
    /// that haven't yet been applied
    /// (hence "remaining", e.g. if node is restarted mid-protocol update).
    fn execute_remaining_state_updates(&self, store: Arc<StateLock<StateManagerDatabase>>);
}

pub struct NoOpProtocolUpdater;

impl ProtocolUpdater for NoOpProtocolUpdater {
    fn execute_remaining_state_updates(&self, _store: Arc<StateLock<StateManagerDatabase>>) {
        // no-op
    }
}

pub(crate) struct BatchedUpdater<R: UpdateBatchGenerator> {
    new_protocol_version: ProtocolVersionName,
    new_state_computer_config: ProtocolStateComputerConfig,
    resolver: R,
}

impl<G: UpdateBatchGenerator> BatchedUpdater<G> {
    pub fn new(
        new_protocol_version: ProtocolVersionName,
        new_state_computer_config: ProtocolStateComputerConfig,
        batch_generator: G,
    ) -> Self {
        Self {
            new_protocol_version,
            new_state_computer_config,
            resolver: batch_generator,
        }
    }
}

impl<R: UpdateBatchGenerator> ProtocolUpdater for BatchedUpdater<R> {
    fn execute_remaining_state_updates(&self, store: Arc<StateLock<StateManagerDatabase>>) {
        let mut txn_committer = ProtocolUpdateTransactionCommitter::new(
            self.new_protocol_version.clone(),
            store.clone(),
            self.new_state_computer_config.execution_configurator(true), /* No fees for protocol updates */
            self.new_state_computer_config
                .ledger_transaction_validator(),
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            let batch = {
                // Put it in a scope to ensure the read lock is dropped before we attempt to commit
                let read_store = store.read_current();
                self.resolver
                    .generate_batch(read_store.deref(), next_batch_idx)
            };
            match batch {
                Some(flash_txns) => {
                    txn_committer.commit_batch(flash_txns);
                }
                None => break,
            }
        }
    }
}

pub(crate) trait UpdateBatchGenerator {
    /// Generate a batch of transactions to be committed atomically with a proof.
    /// Return None if it's the last batch.
    fn generate_batch(
        &self,
        state_database: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>>;
}
