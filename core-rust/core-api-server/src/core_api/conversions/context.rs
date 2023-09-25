use radix_engine::types::{AddressBech32Decoder, AddressBech32Encoder};
use radix_engine_interface::network::NetworkDefinition;
use transaction::model::{TransactionHashBech32Decoder, TransactionHashBech32Encoder};

use crate::core_api::models;

pub struct MappingContext {
    pub network_definition: NetworkDefinition,
    pub address_encoder: AddressBech32Encoder,
    pub transaction_hash_encoder: TransactionHashBech32Encoder,
    /// If this is true, then the data (eg transaction data) being mapped can be trusted less, so we need to be more lenient about invalid data
    pub uncommitted_data: bool,
    pub sbor_options: SborOptions,
    pub transaction_options: TransactionOptions,
    pub substate_options: SubstateOptions,
}

impl MappingContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            uncommitted_data: false,
            sbor_options: Default::default(),
            transaction_options: Default::default(),
            substate_options: Default::default(),
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
            network_definition: network_definition.clone(),
            address_encoder: AddressBech32Encoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(network_definition),
            uncommitted_data: true,
            sbor_options: Default::default(),
            transaction_options: Default::default(),
            substate_options: Default::default(),
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

    pub fn with_transaction_formats(
        mut self,
        format_options: &Option<Box<models::TransactionFormatOptions>>,
    ) -> Self {
        let options = &mut self.transaction_options;
        if let Some(formats) = format_options {
            if let Some(value) = formats.manifest {
                options.include_manifest = value;
            }
            if let Some(value) = formats.blobs {
                options.include_blobs = value;
            }
            if let Some(value) = formats.message {
                options.include_message = value;
            }
            if let Some(value) = formats.raw_system_transaction {
                options.include_raw_system = value;
            }
            if let Some(value) = formats.raw_notarized_transaction {
                options.include_raw_notarized = value;
            }
            if let Some(value) = formats.raw_ledger_transaction {
                options.include_raw_ledger = value;
            }
        }
        self
    }

    pub fn with_substate_formats(
        mut self,
        format_options: &Option<Box<models::SubstateFormatOptions>>,
    ) -> Self {
        let options = &mut self.substate_options;
        if let Some(formats) = format_options {
            if let Some(value) = formats.hash {
                options.include_hash = value;
            }
            if let Some(value) = formats.raw {
                options.include_raw = value;
            }
            if let Some(value) = formats.typed {
                options.include_typed = value;
            }
            if let Some(value) = formats.previous {
                options.include_previous = value;
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

pub struct TransactionOptions {
    pub include_manifest: bool,
    pub include_blobs: bool,
    pub include_message: bool,
    pub include_raw_system: bool,
    pub include_raw_notarized: bool,
    pub include_raw_ledger: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            include_manifest: true,
            include_blobs: false,
            include_message: true,
            include_raw_system: false,
            include_raw_notarized: true,
            include_raw_ledger: false,
        }
    }
}

pub struct SubstateOptions {
    pub include_raw: bool,
    pub include_hash: bool,
    pub include_typed: bool,
    pub include_previous: bool,
}

impl Default for SubstateOptions {
    fn default() -> Self {
        Self {
            include_raw: false,
            include_hash: false,
            include_typed: true,
            include_previous: false,
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
