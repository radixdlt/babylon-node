use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{EpochManagerOffset, GlobalAddress, SubstateOffset, EPOCH_MANAGER};
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_v0_state_epoch(
    state: Extension<CoreApiState>,
) -> Result<Json<models::V0StateEpochResponse>, RequestHandlingError> {
    core_api_read_handler(state, Json(()), handle_v0_state_epoch_internal)
}

fn handle_v0_state_epoch_internal(
    state_manager: &ActualStateManager,
    _request: (),
) -> Result<models::V0StateEpochResponse, RequestHandlingError> {
    let node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Component(EPOCH_MANAGER))?;

    let epoch_manager_substate = {
        let substate_offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let loaded_substate = read_known_substate(state_manager, node_id, &substate_offset)?;
        let PersistedSubstate::EpochManager(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::V0StateEpochResponse {
        epoch: to_api_epoch(epoch_manager_substate.epoch)?,
    })
}
