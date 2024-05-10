use crate::engine_prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn dumunet_protocol_config() -> ProtocolConfig {
    ProtocolConfig::new_with_triggers(hashmap! {
        ANEMONE_PROTOCOL_VERSION => EnactAtStartOfEpochUnconditionally(Epoch::of(3)),
        BOTTLENOSE_PROTOCOL_VERSION => EnactAtStartOfEpochUnconditionally(Epoch::of(4))
    })
}
