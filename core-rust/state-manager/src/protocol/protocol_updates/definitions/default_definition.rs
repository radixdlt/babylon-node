use crate::prelude::*;

pub struct NoOpProtocolDefinition;

impl ProtocolUpdateDefinition for NoOpProtocolDefinition {
    type Overrides = ();

    fn create_batch_generator(
        &self,
        _context: ProtocolUpdateContext,
        _overrides_hash: Option<Hash>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        Box::new(EmptyNodeBatchGenerator)
    }
}

struct EmptyNodeBatchGenerator;

impl ProtocolUpdateNodeBatchGenerator for EmptyNodeBatchGenerator {
    fn config_hash(&self) -> Hash {
        Hash([0; Hash::LENGTH])
    }

    fn generate_batch(
        &self,
        _batch_group_index: usize,
        _batch_index: usize,
    ) -> ProtocolUpdateNodeBatch {
        panic!("no batches")
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        vec![]
    }

    fn batch_count(&self, _batch_group_index: usize) -> usize {
        0
    }
}
