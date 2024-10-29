use crate::prelude::*;

pub struct NoOpProtocolDefinition;

impl ProtocolUpdateDefinition for NoOpProtocolDefinition {
    type Overrides = ();

    fn create_batch_generator(
        &self,
        _context: ProtocolUpdateContext,
        _overrides_hash: Option<Hash>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn NodeProtocolUpdateGenerator> {
        Box::new(EmptyNodeBatchGenerator)
    }
}

struct EmptyNodeBatchGenerator;

impl NodeProtocolUpdateGenerator for EmptyNodeBatchGenerator {
    fn config_hash(&self) -> Hash {
        Hash([0; Hash::LENGTH])
    }
    
    fn batch_groups(&self) -> Vec<Box<dyn NodeProtocolUpdateBatchGroupGenerator + '_>> {
        vec![]
    }
}
