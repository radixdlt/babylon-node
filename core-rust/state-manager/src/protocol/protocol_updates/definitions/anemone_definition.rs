use crate::engine_prelude::*;
use crate::protocol::*;
use crate::transaction::FlashTransactionV1;

pub const DEFAULT_MAX_VERTEX_TRANSACTION_COUNT: u32 = 100;

const ANEMONE_ENTRIES: [(&str, ProtocolUpdateEntry); 4] = [
    (
        "anemone-validator-fee-fix",
        ProtocolUpdateEntry::ValidatorCreationFeeFix,
    ),
    (
        "anemone-seconds-precision",
        ProtocolUpdateEntry::SecondPrecisionTimestamp,
    ),
    ("anemone-vm-boot", ProtocolUpdateEntry::Bls12381AndKeccak256),
    ("anemone-pools", ProtocolUpdateEntry::PoolMathPrecisionFix),
];

pub struct AnemoneProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for AnemoneProtocolUpdateDefinition {
    type Overrides = ();

    fn state_computer_config(
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        ProtocolStateComputerConfig::default(network_definition.clone())
    }

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        _config: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            Self::state_computer_config(network_definition),
            AnemoneBatchGenerator,
        ))
    }
}

struct AnemoneBatchGenerator;

impl UpdateBatchGenerator for AnemoneBatchGenerator {
    fn generate_batch(
        &self,
        store: &impl SubstateDatabase,
        network: &NetworkDefinition,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>> {
        match batch_index {
            0 => {
                // Just a single batch for Anemone, which includes the `ANEMONE_ENTRIES`:
                Some(
                    ANEMONE_ENTRIES
                        .iter()
                        .map(|(name, entry)| {
                            FlashTransactionV1 {
                                name: name.to_string(),
                                state_updates: entry.generate_state_updates(store, network),
                            }
                            .into()
                        })
                        .collect(),
                )
            }
            _ => None,
        }
    }
}
