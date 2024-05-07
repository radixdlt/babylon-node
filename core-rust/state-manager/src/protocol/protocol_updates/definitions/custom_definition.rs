use crate::engine_prelude::*;
use crate::protocol::*;
use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::sync::Arc;

/// Any protocol update beginning `custom-` can have content injected via config.
pub struct CustomProtocolUpdateDefinition;

impl CustomProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "custom-";

    pub fn subnamed(subname: &str) -> ProtocolVersionName {
        ProtocolVersionName::of(format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)).unwrap()
    }

    pub fn matches(name_string: &str) -> bool {
        name_string.starts_with(Self::RESERVED_NAME_PREFIX)
    }
}

impl ProtocolUpdateDefinition for CustomProtocolUpdateDefinition {
    type Overrides = Vec<ProtocolUpdateAction>;

    fn create_action_provider(
        &self,
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateActionProvider> {
        Box::new(ArbitraryActionProvider {
            batches: overrides.unwrap_or_default(),
        })
    }
}

pub struct ArbitraryActionProvider {
    pub batches: Vec<ProtocolUpdateAction>,
}

impl ProtocolUpdateActionProvider for ArbitraryActionProvider {
    fn provide_action(&self, index: u32) -> Option<ProtocolUpdateAction> {
        self.batches.get(index as usize).cloned()
    }
}
