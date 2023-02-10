use crate::core_api::*;
use radix_engine::system::substates::PersistedSubstate;
use radix_engine::types::{EpochManagerOffset, GlobalAddress, SubstateOffset, EPOCH_MANAGER};
use radix_engine_interface::api::types::NodeModuleId;
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_state_epoch(
    state: Extension<CoreApiState>,
    request: Json<models::StateEpochRequest>,
) -> Result<Json<models::StateEpochResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_state_epoch_internal)
}

fn handle_state_epoch_internal(
    state_manager: &ActualStateManager,
    request: models::StateEpochRequest,
) -> Result<models::StateEpochResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let node_id =
        read_derefed_global_node_id(state_manager, GlobalAddress::Component(EPOCH_MANAGER))?;

    let epoch_manager_substate = {
        let substate_offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let loaded_substate =
            read_known_substate(state_manager, node_id, NodeModuleId::SELF, &substate_offset)?;
        let PersistedSubstate::EpochManager(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::StateEpochResponse {
        epoch: to_api_epoch(epoch_manager_substate.epoch)?,
    })
}
