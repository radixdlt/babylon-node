use crate::engine_prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn testnet_protocol_config() -> ProtocolConfig {
    // We wish most testnets to upgrade to the latest protocol version as soon as they can...
    // Currently we can only enact protocol updates after the end of a "normal" epoch.
    //
    // On testnets:
    // * Epoch 1 is genesis
    // * Epoch 2 is the first normal epoch
    //
    // So we should target applying protocol updates from 3 onwards (1 per epoch)
    ProtocolConfig::new_with_triggers(hashmap! {
        ANEMONE_PROTOCOL_VERSION => EnactAtStartOfEpochUnconditionally(Epoch::of(3)),
        BOTTLENOSE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            // =================================================================
            // PROTOCOL_VERSION: "bottlenose"
            // READINESS_SIGNAL: "7fbcb0758cc14849000000bottlenose"
            // =================================================================
            lower_bound_inclusive: Epoch::of(4),
            upper_bound_exclusive: Epoch::of(1000000),
            readiness_thresholds: vec![SignalledReadinessThreshold {
                required_ratio_of_stake_supported: dec!("0.80"),
                required_consecutive_completed_epochs_of_support: 2,
            }],
        }
    })
}
