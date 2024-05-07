use crate::engine_prelude::*;
use crate::protocol::*;

pub struct AnemoneProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for AnemoneProtocolUpdateDefinition {
    type Overrides = ();

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        _config: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            AnemoneSettings::all_enabled_as_default_for_network(network_definition)
                .create_batch_generator(),
        ))
    }
}
