use radix_engine::model::{GlobalAddressSubstate, SystemSubstate};
use radix_engine::types::SubstateId;
use scrypto::constants::SYS_SYSTEM_COMPONENT;
use scrypto::engine::types::{GlobalAddress, GlobalOffset, RENodeId, SubstateOffset, SystemOffset};

use crate::store::traits::*;

pub trait StateManagerSubstateQueries {
    fn get_epoch(&self) -> u64;
}

impl<T: ReadableSubstateStore + QueryableSubstateStore> StateManagerSubstateQueries for T {
    fn get_epoch(&self) -> u64 {
        let global_address_substate: GlobalAddressSubstate = self
            .get_substate(&SubstateId(
                RENodeId::Global(GlobalAddress::Component(SYS_SYSTEM_COMPONENT)),
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .unwrap()
            .substate
            .to_runtime()
            .into();

        if let GlobalAddressSubstate::SystemComponent(scrypto::component::Component(component_id)) =
            global_address_substate
        {
            let system_substate: SystemSubstate = self
                .get_substate(&SubstateId(
                    RENodeId::System(component_id),
                    SubstateOffset::System(SystemOffset::System),
                ))
                .unwrap()
                .substate
                .to_runtime()
                .into();

            return system_substate.epoch;
        }

        panic!("Failed to read SystemSubstate");
    }
}
