use crate::engine_prelude::*;
use crate::protocol::*;
use crate::store::rocks_db::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::sync::Arc;

pub struct NoOpProtocolDefinition;

impl ProtocolUpdateDefinition for NoOpProtocolDefinition {
    type Overrides = ();

    fn create_batch_generator(
        &self,
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        Box::new(EmptyNodeBatchGenerator)
    }
}

struct EmptyNodeBatchGenerator;

impl ProtocolUpdateNodeBatchGenerator for EmptyNodeBatchGenerator {
    fn generate_batch(&self, _batch_idx: u32) -> ProtocolUpdateNodeBatch {
        panic!("no batches")
    }

    fn batch_count(&self) -> u32 {
        0
    }
}
