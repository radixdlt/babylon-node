// This file contains the protocol update logic for specific protocol versions

use crate::engine_prelude::*;
use crate::protocol::*;

/// A protocol update definition consists of two parts:
/// 1) Updating the current (state computer) configuration ("transaction processing rules").
///    This includes: transaction validation, execution configuration, etc
/// 2) Executing arbitrary state updates against the current database state.
///    While the abstraction is quite flexible, the only concrete implementation at the moment
///    only modifies the state through committing system transactions (e.g. substate flash).
pub trait ProtocolUpdateDefinition {
    /// Additional (static) config which can be used to re-configure the updater.
    type Overrides: ScryptoDecode;

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater>;
}

pub trait ConfigurableProtocolUpdateDefinition {
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
