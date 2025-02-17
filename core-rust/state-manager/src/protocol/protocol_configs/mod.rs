#[cfg(test)]
mod config_printer;
mod dumunet_protocol_config;
mod mainnet_protocol_config;
mod stokenet_protocol_config;
mod testnet_protocol_config;

use crate::engine_prelude::*;
use crate::protocol::*;

pub fn resolve_protocol_config(network: &NetworkDefinition) -> ProtocolConfig {
    match network.logical_name.as_ref() {
        "mainnet" => mainnet_protocol_config::mainnet_protocol_config(),
        "stokenet" => stokenet_protocol_config::stokenet_protocol_config(),
        "dumunet" => dumunet_protocol_config::dumunet_protocol_config(),
        _ => testnet_protocol_config::testnet_protocol_config(),
    }
}
