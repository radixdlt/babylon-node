use crate::prelude::*;

pub struct CuttlefishPart1ProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for CuttlefishPart1ProtocolUpdateDefinition {
    type Overrides = CuttlefishPart1Settings;

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
