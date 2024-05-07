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

impl<G: ProtocolUpdateBatchGenerator> UpdateBatchGenerator for G {
    fn generate_transactions(
        &self,
        store: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<ProtocolUpdateTransactionBatch> {
        self.generate_batch(store, batch_index).map(|batch| {
            ProtocolUpdateTransactionBatch::FlashTransactions(
                batch
                    .transactions
                    .into_iter()
                    .map(|details| {
                        let ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) = details;
                        let FlashProtocolUpdateTransactionDetails {
                            name,
                            state_updates,
                        } = flash;
                        FlashTransactionV1 {
                            name,
                            state_updates,
                        }
                    })
                    .collect(),
            )
        })
    }
}
