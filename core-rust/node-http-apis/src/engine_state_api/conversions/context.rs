use radix_engine::types::{AddressBech32Decoder, AddressBech32Encoder};
use radix_engine_interface::network::NetworkDefinition;
use transaction::model::{TransactionHashBech32Decoder, TransactionHashBech32Encoder};

pub struct MappingContext {
    pub address_encoder: AddressBech32Encoder,
    pub transaction_hash_encoder: TransactionHashBech32Encoder,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
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
