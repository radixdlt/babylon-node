use crate::prelude::*;

pub struct MappingContext {
    pub network_definition: NetworkDefinition,
    pub address_encoder: AddressBech32Encoder,
    pub transaction_hash_encoder: TransactionHashBech32Encoder,
    /// If this is true, then the data (eg transaction data) being mapped can be trusted less, so we need to be more lenient about invalid data
    pub uncommitted_data: bool,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            uncommitted_data: false,
            // sbor_options: Default::default(),
            // transaction_options: Default::default(),
            // substate_options: Default::default(),
        }
    }

    pub fn new_for_uncommitted_data(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            uncommitted_data: true,
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
