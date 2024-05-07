use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;

use crate::engine_prelude::*;

use crate::protocol::*;
use crate::transaction::{ExecutionConfigurator, LedgerTransactionValidator};

pub trait ProtocolUpdater {
    /// Executes these state updates associated with the given protocol version
    /// that haven't yet been applied
    /// (hence "remaining", e.g. if node is restarted mid-protocol update).
    fn execute_remaining_state_updates(
        &self,
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
    );
}

pub struct NoOpProtocolUpdater;

impl ProtocolUpdater for NoOpProtocolUpdater {
    fn execute_remaining_state_updates(
        &self,
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
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
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
    ) {
        let database = database.lock();
        let mut txn_committer = ProtocolUpdateTransactionCommitter::new(
            self.new_protocol_version.clone(),
            database.deref(),
            // The costing and logging parameters (of the Engine) are not really used for flash
            // transactions; let's still pass sane values.
            // TODO(when we need non-flash transactions): pass the actually configured flags here.
            ExecutionConfigurator::new(network, true, false),
            LedgerTransactionValidator::default_from_network(network),
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            let batch = self
                .batch_generator
                .generate_transactions(database.deref(), next_batch_idx);
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
    fn generate_transactions(
        &self,
        state_database: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>>;
}
