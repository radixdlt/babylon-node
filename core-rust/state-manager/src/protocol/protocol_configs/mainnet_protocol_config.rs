use crate::engine_prelude::*;

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
            // - Assuming epoch length will be 5 mins
            // =================================================================
            lower_bound_inclusive: Epoch::of(70019), // estimated: 2024-02-05T18:00:57.229Z
            upper_bound_exclusive: Epoch::of(74051), // estimated: 2024-02-19T18:00:57.229Z
            readiness_thresholds: vec![
                SignalledReadinessThreshold {
                    required_ratio_of_stake_supported: dec!(0.75),
                    required_consecutive_completed_epochs_of_support: 1152, // estimated: 4 days
                },
            ],
        },
        BOTTLENOSE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            // =================================================================
            // PROTOCOL_VERSION: "bottlenose"
            // READINESS_SIGNAL: "86894b9104afb73a000000bottlenose"
            // =================================================================
            // The below estimates are based off:
            // - Calculating relative to epoch 97091
            // - Using that epoch 97091 started at 2024-05-09T18:01:00.000Z
            // - Assuming epoch length will be 5 mins
            // =================================================================
            lower_bound_inclusive: Epoch::of(104291), // estimated: 2024-06-03T18:01:00.000Z
            upper_bound_exclusive: Epoch::of(112355), // estimated: 2024-07-01T18:01:00.000Z
            readiness_thresholds: vec![
                SignalledReadinessThreshold {
                    required_ratio_of_stake_supported: dec!(0.75),
                    required_consecutive_completed_epochs_of_support: 4032, // estimated: 2 weeks
                },
            ],
        },
    })
}
