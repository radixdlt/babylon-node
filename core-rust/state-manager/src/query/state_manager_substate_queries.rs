use radix_engine::blueprints::epoch_manager::EpochManagerSubstate;
use radix_engine::types::{EpochManagerOffset, SubstateId, EPOCH_MANAGER};
use radix_engine::types::{RENodeId, SubstateOffset};
use radix_engine_interface::api::types::NodeModuleId;

use crate::store::traits::*;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> u64;
}

impl<T: ReadableSubstateStore> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> u64 {
        let system_substate: EpochManagerSubstate = self
            .get_substate(&SubstateId(
                RENodeId::GlobalComponent(EPOCH_MANAGER),
                NodeModuleId::SELF,
                SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            ))
            .expect("Couldn't find Epoch Manager substate!")
            .substate
            .to_runtime()
            .into();

        system_substate.epoch
    }
}
