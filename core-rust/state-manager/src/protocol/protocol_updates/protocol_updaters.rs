use std::ops::Deref;
use std::sync::Arc;

use crate::protocol::*;
use crate::traits::*;
use crate::StateManagerDatabaseLock;

pub trait ProtocolUpdater {
    /// Executes these state updates associated with the given protocol version
    /// that haven't yet been applied
    /// (hence "remaining", e.g. if node is restarted mid-protocol update).
    fn execute_remaining_state_updates(&self, database: Arc<StateManagerDatabaseLock>);
}

pub struct NoOpProtocolUpdater;

impl ProtocolUpdater for NoOpProtocolUpdater {
    fn execute_remaining_state_updates(&self, _database: Arc<StateManagerDatabaseLock>) {
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
    fn execute_remaining_state_updates(&self, database: Arc<StateManagerDatabaseLock>) {
        let database = database.lock();
        let mut txn_committer = ProtocolUpdateTransactionCommitter::new(
            self.new_protocol_version.clone(),
            database.deref(),
            // The costing and logging parameters (of the Engine) are not really used for flash
            // transactions; let's still pass sane values.
            self.new_state_computer_config
                .execution_configurator(true, false),
            self.new_state_computer_config
                .ledger_transaction_validator(),
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            let batch = self
                .resolver
                .generate_batch(database.deref(), next_batch_idx);
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
