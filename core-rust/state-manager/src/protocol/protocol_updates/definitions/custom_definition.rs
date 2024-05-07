use crate::engine_prelude::*;
use crate::protocol::*;

/// Any protocol update beginning `custom-` can have content injected via config.
pub struct CustomProtocolUpdateDefinition;

impl CustomProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "custom-";

    pub fn subnamed(subname: &str) -> ProtocolVersionName {
        ProtocolVersionName::of(format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)).unwrap()
    }

    pub fn matches(protocol_name: &ProtocolVersionName) -> bool {
        protocol_name
            .as_str()
            .starts_with(Self::RESERVED_NAME_PREFIX)
    }
}

impl ProtocolUpdateDefinition for CustomProtocolUpdateDefinition {
    type Overrides = Vec<Vec<UpdateTransaction>>;

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            Self::state_computer_config(network_definition),
            ArbitraryBatchGenerator {
                batches: overrides.unwrap_or_default(),
            },
        ))
    }
}

pub struct ArbitraryBatchGenerator {
    pub batches: Vec<Vec<UpdateTransaction>>,
}

impl UpdateBatchGenerator for ArbitraryBatchGenerator {
    fn generate_transactions(
        &self,
        _store: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>> {
        self.batches.get(batch_index as usize).cloned()
    }
}
