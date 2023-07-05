use radix_engine::types::*;

use radix_engine_queries::typed_substate_layout::ConsensusManagerSubstate;
use radix_engine_store_interface::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine_store_interface::interface::SubstateDatabase;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> Epoch;
}

impl<T: SubstateDatabase> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> Epoch {
        let substate = self
            .get_mapped::<SpreadPrefixKeyMapper, ConsensusManagerSubstate>(
                CONSENSUS_MANAGER.as_node_id(),
                MAIN_BASE_PARTITION,
                &ConsensusManagerField::ConsensusManager.into(),
            )
            .unwrap();
        substate.epoch
    }
}
