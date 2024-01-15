use radix_engine::types::{AddressBech32Decoder, AddressBech32Encoder};
use radix_engine_interface::network::NetworkDefinition;
use transaction::model::{TransactionHashBech32Decoder, TransactionHashBech32Encoder};

use crate::browse_api::models;

pub struct MappingContext {
    pub address_encoder: AddressBech32Encoder,
    pub transaction_hash_encoder: TransactionHashBech32Encoder,
    pub sbor_options: SborOptions,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            sbor_options: Default::default(),
        }
    }

    /// For the transactions stream, we default to settings which the Gateway requires, and aim to
    /// optimize for performance.
    ///
    /// We therefore disable programmatic JSON to cut down on bandwidth for large transaction receipts.
    pub fn new_for_transaction_stream(network_definition: &NetworkDefinition) -> Self {
        Self {
            sbor_options: SborOptions {
                include_raw: true,
                include_programmatic_json: false,
            },
            ..Self::new(network_definition)
        }
    }

    pub fn new_for_uncommitted_data(network_definition: &NetworkDefinition) -> Self {
        Self {
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            sbor_options: Default::default(),
        }
    }

    pub fn with_sbor_formats(
        mut self,
        format_options: &Option<Box<models::SborFormatOptions>>,
    ) -> Self {
        let options = &mut self.sbor_options;
        if let Some(formats) = format_options {
            if let Some(value) = formats.raw {
                options.include_raw = value;
            }
            if let Some(value) = formats.programmatic_json {
                options.include_programmatic_json = value;
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
