use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;

use crate::engine_prelude::*;

use crate::protocol::*;

pub trait ProtocolUpdater {
    /// Executes these state updates associated with the given protocol version
    /// that haven't yet been applied
    /// (hence "remaining", e.g. if node is restarted mid-protocol update).
    fn execute_remaining_state_updates(&self, database: Arc<DbLock<ActualStateManagerDatabase>>);
}

pub struct NoOpProtocolUpdater;

impl ProtocolUpdater for NoOpProtocolUpdater {
    fn execute_remaining_state_updates(&self, _database: Arc<DbLock<ActualStateManagerDatabase>>) {
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
    fn execute_remaining_state_updates(&self, database: Arc<DbLock<ActualStateManagerDatabase>>) {
        let database = database.lock();
        let mut txn_committer = ProtocolUpdateTransactionCommitter::new(
            self.new_protocol_version.clone(),
            database.deref(),
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            let batch = self
                .batch_generator
                .generate_batch(database.deref(), next_batch_idx);
            match batch {
                Some(flash_txns) => {
                    txn_committer.commit_batch(flash_txns);
                }
                None => break,
            }
        }

        // if let Some(next_protocol_version) = series_executor.next_protocol_version() {
        //     self.
        //     let mut system_commit_request_factory = SystemCommitRequestFactory {
        //         epoch: series_executor.epoch(),
        //         timestamp: proposer_timestamp_ms,
        //         state_version: series_executor.latest_state_version(),
        //         proof_origin: LedgerProofOrigin::ProtocolUpdate { genesis_opaque_hash },
        //     };
        //     let protocol_update_scenarios = self.scenarios_execution_config
        //         .to_run_after_protocol_update(&next_protocol_version);
        //     self.execute_scenarios(&mut system_commit_request_factory, protocol_update_scenarios);
        // }
        // TODO(wip): scenarios
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
