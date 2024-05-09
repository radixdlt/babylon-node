use crate::engine_prelude::*;
use crate::{protocol::*, transaction::FlashTransactionV1};

/// Any protocol update beginning `test-` just injects a single transaction.
pub struct TestProtocolUpdateDefinition;

impl TestProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "test-";

    pub fn subnamed(subname: &str) -> ProtocolVersionName {
        ProtocolVersionName::of(format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)).unwrap()
    }

    pub fn matches(protocol_name: &ProtocolVersionName) -> bool {
        protocol_name
            .as_str()
            .starts_with(Self::RESERVED_NAME_PREFIX)
    }
}

impl ProtocolUpdateDefinition for TestProtocolUpdateDefinition {
    type Overrides = ();

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        _network_definition: &NetworkDefinition,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            ArbitraryBatchGenerator {
                batches: vec![vec![FlashTransactionV1 {
                    name: format!("{}-txn", &new_protocol_version),
                    state_updates: StateUpdates::default(),
                }
                .into()]],
            },
        ))
    }
}
