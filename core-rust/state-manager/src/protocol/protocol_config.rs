use crate::engine_prelude::*;

use crate::protocol::*;

// This file contains types for node's local static protocol configuration

const MIN_PROTOCOL_VERSION_NAME_LEN: usize = 2;
const MAX_PROTOCOL_VERSION_NAME_LEN: usize = 16;

const GENESIS_PROTOCOL_VERSION: &str = "babylon-genesis";
const ANEMONE_PROTOCOL_VERSION: &str = "anemone";
const BOTTLENOSE_PROTOCOL_VERSION: &str = "bottlenose";
const CUTTLEFISH_PROTOCOL_VERSION: &str = "cuttlefish";

pub enum ResolvedProtocolVersion {
    Babylon,
    Anemone,
    Bottlenose,
    Cuttlefish,
    Custom(ProtocolVersionName),
    Test(ProtocolVersionName),
}

impl ResolvedProtocolVersion {
    pub fn of(protocol_version_name: &ProtocolVersionName) -> Option<Self> {
        match protocol_version_name.as_str() {
            GENESIS_PROTOCOL_VERSION => Some(ResolvedProtocolVersion::Babylon),
            ANEMONE_PROTOCOL_VERSION => Some(ResolvedProtocolVersion::Anemone),
            BOTTLENOSE_PROTOCOL_VERSION => Some(ResolvedProtocolVersion::Bottlenose),
            CUTTLEFISH_PROTOCOL_VERSION => Some(ResolvedProtocolVersion::Cuttlefish),
            // Updates starting "custom-" are intended for use with tests, where the thresholds and config are injected on all nodes
            name_string if CustomProtocolUpdateDefinition::matches(name_string) => Some(
                ResolvedProtocolVersion::Custom(protocol_version_name.clone()),
            ),
            // Updates starting "test-" are intended for use with tests, where it uses a dummy update
            name_string if TestProtocolUpdateDefinition::matches(name_string) => {
                Some(ResolvedProtocolVersion::Test(protocol_version_name.clone()))
            }
            _ => None,
        }
    }

    pub fn resolve_engine_version(&self) -> Option<ProtocolVersion> {
        match self {
            ResolvedProtocolVersion::Babylon => Some(ProtocolVersion::Babylon),
            ResolvedProtocolVersion::Anemone => Some(ProtocolVersion::Anemone),
            ResolvedProtocolVersion::Bottlenose => Some(ProtocolVersion::Bottlenose),
            ResolvedProtocolVersion::Cuttlefish => Some(ProtocolVersion::Cuttlefish),
            ResolvedProtocolVersion::Custom { .. } => None,
            ResolvedProtocolVersion::Test { .. } => None,
        }
    }

    pub fn definition(&self) -> Box<dyn ConfigurableProtocolUpdateDefinition> {
        match self {
            ResolvedProtocolVersion::Babylon => Box::new(BabylonProtocolUpdateDefinition),
            ResolvedProtocolVersion::Anemone => Box::new(AnemoneProtocolUpdateDefinition),
            ResolvedProtocolVersion::Bottlenose => Box::new(BottlenoseProtocolUpdateDefinition),
            ResolvedProtocolVersion::Cuttlefish => Box::new(CuttlefishProtocolUpdateDefinition),
            ResolvedProtocolVersion::Custom(..) => Box::new(CustomProtocolUpdateDefinition),
            ResolvedProtocolVersion::Test(name) => {
                Box::new(TestProtocolUpdateDefinition::new(name.clone()))
            }
        }
    }
}

/// The `ProtocolConfig` is a static configuration provided per-network, or overriden for testing.
///
/// When a node commits (or creates a proof), it checks `protocol_update_triggers` to see if a protocol update
/// should be triggered next.
///
/// If the update is triggered, any relevant overrides are combined with the `protocol_definition_resolver` to
/// work out what it should do for the update.
#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProtocolConfig {
    pub genesis_protocol_version: ProtocolVersionName,
    pub protocol_update_triggers: Vec<ProtocolUpdateTrigger>,
    /// This allows overriding the configuration of a protocol update.
    ///
    /// You can create this with `ProtocolUpdateContentOverrides::empty().into()` or `Default::default()`.
    ///
    /// This essentially wraps an optional `Map<String, Vec<u8>>` where the `Vec<u8>` is the encoded config
    /// for the protocol updater matching the given protocol update.
    ///
    /// All nodes must agree on the content overrides used. The content overrides form a part of
    /// the definition of the protocol update, and if nodes use different overrides, they will execute
    /// different updates and need manual recovery.
    pub protocol_update_content_overrides: RawProtocolUpdateContentOverrides,
}

impl ProtocolConfig {
    pub fn new_with_no_updates() -> Self {
        Self::new_with_triggers([])
    }

    pub fn new_with_triggers(
        triggers: impl IntoIterator<Item = (ProtocolVersionName, ProtocolUpdateEnactmentCondition)>,
    ) -> Self {
        Self {
            genesis_protocol_version: ProtocolVersionName::babylon(),
            protocol_update_triggers: triggers
                .into_iter()
                .map(|(version, enactment_condition)| {
                    ProtocolUpdateTrigger::of(version, enactment_condition)
                })
                .collect(),
            protocol_update_content_overrides: ProtocolUpdateContentOverrides::empty().into(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.genesis_protocol_version
            .validate()
            .map_err(|err| format!("{err:?}"))?;

        let mut protocol_versions = hashset!(&self.genesis_protocol_version);

        for protocol_update_trigger in self.protocol_update_triggers.iter() {
            protocol_update_trigger.validate()?;

            if !protocol_versions.insert(&protocol_update_trigger.next_protocol_version) {
                return Err(format!(
                    "Duplicate specification of protocol version {}",
                    protocol_update_trigger.next_protocol_version
                ));
            }
        }

        // Note - The protocol_update_content_overrides contents are validated in the ProtocolDefinitionResolver::new_with_raw_overrides
        // But let's check the length here too, which isn't checked there.
        for (protocol_version_name, raw_overrides) in self.protocol_update_content_overrides.iter()
        {
            let resolved = protocol_version_name
                .validate()
                .map_err(|err| format!("{err:?}"))?;

            resolved
                .definition()
                .validate_raw_overrides(raw_overrides)
                .map_err(|err| {
                    format!(
                    "Protocol version ({protocol_version_name}) has invalid raw overrides: {err:?}"
                )
                })?;
        }

        Ok(())
    }
}

// Note - at present we don't validate this on SBOR decode, but we do validate it when
// it's first used for
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct ProtocolVersionName(String);

#[derive(Clone, Debug, Eq, PartialEq, Sbor)]
pub enum ProtocolVersionNameValidationError {
    LengthInvalid {
        invalid_name: String,
        min_inclusive: usize,
        max_inclusive: usize,
        actual: usize,
    },
    CharsInvalid {
        invalid_name: String,
        allowed_chars: String,
    },
    Unrecognized {
        invalid_name: String,
    },
}

impl ProtocolVersionName {
    pub fn babylon() -> Self {
        Self::of(GENESIS_PROTOCOL_VERSION).unwrap()
    }

    pub fn anemone() -> Self {
        Self::of(ANEMONE_PROTOCOL_VERSION).unwrap()
    }

    pub fn bottlenose() -> Self {
        Self::of(BOTTLENOSE_PROTOCOL_VERSION).unwrap()
    }

    pub fn cuttlefish() -> Self {
        Self::of(CUTTLEFISH_PROTOCOL_VERSION).unwrap()
    }

    pub fn for_engine(version: ProtocolVersion) -> Self {
        match version {
            ProtocolVersion::Unbootstrapped => panic!("Unbootstrapped is not supported!"),
            ProtocolVersion::Babylon => Self::babylon(),
            ProtocolVersion::Anemone => Self::anemone(),
            ProtocolVersion::Bottlenose => Self::bottlenose(),
            ProtocolVersion::Cuttlefish => Self::cuttlefish(),
        }
    }

    pub fn of(name: impl Into<String>) -> Result<Self, ProtocolVersionNameValidationError> {
        let name = Self(name.into());
        name.validate()?;
        Ok(name)
    }

    /// Usable by persisted names. We panic if not valid in padded_len_16_version_name_for_readiness_signal.
    pub fn of_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn validate(&self) -> Result<ResolvedProtocolVersion, ProtocolVersionNameValidationError> {
        let length = self.0.len();
        if !(MIN_PROTOCOL_VERSION_NAME_LEN..=MAX_PROTOCOL_VERSION_NAME_LEN).contains(&length) {
            return Err(ProtocolVersionNameValidationError::LengthInvalid {
                invalid_name: self.0.clone(),
                min_inclusive: MIN_PROTOCOL_VERSION_NAME_LEN,
                max_inclusive: MAX_PROTOCOL_VERSION_NAME_LEN,
                actual: length,
            });
        }
        let passes_char_check = self.0.chars().all(|c| match c {
            _ if c.is_ascii_alphanumeric() => true,
            '_' | '-' => true,
            _ => false,
        });
        if !passes_char_check {
            return Err(ProtocolVersionNameValidationError::CharsInvalid {
                invalid_name: self.0.clone(),
                allowed_chars: "[A-Za-z0-9] and -".to_string(),
            });
        }

        ResolvedProtocolVersion::of(self).ok_or_else(|| {
            ProtocolVersionNameValidationError::Unrecognized {
                invalid_name: self.0.clone(),
            }
        })
    }

    pub fn as_ascii_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The caller is assumed to have validated the version name before this is called.
    pub fn padded_len_16_version_name_for_readiness_signal(&self) -> String {
        self.validate()
            .expect("Must be valid before extracting readiness signal name");
        std::iter::repeat('0')
            .take(16 - self.0.len())
            .chain(self.0.chars())
            .collect()
    }
}

impl From<ProtocolVersionName> for String {
    fn from(value: ProtocolVersionName) -> Self {
        value.0
    }
}

impl fmt::Display for ProtocolVersionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The `next_protocol_version` must be valid
#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProtocolUpdateTrigger {
    pub next_protocol_version: ProtocolVersionName,
    pub enactment_condition: ProtocolUpdateEnactmentCondition,
}

impl ProtocolUpdateTrigger {
    /// Note: panics if next_protocol_version is invalid
    pub fn of(
        next_protocol_version: ProtocolVersionName,
        enactment_condition: ProtocolUpdateEnactmentCondition,
    ) -> Self {
        Self {
            next_protocol_version,
            enactment_condition,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let protocol_version_name = &self.next_protocol_version;

        protocol_version_name
            .validate()
            .map_err(|err| format!("{err:?}"))?;

        match &self.enactment_condition {
            ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
                lower_bound_inclusive,
                upper_bound_exclusive,
                readiness_thresholds,
            } => {
                if lower_bound_inclusive >= upper_bound_exclusive {
                    return Err(format!("Protocol update {protocol_version_name} has an empty [inclusive lower bound, exclusive upper bound) range"));
                }
                if readiness_thresholds.is_empty() {
                    return Err(format!(
                        "Protocol update {protocol_version_name} does not specify at least one readiness threshold"
                    ));
                }
                for threshold in readiness_thresholds {
                    if threshold.required_ratio_of_stake_supported <= Decimal::zero()
                        || threshold.required_ratio_of_stake_supported > Decimal::one()
                    {
                        return Err(format!(
                            "Protocol update {protocol_version_name} does not have a ratio of stake supported must be between 0 (exclusive) and 1 (inclusive)"
                        ));
                    }
                }
            }
            ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(_) => {
                // Nothing to check here
            }
            ProtocolUpdateEnactmentCondition::EnactImmediatelyAfterEndOfProtocolUpdate {
                ..
            } => {
                // Nothing to check here
            }
        }
        Ok(())
    }

    pub fn readiness_signal_name(&self) -> String {
        // Readiness signal name is 32 ASCII characters long and consists of:
        // - 16 hexadecimal chars of leading bytes of `hash(enactment_condition + next_protocol_version)`
        // - next_protocol_version: 16 bytes,
        //      left padded with ASCII 0's if protocol version name is shorter than 16 characters
        let mut bytes_to_hash = scrypto_encode(&self.enactment_condition).unwrap();
        bytes_to_hash.extend_from_slice(self.next_protocol_version.as_ascii_bytes());
        let protocol_update_hash = hash(&bytes_to_hash);
        let mut res = hex::encode(protocol_update_hash)[0..16].to_string();
        res.push_str(
            &self
                .next_protocol_version
                .padded_len_16_version_name_for_readiness_signal(),
        );
        res
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub enum ProtocolUpdateEnactmentCondition {
    /// The enactment only proceeds if it's the start of epoch X,
    /// at least one readiness threshold is met, and X satisfies
    /// `lower_bound_inclusive <= X < upper_bound_exclusive`.
    EnactAtStartOfEpochIfValidatorsReady {
        /// Minimum epoch at which the protocol update can be enacted (inclusive)
        lower_bound_inclusive: Epoch,
        /// Maximum epoch at which the protocol update can be enacted (exclusive)
        upper_bound_exclusive: Epoch,
        /// A list of readiness thresholds. At least one threshold
        /// from the list must match for the protocol update to be enacted.
        readiness_thresholds: Vec<SignalledReadinessThreshold>,
    },
    /// The enactment proceeds unconditionally
    /// at the start of specified epoch.
    EnactAtStartOfEpochUnconditionally(Epoch),
    EnactImmediatelyAfterEndOfProtocolUpdate {
        trigger_after: ProtocolVersionName,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct SignalledReadinessThreshold {
    /// Required stake threshold (inclusive). Evaluated at an epoch change using validators
    /// from the _next_ epoch validator set.
    /// E.g. a value of 0.5 means: at least 50% stake required.
    pub required_ratio_of_stake_supported: Decimal,
    /// A number of required fully completed epochs on or above the threshold.
    /// Note that:
    /// - a value of 0 means:
    ///     "enact immediately at the beginning of an epoch on or above the threshold"
    /// - a value of 1 means:
    ///     "enact at the beginning of the _next_ epoch (if it still has enough support)"
    pub required_consecutive_completed_epochs_of_support: u64,
}
