use crate::engine_prelude::*;
use crate::ActualStateManagerDatabase;
use crate::{protocol::*, transaction::FlashTransactionV1};
use node_common::locks::DbLock;
use std::sync::Arc;

/// Any protocol update beginning `test-` just injects a single transaction.
pub struct TestProtocolUpdateDefinition {
    protocol_name: ProtocolVersionName,
}

impl TestProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "test-";

    pub fn subnamed(subname: &str) -> ProtocolVersionName {
        ProtocolVersionName::of(format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)).unwrap()
    }

    pub fn matches(name_string: &str) -> bool {
        name_string.starts_with(Self::RESERVED_NAME_PREFIX)
    }

    pub fn new(protocol_name: ProtocolVersionName) -> Self {
        if !Self::matches(protocol_name.as_str()) {
            panic!("not a test name");
        }
        Self { protocol_name }
    }
}

impl ProtocolUpdateDefinition for TestProtocolUpdateDefinition {
    type Overrides = ();

    fn create_action_provider(
        &self,
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateActionProvider> {
        Box::new(ArbitraryActionProvider {
            batches: vec![ProtocolUpdateAction::FlashTransactions(vec![
                FlashTransactionV1 {
                    name: format!("{}-txn", self.protocol_name),
                    state_updates: StateUpdates::default(),
                },
            ])],
        })
    }
}
