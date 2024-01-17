mod anemone;

use crate::mainnet_updates::anemone::AnemoneProtocolUpdater;
use crate::{
    NoStateUpdatesProtocolUpdater, ProtocolUpdater, ProtocolUpdaterFactory,
    ANEMONE_PROTOCOL_VERSION, GENESIS_PROTOCOL_VERSION,
};

use radix_engine_common::network::NetworkDefinition;

pub struct ProductionProtocolUpdaterFactory {
    network: NetworkDefinition,
}

impl ProductionProtocolUpdaterFactory {
    pub fn new(network: NetworkDefinition) -> ProductionProtocolUpdaterFactory {
        ProductionProtocolUpdaterFactory { network }
    }
}

impl ProtocolUpdaterFactory for ProductionProtocolUpdaterFactory {
    fn updater_for(&self, protocol_version_name: &str) -> Option<Box<dyn ProtocolUpdater>> {
        match protocol_version_name {
            GENESIS_PROTOCOL_VERSION => Some(Box::new(NoStateUpdatesProtocolUpdater::default(
                self.network.clone(),
            ))),
            ANEMONE_PROTOCOL_VERSION => Some(Box::new(AnemoneProtocolUpdater {
                network: self.network.clone(),
            })),
            _ => None,
        }
    }
}
