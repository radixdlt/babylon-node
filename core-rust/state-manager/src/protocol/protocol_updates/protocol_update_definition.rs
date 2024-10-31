use crate::prelude::*;

/// A protocol update definition.
///
/// Note:
/// Currently, protocol updates are only interested in modifying the current ledger state.
/// Consecutive transaction batches to be executed and individually committed are defined by
/// [`Self::create_batch_generator()`].
/// Future protocol updates may additionally want to e.g. modify the configuration of some
/// services (like transaction validation rules). Such customizable parts will have to be
/// represented as other methods on this trait.
pub trait ProtocolUpdateDefinition {
    /// Additional (static) config which can be used to re-configure the updater.
    type Overrides: ScryptoDecode;

    /// Can be overriden for more efficient validation
    fn config_hash(
        &self,
        context: ProtocolUpdateContext,
        overrides_hash: Option<Hash>,
        overrides: Option<Self::Overrides>,
    ) -> Hash {
        self.create_batch_generator(context, overrides_hash, overrides)
            .config_hash()
    }

    /// Returns a provider of on-ledger actions to be executed as part of this protocol update.
    fn create_batch_generator(
        &self,
        context: ProtocolUpdateContext,
        overrides_hash: Option<Hash>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn NodeProtocolUpdateGenerator>;
}

#[derive(Copy, Clone)]
pub struct ProtocolUpdateContext<'a> {
    pub network: &'a NetworkDefinition,
    pub database: &'a Arc<DbLock<ActualStateManagerDatabase>>,
    pub genesis_data_resolver: &'a Arc<dyn ResolveGenesisData>,
    pub scenario_config: &'a ScenariosExecutionConfig,
}

/// A convenience trait for easier validation/parsing of [`ProtocolUpdateDefinition::Overrides`],
/// automatically implemented for all [`ProtocolUpdateDefinition`].
pub trait ConfigurableProtocolUpdateDefinition {
    /// Resolves the configured config hash. This is used to compare against the config
    /// hash stored for enacted protocol updates on boot-up - to detect possible errors
    /// causing by updating the configuration after the update has been enacted.
    fn resolve_config_hash(
        &self,
        context: ProtocolUpdateContext,
        raw_overrides: Option<&[u8]>,
    ) -> Hash;

    /// Parses the given raw overrides and passes them to
    /// [`ProtocolUpdateDefinition::create_batch_generator`].
    /// Panics on any [`DecodeError`] from [`Self::validate_overrides()`].
    fn create_update_generator_raw(
        &self,
        context: ProtocolUpdateContext,
        raw_overrides: Option<&[u8]>,
    ) -> Box<dyn NodeProtocolUpdateGenerator>;

    /// Checks that the given raw overrides can be parsed.
    fn validate_raw_overrides(&self, raw_overrides: &[u8]) -> Result<(), DecodeError>;
}

impl<T: ProtocolUpdateDefinition> ConfigurableProtocolUpdateDefinition for T {
    fn resolve_config_hash(
        &self,
        context: ProtocolUpdateContext,
        raw_overrides: Option<&[u8]>,
    ) -> Hash {
        let overrides = raw_overrides
            .map(scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>)
            .transpose()
            .expect("Raw overrides should have been validated before being passed to this method");

        let overrides_hash = raw_overrides.map(hash);

        self.config_hash(context, overrides_hash, overrides)
    }

    fn create_update_generator_raw(
        &self,
        context: ProtocolUpdateContext,
        raw_overrides: Option<&[u8]>,
    ) -> Box<dyn NodeProtocolUpdateGenerator> {
        let overrides = raw_overrides
            .map(scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>)
            .transpose()
            .expect("Raw overrides should have been validated before being passed to this method");

        let overrides_hash = raw_overrides.map(hash);

        self.create_batch_generator(context, overrides_hash, overrides)
    }

    fn validate_raw_overrides(&self, raw_overrides: &[u8]) -> Result<(), DecodeError> {
        scrypto_decode::<<Self as ProtocolUpdateDefinition>::Overrides>(raw_overrides).map(|_| ())
    }
}
