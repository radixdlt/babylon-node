mod anemone;

use crate::mainnet_updates::anemone::AnemoneProtocolUpdater;
use crate::{
    NoStateUpdatesProtocolUpdater, ProtocolUpdater, ProtocolUpdaterFactory, StateManagerDatabase,
    ANEMONE_PROTOCOL_VERSION, GENESIS_PROTOCOL_VERSION,
};
use node_common::locks::StateLock;
use radix_engine_common::network::NetworkDefinition;
use std::sync::Arc;

pub struct MainnetProtocolUpdaterFactory {
    network: NetworkDefinition,
}

impl MainnetProtocolUpdaterFactory {
    pub fn new(network: NetworkDefinition) -> MainnetProtocolUpdaterFactory {
        MainnetProtocolUpdaterFactory { network }
    }
}

impl ProtocolUpdaterFactory for MainnetProtocolUpdaterFactory {
    fn supports_protocol_version(&self, protocol_version_name: &str) -> bool {
        [GENESIS_PROTOCOL_VERSION, ANEMONE_PROTOCOL_VERSION].contains(&protocol_version_name)
    }

    fn updater_for(
        &self,
        protocol_version_name: &str,
        store: Arc<StateLock<StateManagerDatabase>>,
    ) -> Box<dyn ProtocolUpdater> {
        match protocol_version_name {
            GENESIS_PROTOCOL_VERSION => {
                Box::new(NoStateUpdatesProtocolUpdater::default(self.network.clone()))
            }
            ANEMONE_PROTOCOL_VERSION => Box::new(AnemoneProtocolUpdater {
                network: self.network.clone(),
                store,
            }),
            _ => panic!("Unknown protocol version {:?}", protocol_version_name),
        }
    }
}
