// This file contains the protocol update logic for specific protocol versions

use crate::engine_prelude::*;
use crate::protocol::*;
use crate::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::sync::Arc;

/// A protocol update definition.
///
/// Note:
/// Currently, protocol updates are only interested in modifying the current ledger state.
/// Consecutive "actions" to be executed and individually committed are defined by
/// [`Self::create_action_provider()`].
/// Future protocol updates may additionally want to e.g. modify the configuration of some
/// services (like transaction validation rules). Such customizable parts will have to be
/// represented as other methods on this trait.
pub trait ProtocolUpdateDefinition {
    /// Additional (static) config which can be used to re-configure the updater.
    type Overrides: ScryptoDecode;

    /// Returns a provider of on-ledger actions to be executed as part of this protocol update.
    fn create_action_provider(
        &self,
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateActionProvider>;
}

/// A convenience trait for easier validation/parsing of [`ProtocolUpdateDefinition::Overrides`],
/// automatically implemented for all [`ProtocolUpdateDefinition`].
pub trait ConfigurableProtocolUpdateDefinition {
    /// Parses the given raw overrides and passes them to [`Self::create_action_provider`].
    /// Panics on any [`DecodeError`] from [`Self::validate_overrides()`].
    fn create_action_provider_raw(
        &self,
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        raw_overrides: Option<&[u8]>,
    ) -> Box<dyn ProtocolUpdateActionProvider>;

    /// Checks that the given raw overrides can be parsed.
    fn validate_raw_overrides(&self, raw_overrides: &[u8]) -> Result<(), DecodeError>;
}

impl<T: ProtocolUpdateDefinition> ConfigurableProtocolUpdateDefinition for T {
    fn create_action_provider_raw(
        &self,
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        raw_overrides: Option<&[u8]>,
    ) -> Box<dyn ProtocolUpdateActionProvider> {
        let overrides = raw_overrides
            .map(scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>)
            .transpose()
            .expect("Raw overrides should have been validated before being passed to this method");

        self.create_action_provider(network, database, overrides)
    }

    fn validate_raw_overrides(&self, raw_overrides: &[u8]) -> Result<(), DecodeError> {
        scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>(raw_overrides).map(|_| ())
    }
}
