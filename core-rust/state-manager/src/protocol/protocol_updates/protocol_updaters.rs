use crate::{ActualStateManagerDatabase, SystemExecutor};
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;

use crate::engine_prelude::*;

use crate::protocol::*;

use crate::transaction::FlashTransactionV1;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ProtocolUpdateTransactionBatch {
    FlashTransactions(Vec<FlashTransactionV1>),
    Scenario(String),
}

pub trait ProtocolUpdater {
    /// Executes these state updates associated with the given protocol version that have not yet
    /// been applied (hence "remaining", e.g. if node is restarted mid-protocol update).
    fn execute_remaining_batches(
        &self,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        system_executor: &SystemExecutor,
    );
}

pub struct NoOpProtocolUpdater;

impl ProtocolUpdater for NoOpProtocolUpdater {
    fn execute_remaining_batches(
        &self,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        _system_executor: &SystemExecutor,
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

    fn resolve_next_batch_to_execute(
        &self,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
    ) -> Option<(u32, ProtocolUpdateTransactionBatch)> {
        let database = database.lock();
        let next_batch_idx = ProtocolUpdateProgress::resolve(database.deref())
            .scoped_on(&self.new_protocol_version)
            .next_batch_idx()?;
        let next_batch = self
            .batch_generator
            .generate_batch(database.deref(), next_batch_idx)?;
        Some((next_batch_idx, next_batch))
    }
}

impl<G: UpdateBatchGenerator> ProtocolUpdater for BatchedUpdater<G> {
    fn execute_remaining_batches(
        &self,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        system_executor: &SystemExecutor,
    ) {
        while let Some((idx, batch)) = self.resolve_next_batch_to_execute(database.clone()) {
            system_executor.execute_protocol_update_batch(&self.new_protocol_version, idx, batch);
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
    ) -> Option<ProtocolUpdateTransactionBatch>;
}
