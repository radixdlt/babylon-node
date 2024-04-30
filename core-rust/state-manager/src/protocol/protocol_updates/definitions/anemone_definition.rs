use crate::engine_prelude::*;
use crate::protocol::protocol_updates::definitions::ScryptoEntriesBatchGenerator;
use crate::protocol::*;

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

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        _config: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            ScryptoEntriesBatchGenerator::new(network_definition, &ANEMONE_ENTRIES),
        ))
    }
}
