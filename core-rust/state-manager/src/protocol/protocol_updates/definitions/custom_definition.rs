use crate::protocol::*;
use radix_engine::types::*;
use radix_engine_store_interface::interface::SubstateDatabase;

/// Any protocol update beginning `custom-` can have content injected via config.
pub struct CustomProtocolUpdateDefinition;

impl CustomProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "custom-";

    pub fn subnamed(subname: &str) -> String {
        format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)
    }

    pub fn matches(protocol_name: &str) -> bool {
        protocol_name.starts_with(Self::RESERVED_NAME_PREFIX)
    }
}

impl ProtocolUpdateDefinition for CustomProtocolUpdateDefinition {
    type Overrides = Vec<Vec<UpdateTransaction>>;

    fn create_updater(
        new_protocol_version: &str,
        network_definition: &NetworkDefinition,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.to_string(),
            Self::state_computer_config(network_definition),
            ArbitraryBatchGenerator {
                batches: overrides.unwrap_or_default(),
            },
        ))
    }

    fn state_computer_config(
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        ProtocolStateComputerConfig::default(network_definition.clone())
    }
}

struct ArbitraryBatchGenerator {
    batches: Vec<Vec<UpdateTransaction>>,
}

impl UpdateBatchGenerator for ArbitraryBatchGenerator {
    fn generate_batch(
        &self,
        _store: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>> {
        self.batches.get(batch_index as usize).cloned()
    }
}
