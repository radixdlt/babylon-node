use crate::engine_prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn stokenet_protocol_config() -> ProtocolConfig {
    ProtocolConfig::new_with_triggers(hashmap! {
        ANEMONE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            // =================================================================
            // PROTOCOL_VERSION: "anemone"
            // READINESS_SIGNAL: "811c31d2bc6a2631000000000anemone"
            // =================================================================
            lower_bound_inclusive: Epoch::of(1),
            upper_bound_exclusive: Epoch::of(1000000),
            readiness_thresholds: vec![SignalledReadinessThreshold {
                required_ratio_of_stake_supported: dec!("0.80"),
                required_consecutive_completed_epochs_of_support: 10,
            }],
        },
        BOTTLENOSE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            // =================================================================
            // PROTOCOL_VERSION: "bottlenose"
            // READINESS_SIGNAL: "35701a6147bfd870000000bottlenose"
            // =================================================================
            lower_bound_inclusive: Epoch::of(1),
            upper_bound_exclusive: Epoch::of(1000000),
            readiness_thresholds: vec![SignalledReadinessThreshold {
                required_ratio_of_stake_supported: dec!("0.80"),
                required_consecutive_completed_epochs_of_support: 10,
            }],
        }
    })
}
