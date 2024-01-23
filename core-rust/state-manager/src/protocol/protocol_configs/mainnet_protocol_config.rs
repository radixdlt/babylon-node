use radix_engine::prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn mainnet_protocol_config() -> ProtocolConfig {
    ProtocolConfig::new_with_triggers(hashmap! {
        ANEMONE_PROTOCOL_VERSION => EnactAtStartOfEpochIfValidatorsReady {
            // TODO(anemone): update the epoch bounds and thresholds
            lower_bound_inclusive: Epoch::of(10000),
            upper_bound_exclusive: Epoch::of(20000),
            readiness_thresholds: vec![
                SignalledReadinessThreshold {
                    required_ratio_of_stake_supported: dec!("0.80"),
                    required_consecutive_completed_epochs_of_support: 10,
                }
            ],
        }
    })
}
