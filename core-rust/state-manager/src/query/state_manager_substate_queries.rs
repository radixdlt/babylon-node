use radix_engine::model::EpochManagerSubstate;
use radix_engine::types::{EpochManagerOffset, SubstateId, EPOCH_MANAGER};
use radix_engine::types::{GlobalAddress, GlobalOffset, RENodeId, SubstateOffset};

use crate::store::traits::*;

pub trait StateManagerSubstateQueries {
    fn global_deref(&self, global_address: GlobalAddress) -> Option<RENodeId>;
    fn get_epoch(&self) -> u64;
}

impl<T: ReadableSubstateStore> StateManagerSubstateQueries for T {
    fn global_deref(&self, global_address: GlobalAddress) -> Option<RENodeId> {
        let node_id = self
            .get_substate(&SubstateId(
                RENodeId::Global(global_address),
                SubstateOffset::Global(GlobalOffset::Global),
            ))?
            .substate
            .to_runtime()
            .global()
            .node_deref();

        Some(node_id)
    }

    fn get_epoch(&self) -> u64 {
        let epoch_manager_node = self
            .global_deref(GlobalAddress::System(EPOCH_MANAGER))
            .expect("Couldn't find Epoch Manager from Global Address!");

        let system_substate: EpochManagerSubstate = self
            .get_substate(&SubstateId(
                epoch_manager_node,
                SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            ))
            .expect("Couldn't find Epoch Manager substate!")
            .substate
            .to_runtime()
            .into();

        system_substate.epoch
    }
}
