use radix_engine::blueprints::epoch_manager::EpochManagerSubstate;
use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::types::{EpochManagerOffset, OBJECT_BASE_MODULE};

use radix_engine_store_interface::interface::SubstateDatabase;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> u64;
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> u64 {
        let epoch_manager_substate: EpochManagerSubstate = self
            .get_mapped::<SpreadPrefixKeyMapper, EpochManagerSubstate>(
                EPOCH_MANAGER.as_node_id(),
                OBJECT_BASE_MODULE,
                &EpochManagerOffset::EpochManager.into(),
            )
            .unwrap();
        epoch_manager_substate.epoch
    }
}
