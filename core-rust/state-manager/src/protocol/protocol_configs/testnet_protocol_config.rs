use crate::engine_prelude::*;

use crate::protocol::*;
use ProtocolUpdateEnactmentCondition::*;

pub fn testnet_protocol_config() -> ProtocolConfig {
    // We wish most testnets to simply run the latest update, so we run
    // the updates back-to-back after genesis.
    ProtocolConfig::new_with_triggers(hashmap! {
        ProtocolVersionName::anemone() => EnactImmediatelyAfterEndOfProtocolUpdate {
            trigger_after: ProtocolVersionName::babylon(),
        },
        ProtocolVersionName::bottlenose() => EnactImmediatelyAfterEndOfProtocolUpdate {
            trigger_after: ProtocolVersionName::anemone(),
        },
        ProtocolVersionName::cuttlefish() => EnactImmediatelyAfterEndOfProtocolUpdate {
            trigger_after: ProtocolVersionName::bottlenose(),
        },
    })
}
