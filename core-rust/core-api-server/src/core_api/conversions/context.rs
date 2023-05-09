use radix_engine::types::{Bech32Decoder, Bech32Encoder};
use radix_engine_interface::network::NetworkDefinition;

use crate::core_api::models;

pub struct MappingContext {
    pub network_definition: NetworkDefinition,
    pub bech32_encoder: Bech32Encoder,
    /// If this is true, then the data (eg transaction data) being mapped can be trusted less, so we need to be more lenient about invalid data
    pub uncommitted_data: bool,
    pub sbor_options: SborOptions,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            bech32_encoder: Bech32Encoder::new(network_definition),
            uncommitted_data: false,
            sbor_options: Default::default(),
        }
    }

    pub fn new_for_uncommitted_data(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            bech32_encoder: Bech32Encoder::new(network_definition),
            uncommitted_data: true,
            sbor_options: Default::default(),
        }
    }

    pub fn with_sbor_formats(mut self, options: &Option<Box<models::SborFormats>>) -> Self {
        if let Some(formats) = options {
            if let Some(raw_hex) = formats.raw_hex {
                self.sbor_options.include_raw = raw_hex;
            }
            if let Some(programmatic_json) = formats.programmatic_json {
                self.sbor_options.include_programmatic_json = programmatic_json;
            }
        }
        self
    }
}

pub struct SborOptions {
    pub include_raw: bool,
    pub include_programmatic_json: bool,
}

impl Default for SborOptions {
    fn default() -> Self {
        Self {
            include_raw: true,
            include_programmatic_json: true,
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
