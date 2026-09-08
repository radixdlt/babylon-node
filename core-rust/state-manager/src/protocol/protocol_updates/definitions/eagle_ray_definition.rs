use crate::prelude::*;

pub struct EagleRayProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for EagleRayProtocolUpdateDefinition {
    type Overrides = EagleRaySettings;

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
