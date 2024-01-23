use super::by_network::*;
use crate::protocol::*;

use radix_engine_common::network::NetworkDefinition;

pub struct ProtocolConfigResolver;

impl ProtocolConfigResolver {
    pub fn resolve_config(network: &NetworkDefinition) -> ProtocolConfig {
        match network.logical_name.as_str() {
            "mainnet" => mainnet_protocol_config(),
            "stokenet" => stokenet_protocol_config(),
            "dumunet" => dumunet_protocol_config(),
            _ => testnet_protocol_config(),
        }
    }
}
