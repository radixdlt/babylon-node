use crate::engine_prelude::*;

pub trait StateManagerSubstateQueries {
    fn get_epoch_and_round(&self) -> (Epoch, Round);
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch_and_round(&self) -> (Epoch, Round) {
        let consensus_manager_state = self
            .get_substate::<ConsensusManagerStateFieldSubstate>(
                CONSENSUS_MANAGER,
                MAIN_BASE_PARTITION,
                ConsensusManagerField::State,
            )
            .unwrap()
            .into_payload()
            .fully_update_and_into_latest_version();
        (consensus_manager_state.epoch, consensus_manager_state.round)
    }
}
