use crate::engine_prelude::*;
use crate::protocol::*;
use crate::store::consensus::rocks_db::ActualStateManagerDatabase;
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
    type Overrides = Vec<ProtocolUpdateNodeBatch>;

    fn create_batch_generator(
        &self,
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        Box::new(ArbitraryNodeBatchGenerator {
            batches: overrides.unwrap_or_default(),
        })
    }
}

pub struct ArbitraryNodeBatchGenerator {
    pub batches: Vec<ProtocolUpdateNodeBatch>,
}

impl ProtocolUpdateNodeBatchGenerator for ArbitraryNodeBatchGenerator {
    fn generate_batch(&self, batch_idx: u32) -> ProtocolUpdateNodeBatch {
        self.batches.get(batch_idx as usize).unwrap().clone()
    }

    fn batch_count(&self) -> u32 {
        u32::try_from(self.batches.len()).unwrap()
    }
}
