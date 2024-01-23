use radix_engine::prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn dumunet_protocol_config() -> ProtocolConfig {
    ProtocolConfig::new_with_triggers(hashmap! {
        ANEMONE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            lower_bound_inclusive: Epoch::of(1),
            upper_bound_exclusive: Epoch::of(1000000),
            readiness_thresholds: vec![SignalledReadinessThreshold {
                required_ratio_of_stake_supported: dec!("0.80"),
                required_consecutive_completed_epochs_of_support: 10,
            }],
        }
    })
}
