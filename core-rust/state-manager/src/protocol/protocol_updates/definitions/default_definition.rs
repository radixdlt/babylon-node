use crate::engine_prelude::*;
use crate::protocol::*;
use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::sync::Arc;

pub struct NoOpProtocolDefinition;

impl ProtocolUpdateDefinition for NoOpProtocolDefinition {
    type Overrides = ();

    fn create_action_provider(
        &self,
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateActionProvider> {
        Box::new(EmptyActionProvider)
    }
}

pub struct EmptyActionProvider;

impl ProtocolUpdateActionProvider for EmptyActionProvider {
    fn provide_action(&self, _index: u32) -> Option<ProtocolUpdateAction> {
        None
    }
}
