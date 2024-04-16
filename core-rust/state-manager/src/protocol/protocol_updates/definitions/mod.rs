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

struct ScryptoEntriesBatchGenerator {
    named_entries: Vec<(String, ProtocolUpdateEntry)>,
}

impl ScryptoEntriesBatchGenerator {
    pub fn new(named_entries: &[(&str, ProtocolUpdateEntry)]) -> Self {
        Self {
            named_entries: named_entries
                .iter()
                .map(|(name, entry)| (name.to_string(), *entry))
                .collect(),
        }
    }
}

impl UpdateBatchGenerator for ScryptoEntriesBatchGenerator {
    fn generate_batch(
        &self,
        store: &impl SubstateDatabase,
        network: &NetworkDefinition,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>> {
        match batch_index {
            // Just a single batch for regular Scrypto updates:
            0 => Some(
                self.named_entries
                    .iter()
                    .map(|(name, entry)| {
                        FlashTransactionV1 {
                            name: name.clone(),
                            state_updates: entry.generate_state_updates(store, network),
                        }
                        .into()
                    })
                    .collect(),
            ),
            _ => None,
        }
    }
}
