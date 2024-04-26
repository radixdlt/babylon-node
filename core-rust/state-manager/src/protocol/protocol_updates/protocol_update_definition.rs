// This file contains the protocol update logic for specific protocol versions

use crate::engine_prelude::*;
use crate::protocol::*;

use crate::transaction::*;

/// A protocol update definition consists of two parts:
/// 1) Updating the current (state computer) configuration ("transaction processing rules").
///    This includes: transaction validation, execution configuration, etc
/// 2) Executing arbitrary state updates against the current database state.
///    While the abstraction is quite flexible, the only concrete implementation at the moment
///    only modifies the state through committing system transactions (e.g. substate flash).
pub trait ProtocolUpdateDefinition {
    /// Additional (static) config which can be used to re-configure the updater.
    type Overrides: ScryptoDecode;

    /// Returns the new configuration that the state computer should use after enacting the given
    /// protocol version.
    fn state_computer_config(
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        ProtocolStateComputerConfig::default(network_definition.clone())
    }

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater>;
}

pub trait ConfigurableProtocolUpdateDefinition {
    fn resolve_state_computer_config(
        &self,
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig;

    /// This method panics if the `raw_overrides` is present and invalid.
    /// A caller should have first validated with validate_raw_overrides.
    fn create_updater_with_raw_overrides(
        &self,
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        raw_overrides: Option<&[u8]>,
    ) -> Box<dyn ProtocolUpdater>;

    fn validate_raw_overrides(&self, raw_overrides: &[u8]) -> Result<(), DecodeError>;
}

impl<T: ProtocolUpdateDefinition> ConfigurableProtocolUpdateDefinition for T {
    fn resolve_state_computer_config(
        &self,
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        Self::state_computer_config(network_definition)
    }

    fn create_updater_with_raw_overrides(
        &self,
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        raw_overrides: Option<&[u8]>,
    ) -> Box<dyn ProtocolUpdater> {
        let overrides = raw_overrides.map(|overrides| {
            scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>(overrides).expect(
                "Raw overrides should have been validated before being passed to this method",
            )
        });

        Self::create_updater(new_protocol_version, network_definition, overrides)
    }

    fn validate_raw_overrides(&self, raw_overrides: &[u8]) -> Result<(), DecodeError> {
        scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>(raw_overrides).map(|_| ())
    }
}

#[derive(Clone, Debug)]
pub struct ProtocolStateComputerConfig {
    pub network: NetworkDefinition,
    pub validation_config: ValidationConfig,
}

impl ProtocolStateComputerConfig {
    pub fn default(network: NetworkDefinition) -> ProtocolStateComputerConfig {
        let network_id = network.id;
        ProtocolStateComputerConfig {
            network,
            validation_config: ValidationConfig::default(network_id),
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

    pub fn execution_configurator(
        &self,
        no_fees: bool,
        engine_trace: bool,
    ) -> ExecutionConfigurator {
        ExecutionConfigurator::new(&self.network, no_fees, engine_trace)
    }
}
