use radix_engine::prelude::{ScryptoCategorize, ScryptoDecode, ScryptoEncode};
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::{hash, scrypto_encode};

use radix_engine_common::types::Epoch;

use crate::{ProtocolUpdaterFactory, StateVersion};
use utils::btreeset;

// This file contains types for node's local static protocol configuration

pub const GENESIS_PROTOCOL_VERSION: &str = "babylon-genesis";

const MAX_PROTOCOL_VERSION_NAME_LEN: usize = 16;

/// Returns a protocol version name left padded to canonical length (16 bytes)
pub fn padded_protocol_version_name(unpadded_protocol_version_name: &str) -> String {
    let mut res = "".to_owned();
    for _ in 0..16 - unpadded_protocol_version_name.len() {
        res.push('0');
    }
    res.push_str(unpadded_protocol_version_name);
    res
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ProtocolUpdate {
    pub next_protocol_version: String,
    pub enactment_condition: ProtocolUpdateEnactmentCondition,
}

impl ProtocolUpdate {
    pub fn readiness_signal_name(&self) -> String {
        // Readiness signal name is 32 bytes long and consists of:
        // - 16 leading bytes of `hash(enactment_condition + next_protocol_version)`
        // - protocol version name: 16 bytes,
        //      left padded with ASCII 0's if protocol version name is shorter than 16 characters
        let mut bytes_to_hash = scrypto_encode(&self.enactment_condition).unwrap();
        bytes_to_hash.extend_from_slice(self.next_protocol_version.as_bytes());
        let protocol_update_hash = hash(&bytes_to_hash);
        let mut res = hex::encode(protocol_update_hash)[0..16].to_string();
        res.push_str(&padded_protocol_version_name(&self.next_protocol_version));
        res
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ProtocolConfig {
    pub genesis_protocol_version: String,
    pub protocol_updates: Vec<ProtocolUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ProtocolUpdateEnactmentCondition {
    EnactWhenSupportedAndWithinBounds {
        lower_bound: ProtocolUpdateEnactmentBound,
        upper_bound: ProtocolUpdateEnactmentBound,
        support_type: ProtocolUpdateSupportType,
    },
    EnactUnconditionallyAtEpoch(Epoch),
    EnactUnconditionallyAtStateVersion(StateVersion),
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ProtocolUpdateEnactmentBound {
    Epoch(Epoch),
    StateVersion(StateVersion),
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ProtocolUpdateSupportType {
    SignalledReadiness(Vec<SignalledReadinessThreshold>),
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl ProtocolConfig {
    pub fn mainnet() -> ProtocolConfig {
        Self {
            genesis_protocol_version: GENESIS_PROTOCOL_VERSION.to_string(),
            protocol_updates: vec![],
        }
    }

    pub fn testing_default() -> ProtocolConfig {
        Self {
            genesis_protocol_version: GENESIS_PROTOCOL_VERSION.to_string(),
            protocol_updates: vec![],
        }
    }

    pub fn sanity_check(
        &self,
        protocol_updater_factory: &(dyn ProtocolUpdaterFactory + Send + Sync),
    ) -> Result<(), String> {
        let mut protocol_versions = btreeset!();

        if self.genesis_protocol_version.len() > MAX_PROTOCOL_VERSION_NAME_LEN {
            return Err("Genesis protocol version name is too long".to_string());
        }

        if !self.genesis_protocol_version.is_ascii() {
            return Err("Genesis protocol version name can't use non-ascii characters".to_string());
        }

        if !protocol_updater_factory
            .supports_protocol_version(self.genesis_protocol_version.as_str())
        {
            return Err(
                "Protocol updater factory does not support genesis protocol version".to_string(),
            );
        }

        for protocol_update in self.protocol_updates.iter() {
            if protocol_update.next_protocol_version.len() > MAX_PROTOCOL_VERSION_NAME_LEN {
                return Err("Protocol version name is too long".to_string());
            }

            if !protocol_update.next_protocol_version.is_ascii() {
                return Err("Protocol version name can't use non-ascii characters".to_string());
            }

            if !protocol_updater_factory
                .supports_protocol_version(protocol_update.next_protocol_version.as_str())
            {
                return Err("Protocol updater factory does not support a configured update protocol version".to_string());
            }

            protocol_versions.insert(&protocol_update.next_protocol_version);

            match &protocol_update.enactment_condition {
                ProtocolUpdateEnactmentCondition::EnactWhenSupportedAndWithinBounds {
                    lower_bound,
                    upper_bound,
                    support_type,
                } => {
                    match (lower_bound, upper_bound) {
                        (
                            ProtocolUpdateEnactmentBound::Epoch(lower_epoch),
                            ProtocolUpdateEnactmentBound::Epoch(upper_epoch),
                        ) => {
                            if lower_epoch >= upper_epoch {
                                return Err(
                                    "Upper bound must be greater than lower bound".to_string()
                                );
                            }
                        }
                        (
                            ProtocolUpdateEnactmentBound::StateVersion(lower_state_version),
                            ProtocolUpdateEnactmentBound::StateVersion(upper_state_version),
                        ) => {
                            if lower_state_version >= upper_state_version {
                                return Err(
                                    "Upper bound must be greater than lower bound".to_string()
                                );
                            }
                        }

                        _ => {
                            // No-op
                        }
                    }
                    match support_type {
                        ProtocolUpdateSupportType::SignalledReadiness(thresholds) => {
                            if thresholds.is_empty() {
                                return Err("SignalledReadiness protocol update must specify at least one threshold".to_string());
                            }
                            for threshold in thresholds {
                                if threshold.required_ratio_of_stake_supported <= Decimal::zero()
                                    || threshold.required_ratio_of_stake_supported > Decimal::one()
                                {
                                    return Err("Required ratio of stake supported must be between 0 (exclusive) and 1 (inclusive)".to_string());
                                }
                            }
                        }
                    }
                }
                ProtocolUpdateEnactmentCondition::EnactUnconditionallyAtEpoch(_)
                | ProtocolUpdateEnactmentCondition::EnactUnconditionallyAtStateVersion(_) => {
                    // Nothing to check here
                }
            }
        }

        if protocol_versions.len() != self.protocol_updates.len() {
            return Err("Protocol versions must have unique names".to_string());
        }

        Ok(())
    }
}
