use radix_engine::types::{AddressBech32Decoder, AddressBech32Encoder};
use radix_engine_interface::network::NetworkDefinition;

use crate::core_api::models;

pub struct MappingContext {
    pub network_definition: NetworkDefinition,
    pub bech32_encoder: AddressBech32Encoder,
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
            bech32_encoder: AddressBech32Encoder::new(network_definition),
            uncommitted_data: false,
            sbor_options: Default::default(),
            transaction_options: Default::default(),
            substate_options: Default::default(),
        }
    }

    pub fn new_for_uncommitted_data(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
            bech32_encoder: AddressBech32Encoder::new(network_definition),
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
        let mut options = &mut self.sbor_options;
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
        let mut options = &mut self.transaction_options;
        if let Some(formats) = format_options {
            if let Some(value) = formats.manifest {
                options.include_manifest = value;
            }
            if let Some(value) = formats.blobs {
                options.include_blobs = value;
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
        let mut options = &mut self.substate_options;
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
    pub include_raw_system: bool,
    pub include_raw_notarized: bool,
    pub include_raw_ledger: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            include_manifest: true,
            include_blobs: false,
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
    pub bech32_decoder: AddressBech32Decoder,
}

impl ExtractionContext {
    pub fn new(network_definition: &NetworkDefinition) -> Self {
        Self {
            bech32_decoder: AddressBech32Decoder::new(network_definition),
        }
    }
}
