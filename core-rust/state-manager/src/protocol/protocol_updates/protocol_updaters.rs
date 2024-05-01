use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;

use crate::engine_prelude::*;

use crate::protocol::*;
use crate::transaction::{LedgerTransactionValidator, TransactionExecutorFactory};

pub trait ProtocolUpdater {
    /// Executes these state updates associated with the given protocol version that have not yet
    /// been applied (hence "remaining", e.g. if node is restarted mid-protocol update).
    fn execute_remaining_state_updates(
        &self,
        // TODO(resolve during review): should it rather be an already-locked `&S`?
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        transaction_executor_factory: &TransactionExecutorFactory,
        ledger_transaction_validator: &LedgerTransactionValidator,
    );
}

pub struct NoOpProtocolUpdater;

impl ProtocolUpdater for NoOpProtocolUpdater {
    fn execute_remaining_state_updates(
        &self,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        _transaction_executor_factory: &TransactionExecutorFactory,
        _ledger_transaction_validator: &LedgerTransactionValidator,
    ) {
        // no-op
    }
}

pub(crate) struct BatchedUpdater<G: UpdateBatchGenerator> {
    new_protocol_version: ProtocolVersionName,
    batch_generator: G,
}

impl<G: UpdateBatchGenerator> BatchedUpdater<G> {
    pub fn new(new_protocol_version: ProtocolVersionName, batch_generator: G) -> Self {
        Self {
            new_protocol_version,
            batch_generator,
        }
    }
}

impl<G: UpdateBatchGenerator> ProtocolUpdater for BatchedUpdater<G> {
    fn execute_remaining_state_updates(
        &self,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        transaction_executor_factory: &TransactionExecutorFactory,
        ledger_transaction_validator: &LedgerTransactionValidator,
    ) {
        let database = database.lock();
        let mut txn_committer = ProtocolUpdateTransactionCommitter::new(
            self.new_protocol_version.clone(),
            database.deref(),
            transaction_executor_factory,
            ledger_transaction_validator,
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            let batch = self
                .batch_generator
                .generate_batch(database.deref(), next_batch_idx);
            match batch {
                Some(flash_txns) => {
                    txn_committer.commit_batch(next_batch_idx, flash_txns);
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
