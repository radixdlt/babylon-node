use crate::engine_prelude::*;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> Epoch;
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
}
