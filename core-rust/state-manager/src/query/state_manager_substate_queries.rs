use radix_engine::types::SubstateId;

use crate::store::traits::*;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> u64;
}

impl<T: ReadableSubstateStore + QueryableSubstateStore> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> u64 {
        self.get_substate(&SubstateId::System)
            .unwrap()
            .substate
            .system()
            .epoch
    }
}
