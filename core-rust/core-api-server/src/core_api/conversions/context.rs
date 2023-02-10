use radix_engine::types::{Bech32Decoder, Bech32Encoder};
use radix_engine_interface::network::NetworkDefinition;

pub struct MappingContext {
    pub network_definition: NetworkDefinition,
    pub bech32_encoder: Bech32Encoder,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            bech32_encoder: Bech32Encoder::new(network_definition),
        }
    }
}

pub struct ExtractionContext {
    pub bech32_decoder: Bech32Decoder,
}

impl ExtractionContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            bech32_decoder: Bech32Decoder::new(network_definition),
        }
    }
}
