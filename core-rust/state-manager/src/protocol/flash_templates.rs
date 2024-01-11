use radix_engine::blueprints::consensus_manager::{
    ConsensusManagerConfigSubstate, ConsensusManagerConfigurationFieldPayload,
    ConsensusManagerField, VersionedConsensusManagerConfiguration,
};
use radix_engine::system::system_substates::{FieldSubstate, FieldSubstateV1, LockStatus};
use radix_engine::track::StateUpdates;
use radix_engine_common::prelude::{scrypto_encode, CONSENSUS_MANAGER};
use radix_engine_interface::blueprints::consensus_manager::ConsensusManagerConfig;
use radix_engine_interface::prelude::MAIN_BASE_PARTITION;
use radix_engine_store_interface::interface::DatabaseUpdate;

pub fn consensus_manager_config_flash(new_config: ConsensusManagerConfig) -> StateUpdates {
    let mut state_updates = StateUpdates::default();
    state_updates
        .of_node(CONSENSUS_MANAGER.into_node_id())
        .of_partition(MAIN_BASE_PARTITION)
        .update_substates(vec![(
            ConsensusManagerField::Configuration.into(),
            DatabaseUpdate::Set(
                scrypto_encode(&FieldSubstate::V1(FieldSubstateV1 {
                    payload: ConsensusManagerConfigurationFieldPayload {
                        content: VersionedConsensusManagerConfiguration::V1(
                            ConsensusManagerConfigSubstate { config: new_config },
                        ),
                    },
                    lock_status: LockStatus::Locked,
                }))
                .unwrap(),
            ),
        )]);
    state_updates
}
