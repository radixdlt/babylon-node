use radix_engine::blueprints::epoch_manager::*;
use radix_engine::track::db_key_mapper::*;
use radix_engine::types::*;

use radix_engine_store_interface::interface::SubstateDatabase;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> u64;
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> u64 {
        let epoch_manager_substate: EpochManagerSubstate = self
            .get_mapped::<SpreadPrefixKeyMapper, EpochManagerSubstate>(
                EPOCH_MANAGER.as_node_id(),
                OBJECT_BASE_PARTITION,
                &EpochManagerField::EpochManager.into(),
            )
            .unwrap();
        epoch_manager_substate.epoch
    }
}
