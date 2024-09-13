mod anemone_definition;
mod bottlenose_definition;
mod custom_definition;
mod default_definition;
mod test_definition;

pub use anemone_definition::*;
pub use bottlenose_definition::*;
pub use custom_definition::*;
pub use default_definition::*;
pub use test_definition::*;

use crate::engine_prelude::*;
use crate::protocol::*;
use crate::store::rocks_db::ActualStateManagerDatabase;
use crate::transaction::*;
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;

/// A [`ProtocolUpdateNodeBatchGenerator`] implementation for the actual Engine's protocol updates.
pub struct EngineBatchGenerator<G> {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    engine_batch_generator: G,
}

/// Creates an [`EngineBatchGenerator`] for the given [`UpdateSettings`], with all
/// the features that Engine wants enabled by default.
pub fn engine_default_for_network<U: UpdateSettings>(
    network: &NetworkDefinition,
    database: Arc<DbLock<ActualStateManagerDatabase>>,
) -> EngineBatchGenerator<U::BatchGenerator> {
    EngineBatchGenerator {
        database,
        engine_batch_generator: U::all_enabled_as_default_for_network(network)
            .create_batch_generator(),
    }
}

impl<G: ProtocolUpdateBatchGenerator> ProtocolUpdateNodeBatchGenerator for EngineBatchGenerator<G> {
    fn generate_batch(&self, batch_idx: u32) -> ProtocolUpdateNodeBatch {
        let ProtocolUpdateBatch { transactions } = self
            .engine_batch_generator
            .generate_batch(self.database.lock().deref(), batch_idx);
        ProtocolUpdateNodeBatch::FlashTransactions(
            transactions
                .into_iter()
                .map(FlashTransactionV1::from)
                .collect(),
        )
    }

    fn batch_count(&self) -> u32 {
        self.engine_batch_generator.batch_count()
    }
}

impl From<ProtocolUpdateTransactionDetails> for FlashTransactionV1 {
    fn from(value: ProtocolUpdateTransactionDetails) -> Self {
        let ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) = value;
        let FlashProtocolUpdateTransactionDetails {
            name,
            state_updates,
        } = flash;
        FlashTransactionV1 {
            name,
            state_updates,
        }
    }
}
