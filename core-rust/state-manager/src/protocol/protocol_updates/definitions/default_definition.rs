use crate::prelude::*;

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
    fn generate_batch(&self, _batch_idx: usize) -> ProtocolUpdateNodeBatch {
        panic!("no batches")
    }

    fn batch_count(&self) -> usize {
        0
    }
}
