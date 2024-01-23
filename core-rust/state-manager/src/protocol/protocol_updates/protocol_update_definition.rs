// This file contains the protocol update logic for specific protocol versions

use crate::protocol::*;
use radix_engine::transaction::CostingParameters;
use radix_engine::types::*;

use crate::transaction::*;
use crate::LoggingConfig;
use transaction::validation::{NotarizedTransactionValidator, ValidationConfig};

/// A protocol update definition consists of two parts:
/// 1) Updating the current (state computer) configuration ("transaction processing rules").
///    This includes: transaction validation, execution configuration, etc
/// 2) Executing arbitrary state updates against the current database state.
///    While the abstraction is quite flexible, the only concrete implementation at the moment
///    only modifies the state through committing system transactions (e.g. substate flash).
pub trait ProtocolUpdateDefinition {
    /// Additional (static) config which can be used to re-configure the updater.
    type Overrides: ScryptoDecode;

    /// Returns the new configuration that the state computer
    /// should use after enacting the given protocol version.
    fn state_computer_config(network_definition: &NetworkDefinition)
        -> ProtocolStateComputerConfig;

    fn create_updater(
        new_protocol_version: &str,
        network_definition: &NetworkDefinition,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater>;
}

pub trait ConfigurableProtocolUpdateDefinition {
    fn resolve_state_computer_config(
        &self,
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig;

    /// Must return `Ok` if the `raw_config` is None or passed validate.
    fn create_updater_with_raw_overrides(
        &self,
        new_protocol_version: &str,
        network_definition: &NetworkDefinition,
        raw_config: Option<&[u8]>,
    ) -> Result<Box<dyn ProtocolUpdater>, DecodeError>;

    fn validate_raw_overrides(&self, raw_config: &[u8]) -> Result<(), DecodeError>;
}

impl<T: ProtocolUpdateDefinition> ConfigurableProtocolUpdateDefinition for T {
    fn resolve_state_computer_config(
        &self,
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        Self::state_computer_config(network_definition)
    }

    /// If no raw config is provided, the default config is used
    fn create_updater_with_raw_overrides(
        &self,
        new_protocol_version: &str,
        network_definition: &NetworkDefinition,
        raw_overrides: Option<&[u8]>,
    ) -> Result<Box<dyn ProtocolUpdater>, DecodeError> {
        let overrides = raw_overrides
            .map(scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>)
            .transpose()?;

        Ok(Self::create_updater(
            new_protocol_version,
            network_definition,
            overrides,
        ))
    }

    fn validate_raw_overrides(&self, raw_config: &[u8]) -> Result<(), DecodeError> {
        scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>(raw_config).map(|_| ())
    }
}

#[derive(Clone, Debug)]
pub struct ProtocolStateComputerConfig {
    pub network: NetworkDefinition,
    pub logging_config: LoggingConfig,
    pub validation_config: ValidationConfig,
    pub costing_parameters: CostingParameters,
}

impl ProtocolStateComputerConfig {
    pub fn default(network: NetworkDefinition) -> ProtocolStateComputerConfig {
        let network_id = network.id;
        ProtocolStateComputerConfig {
            network,
            logging_config: LoggingConfig::default(),
            validation_config: ValidationConfig::default(network_id),
            costing_parameters: CostingParameters::default(),
        }
    }
}

impl ProtocolStateComputerConfig {
    pub fn ledger_transaction_validator(&self) -> LedgerTransactionValidator {
        LedgerTransactionValidator::default_from_validation_config(self.validation_config)
    }

    pub fn user_transaction_validator(&self) -> NotarizedTransactionValidator {
        NotarizedTransactionValidator::new(self.validation_config)
    }

    pub fn validation_config(&self) -> ValidationConfig {
        self.validation_config
    }

    pub fn execution_configurator(&self, no_fees: bool) -> ExecutionConfigurator {
        let mut costing_parameters = self.costing_parameters;
        if no_fees {
            costing_parameters.execution_cost_unit_price = Decimal::ZERO;
            costing_parameters.finalization_cost_unit_price = Decimal::ZERO;
            costing_parameters.state_storage_price = Decimal::ZERO;
            costing_parameters.archive_storage_price = Decimal::ZERO;
        }
        ExecutionConfigurator::new(&self.network, &self.logging_config, costing_parameters)
    }
}
