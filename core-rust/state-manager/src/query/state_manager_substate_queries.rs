use radix_engine::blueprints::consensus_manager::{
    ConsensusManagerConfigurationFieldSubstate, ConsensusManagerField,
    ConsensusManagerStateFieldSubstate,
};
use radix_engine::types::*;
use radix_engine_interface::blueprints::consensus_manager::ConsensusManagerConfig;

use radix_engine_store_interface::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine_store_interface::interface::SubstateDatabase;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> Epoch;
    fn get_consensus_manager_config(&self) -> ConsensusManagerConfig;
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> Epoch {
        let consensus_manager_state = self
            .get_mapped::<SpreadPrefixKeyMapper, ConsensusManagerStateFieldSubstate>(
                CONSENSUS_MANAGER.as_node_id(),
                MAIN_BASE_PARTITION,
                &ConsensusManagerField::State.into(),
            )
            .unwrap()
            .into_payload()
            .into_latest();
        consensus_manager_state.epoch
    }

    fn get_consensus_manager_config(&self) -> ConsensusManagerConfig {
        self.get_mapped::<SpreadPrefixKeyMapper, ConsensusManagerConfigurationFieldSubstate>(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::Configuration.into(),
        )
        .unwrap()
        .into_payload()
        .into_latest()
        .config
    }
}
