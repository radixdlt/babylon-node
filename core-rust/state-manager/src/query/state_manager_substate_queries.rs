use radix_engine::blueprints::consensus_manager::{
    ConsensusManagerField, ConsensusManagerStateFieldSubstate,
};
use radix_engine::types::*;

use radix_engine_store_interface::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine_store_interface::interface::SubstateDatabase;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> Epoch;
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> Epoch {
        self.get_mapped::<SpreadPrefixKeyMapper, ConsensusManagerStateFieldSubstate>(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::State.into(),
        )
        .map_or(Epoch::zero(), |substate| {
            let consensus_manager_state = substate.into_payload().into_latest();
            consensus_manager_state.epoch
        })
    }
}
