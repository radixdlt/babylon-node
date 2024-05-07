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
use crate::transaction::*;
use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;

/// A [`ProtocolUpdateActionProvider`] implementation for the actual Engine's protocol updates.
pub struct EngineProtocolUpdateActionProvider<G> {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    engine_batch_generator: G,
}

/// Creates an [`EngineProtocolUpdateActionProvider`] for the given [`UpdateSettings`], with all
/// the features that Engine wants enabled by default.
pub fn engine_default_for_network<U: UpdateSettings>(
    network: &NetworkDefinition,
    database: Arc<DbLock<ActualStateManagerDatabase>>,
) -> EngineProtocolUpdateActionProvider<U::BatchGenerator> {
    EngineProtocolUpdateActionProvider {
        database,
        engine_batch_generator: U::all_enabled_as_default_for_network(network)
            .create_batch_generator(),
    }
}

impl<G: ProtocolUpdateBatchGenerator> ProtocolUpdateActionProvider
    for EngineProtocolUpdateActionProvider<G>
{
    fn provide_action(&self, index: u32) -> Option<ProtocolUpdateAction> {
        self.engine_batch_generator
            .generate_batch(self.database.lock().deref(), index)
            .map(|batch| {
                ProtocolUpdateAction::FlashTransactions(
                    batch
                        .transactions
                        .into_iter()
                        .map(FlashTransactionV1::from)
                        .collect(),
                )
            })
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
