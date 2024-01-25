use crate::engine_state_api::models::SborFormatOptions;
use radix_engine::types::{AddressBech32Decoder, AddressBech32Encoder};
use radix_engine_interface::network::NetworkDefinition;
use transaction::model::{TransactionHashBech32Decoder, TransactionHashBech32Encoder};

pub struct MappingContext {
    pub address_encoder: AddressBech32Encoder,
    pub transaction_hash_encoder: TransactionHashBech32Encoder,
    pub sbor_options: SborOptions,
}

pub struct SborOptions {
    pub include_raw_hex: bool,
    pub include_programmatic_json: bool,
}

impl Default for SborOptions {
    fn default() -> Self {
        Self {
            include_raw_hex: false,
            include_programmatic_json: true,
        }
    }
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            sbor_options: SborOptions::default(),
        }
    }

    pub fn with_sbor_formats(mut self, requested_options: Option<Box<SborFormatOptions>>) -> Self {
        if let Some(requested_options) = requested_options {
            let options = &mut self.sbor_options;
            if let Some(value) = requested_options.raw_hex {
                options.include_raw_hex = value;
            }
            if let Some(value) = requested_options.programmatic_json {
                options.include_programmatic_json = value;
            }
        }
        self
    }
}

pub struct ExtractionContext {
    pub address_decoder: AddressBech32Decoder,
    pub transaction_hash_decoder: TransactionHashBech32Decoder,
}

impl ExtractionContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            address_decoder: AddressBech32Decoder::new(network_definition),
            transaction_hash_decoder: TransactionHashBech32Decoder::new(network_definition),
        }
    }
}
