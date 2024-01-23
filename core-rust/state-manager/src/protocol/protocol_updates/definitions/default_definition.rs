use crate::protocol::*;
use radix_engine::types::*;

pub struct DefaultConfigOnlyProtocolDefinition;

impl ProtocolUpdateDefinition for DefaultConfigOnlyProtocolDefinition {
    type Overrides = ();

    fn create_updater(
        _new_protocol_version: &ProtocolVersionName,
        _network_definition: &NetworkDefinition,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(NoOpProtocolUpdater)
    }

    fn state_computer_config(
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        ProtocolStateComputerConfig::default(network_definition.clone())
    }
}
