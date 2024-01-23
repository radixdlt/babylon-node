use radix_engine::prelude::ScryptoSbor;
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::{hash, scrypto_encode};

use radix_engine_common::types::Epoch;

use crate::protocol::*;
use utils::btreeset;

// This file contains types for node's local static protocol configuration

const MAX_PROTOCOL_VERSION_NAME_LEN: usize = 16;

/// The `ProtocolConfig` is a static configuration provided per-network, or overriden for testing.
///
/// When a node commits (or creates a proof), it checks `protocol_update_triggers` to see if a protocol update
/// should be triggered next.
///
/// If the update is triggered, any relevant overrides are combined with the `protocol_definition_resolver` to
/// work out what it should do for the update.
#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProtocolConfig {
    pub genesis_protocol_version: String,
    pub protocol_update_triggers: Vec<ProtocolUpdateTrigger>,
    /// This allows overriding the configuration of a protocol update.
    ///
    /// You can create this with `ProtocolUpdateContentOverrides::empty().into()` or `Default::default()`.
    ///
    /// This wraps an optional `Map<String, Vec<u8>>` where the `Vec<u8>` is the encoded config
    /// for the protocol updater matching the given protocol update.
    ///
    /// All nodes must agree on the content overrides used. The content overrides form a part of
    /// the definition of the protocol update, and if nodes use different overrides, they will execute
    /// different updates and need manual recovery.
    pub protocol_update_content_overrides: RawProtocolUpdateContentOverrides,
}

impl ProtocolConfig {
    pub fn new_with_no_updates() -> ProtocolConfig {
        Self {
            genesis_protocol_version: GENESIS_PROTOCOL_VERSION.to_string(),
            protocol_update_triggers: vec![],
            protocol_update_content_overrides: RawProtocolUpdateContentOverrides::default(),
        }
    }

    pub fn new_with_triggers<'a>(
        triggers: impl IntoIterator<Item = (&'a str, ProtocolUpdateEnactmentCondition)>,
    ) -> ProtocolConfig {
        Self {
            genesis_protocol_version: GENESIS_PROTOCOL_VERSION.to_string(),
            protocol_update_triggers: triggers
                .into_iter()
                .map(|(version, enactment_condition)| {
                    ProtocolUpdateTrigger::of(version, enactment_condition)
                })
                .collect(),
            protocol_update_content_overrides: RawProtocolUpdateContentOverrides::default(),
        }
    }

    pub fn validate(
        &self,
        protocol_definition_resolver: &ProtocolDefinitionResolver,
    ) -> Result<(), String> {
        let mut protocol_versions = btreeset!();

        Self::validate_protocol_version_name(
            self.genesis_protocol_version.as_str(),
            protocol_definition_resolver,
        )?;

        for protocol_update in self.protocol_update_triggers.iter() {
            let protocol_version_name = protocol_update.next_protocol_version.as_str();
            Self::validate_protocol_version_name(
                protocol_version_name,
                protocol_definition_resolver,
            )?;

            if !protocol_versions.insert(protocol_version_name) {
                return Err(format!(
                    "Duplicate specification of protocol version {protocol_version_name}"
                ));
            }

            match &protocol_update.enactment_condition {
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
            }
        }

        // Note - The protocol_update_content_overrides are validated in the ProtocolDefinitionResolver::new_with_raw_overrides

        Ok(())
    }

    fn validate_protocol_version_name(
        name: &str,
        protocol_definition_resolver: &ProtocolDefinitionResolver,
    ) -> Result<(), String> {
        if name.len() > MAX_PROTOCOL_VERSION_NAME_LEN {
            return Err(format!(
                "Protocol version ({name}) is longer than {MAX_PROTOCOL_VERSION_NAME_LEN} chars"
            ));
        }

        if !name.is_ascii() {
            return Err(format!(
                "Protocol version ({name}) can't use non-ascii characters"
            ));
        }

        if !protocol_definition_resolver.recognizes(name) {
            return Err(format!(
                "Protocol version ({name}) does not have a recognized definition"
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProtocolUpdateTrigger {
    pub next_protocol_version: String,
    pub enactment_condition: ProtocolUpdateEnactmentCondition,
}

impl ProtocolUpdateTrigger {
    pub fn of(
        next_protocol_version: impl Into<String>,
        enactment_condition: ProtocolUpdateEnactmentCondition,
    ) -> Self {
        Self {
            next_protocol_version: next_protocol_version.into(),
            enactment_condition,
        }
    }

    pub fn readiness_signal_name(&self) -> String {
        // Readiness signal name is 32 ASCII characters long and consists of:
        // - 16 hexadecimal chars of leading bytes of `hash(enactment_condition + next_protocol_version)`
        // - next_protocol_version: 16 bytes,
        //      left padded with ASCII 0's if protocol version name is shorter than 16 characters
        let mut bytes_to_hash = scrypto_encode(&self.enactment_condition).unwrap();
        bytes_to_hash.extend_from_slice(self.next_protocol_version.as_bytes());
        let protocol_update_hash = hash(&bytes_to_hash);
        let mut res = hex::encode(protocol_update_hash)[0..16].to_string();
        res.push_str(&ascii_protocol_version_name_len_16(
            &self.next_protocol_version,
        ));
        res
    }
}

/// Returns an ASCII protocol version name truncated/left padded to canonical length (16 bytes)
pub fn ascii_protocol_version_name_len_16(protocol_version_name: &str) -> String {
    let filtered_version_name: String = protocol_version_name
        .chars()
        .filter_map(|c| match c {
            _ if c.is_ascii_alphanumeric() => Some(c),
            '_' | '-' => Some(c),
            ' ' => Some('_'),
            _ => None,
        })
        .take(16)
        .collect();

    std::iter::repeat('0')
        .take(16 - filtered_version_name.len())
        .chain(filtered_version_name.chars())
        .collect()
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
