use radix_engine::types::{Bech32Decoder, Bech32Encoder};
use radix_engine_interface::node::NetworkDefinition;

pub struct MappingContext {
    pub network_definition: NetworkDefinition,
    pub bech32_encoder: Bech32Encoder,
    /// If this is true, then the data (eg transaction data) being mapped can be trusted less, so we need to be more lenient about invalid data
    pub uncommitted_data: bool,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            bech32_encoder: Bech32Encoder::new(network_definition),
            uncommitted_data: false,
        }
    }

    pub fn new_for_uncommitted_data(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            bech32_encoder: Bech32Encoder::new(network_definition),
            uncommitted_data: true,
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
