use radix_engine::blueprints::epoch_manager::EpochManagerSubstate;
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::types::{EpochManagerOffset, SysModuleId};
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtMapper;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> u64;
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> u64 {
        let epoch_manager_substate: EpochManagerSubstate = self
            .get_mapped_substate::<JmtMapper, EpochManagerSubstate>(
                EPOCH_MANAGER.as_node_id(),
                SysModuleId::Object.into(),
                &EpochManagerOffset::EpochManager.into(),
            )
            .unwrap();
        epoch_manager_substate.epoch
    }
}
