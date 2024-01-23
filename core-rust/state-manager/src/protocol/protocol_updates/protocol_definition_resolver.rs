use super::definitions::*;
use crate::protocol::*;
use sbor::DecodeError;

use radix_engine_common::network::NetworkDefinition;
use utils::prelude::*;

pub const GENESIS_PROTOCOL_VERSION: &str = "babylon-genesis";
pub const ANEMONE_PROTOCOL_VERSION: &str = "anemone";

pub struct ProtocolDefinitionResolver {
    network: NetworkDefinition,
    raw_update_config: RawProtocolUpdateContentOverrides,
}

fn resolve_update_definition_for_version(
    protocol_version_name: &str,
) -> Option<Box<dyn ConfigurableProtocolUpdateDefinition>> {
    match protocol_version_name {
        // Genesis execution is done manually.
        // Genesis only needs to be supported here to identify which configuration to use.
        GENESIS_PROTOCOL_VERSION => Some(Box::new(DefaultConfigOnlyProtocolDefinition)),
        ANEMONE_PROTOCOL_VERSION => Some(Box::new(AnemoneProtocolUpdateDefinition)),
        // Updates starting "custom-" are intended for use with tests, where the thresholds and config are injected on all nodes
        _ if CustomProtocolUpdateDefinition::matches(protocol_version_name) => {
            Some(Box::new(CustomProtocolUpdateDefinition))
        }
        _ if TestProtocolUpdateDefinition::matches(protocol_version_name) => {
            Some(Box::new(TestProtocolUpdateDefinition))
        }
        _ => None,
    }
}

impl ProtocolDefinitionResolver {
    pub fn new(network: &NetworkDefinition) -> ProtocolDefinitionResolver {
        Self::new_with_raw_overrides(network, Default::default()).unwrap()
    }

    pub fn new_with_overrides(
        network: &NetworkDefinition,
        update_config: ProtocolUpdateContentOverrides,
    ) -> Result<Self, ConfigValidationError> {
        Self::new_with_raw_overrides(network, update_config.into())
    }

    pub fn new_with_raw_overrides(
        network: &NetworkDefinition,
        update_config: RawProtocolUpdateContentOverrides,
    ) -> Result<Self, ConfigValidationError> {
        // Validate
        for (configured_version, raw_config) in update_config.iter() {
            let updater_factory = resolve_update_definition_for_version(configured_version).ok_or(
                ConfigValidationError::UnknownProtocolVersion(configured_version.to_string()),
            )?;

            updater_factory
                .validate_raw_overrides(raw_config)
                .map_err(|err| {
                    ConfigValidationError::InvalidConfigForProtocolVersion(
                        configured_version.to_string(),
                        err,
                    )
                })?;
        }

        // Return
        Ok(ProtocolDefinitionResolver {
            network: network.clone(),
            raw_update_config: update_config,
        })
    }

    pub fn recognizes(&self, protocol_version_name: &str) -> bool {
        resolve_update_definition_for_version(protocol_version_name).is_some()
    }

    pub fn resolve(
        &self,
        protocol_version_name: &str,
    ) -> Option<(ProtocolStateComputerConfig, Box<dyn ProtocolUpdater>)> {
        let definition = resolve_update_definition_for_version(protocol_version_name)?;

        let config = definition.resolve_state_computer_config(&self.network);

        // Unwrap is allowed because we have already validated the raw config
        let updater = definition
            .create_updater_with_raw_overrides(
                protocol_version_name,
                &self.network,
                self.raw_update_config.get(protocol_version_name),
            )
            .unwrap();

        Some((config, updater))
    }
}

#[derive(Debug)]
pub enum ConfigValidationError {
    UnknownProtocolVersion(String),
    InvalidConfigForProtocolVersion(String, DecodeError),
}
