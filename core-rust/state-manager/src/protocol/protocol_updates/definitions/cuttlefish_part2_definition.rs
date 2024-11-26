use crate::prelude::*;

pub struct CuttlefishPart2ProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for CuttlefishPart2ProtocolUpdateDefinition {
    type Overrides = CuttlefishPart2Settings;

    fn create_batch_generator(
        &self,
        context: ProtocolUpdateContext,
        overrides_hash: Option<Hash>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn NodeProtocolUpdateGenerator> {
        Box::new(create_default_generator_with_scenarios(
            context,
            overrides_hash,
            overrides,
        ))
    }
}
