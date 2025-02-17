use crate::prelude::*;

pub struct Formatter {
    address_encoder: AddressBech32Encoder,
    transaction_hash_encoder: TransactionHashBech32Encoder,
}

impl Formatter {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        let address_encoder = AddressBech32Encoder::new(network_definition);
        let hash_encoder = TransactionHashBech32Encoder::new(network_definition);

        Self {
            address_encoder,
            transaction_hash_encoder: hash_encoder,
        }
    }

    pub fn address_encoder(&self) -> &AddressBech32Encoder {
        &self.address_encoder
    }
}

impl<'a> From<&'a Formatter> for ScryptoValueDisplayContext<'a> {
    fn from(formatter: &'a Formatter) -> Self {
        ScryptoValueDisplayContext {
            address_bech32_encoder: Some(&formatter.address_encoder),
        }
    }
}

impl<'a> From<&'a Formatter> for AddressDisplayContext<'a> {
    fn from(formatter: &'a Formatter) -> Self {
        AddressDisplayContext {
            encoder: Some(&formatter.address_encoder),
        }
    }
}

impl<'a> From<&'a Formatter> for TransactionHashDisplayContext<'a> {
    fn from(formatter: &'a Formatter) -> Self {
        TransactionHashDisplayContext {
            encoder: Some(&formatter.transaction_hash_encoder),
        }
    }
}
