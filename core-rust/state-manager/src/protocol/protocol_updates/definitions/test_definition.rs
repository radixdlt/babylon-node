use crate::{protocol::*, transaction::FlashTransactionV1};
use radix_engine::{track::StateUpdates, types::*};
use radix_engine_store_interface::interface::SubstateDatabase;

/// Any protocol update beginning `test-` just injects a single transaction.
pub struct TestProtocolUpdateDefinition;

impl TestProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "test-";

    pub fn subnamed(subname: &str) -> String {
        format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)
    }

    pub fn matches(protocol_name: &str) -> bool {
        protocol_name.starts_with(Self::RESERVED_NAME_PREFIX)
    }
}

impl ProtocolUpdateDefinition for TestProtocolUpdateDefinition {
    type Overrides = ();

    fn create_updater(
        new_protocol_version: &str,
        network_definition: &NetworkDefinition,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.to_string(),
            Self::state_computer_config(network_definition),
            TestBatchGenerator {
                protocol_version_name: new_protocol_version.to_string(),
            },
        ))
    }

    fn state_computer_config(
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        ProtocolStateComputerConfig::default(network_definition.clone())
    }
}

struct TestBatchGenerator {
    protocol_version_name: String,
}

impl UpdateBatchGenerator for TestBatchGenerator {
    fn generate_batch(
        &self,
        _store: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>> {
        match batch_index {
            0 => Some(vec![UpdateTransaction::FlashTransactionV1(
                FlashTransactionV1 {
                    name: format!("{}-txn", &self.protocol_version_name),
                    state_updates: StateUpdates::default(),
                },
            )]),
            _ => None,
        }
    }
}
