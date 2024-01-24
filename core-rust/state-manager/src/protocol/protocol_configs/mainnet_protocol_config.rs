use radix_engine::prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn mainnet_protocol_config() -> ProtocolConfig {
    // See config_printer.rs
    ProtocolConfig::new_with_triggers(hashmap! {
        ANEMONE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            // =================================================================
            // PROTOCOL_VERSION: "anemone"
            // READINESS_SIGNAL: "220e2a4a4e86e3e6000000000anemone"
            // =================================================================
            // The below estimates are based off:
            // - Calculating relative to epoch 66516
            // - Using that epoch 66516 started at 2024-01-24T14:05:57.229Z
            // - Assuming epoch length will be 5 mins, 0 secs
            // =================================================================
            lower_bound_inclusive: Epoch::of(70019), // estimated: 2024-02-05T18:00:57.229Z
            upper_bound_exclusive: Epoch::of(74051), // estimated: 2024-02-19T18:00:57.229Z
            readiness_thresholds: vec![
                SignalledReadinessThreshold {
                    required_ratio_of_stake_supported: dec!(0.75),
                    required_consecutive_completed_epochs_of_support: 1152, // estimated: 4 days, 0 hours, 0 mins, 0 secs
                },
            ],
        },
    })
}
