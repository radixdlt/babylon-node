use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{EpochManagerOffset, SubstateOffset, EPOCH_MANAGER};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId};
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
    let mapping_context = MappingContext::new(&state_manager.network);

    let epoch_manager_substate = {
        let substate_offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(EPOCH_MANAGER.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::EpochManager(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let validator_set_substate = {
        let substate_offset = SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet);
        let loaded_substate = read_known_substate(
            state_manager,
            RENodeId::GlobalObject(EPOCH_MANAGER.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::ValidatorSet(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::StateEpochResponse {
        epoch: to_api_epoch(&mapping_context, epoch_manager_substate.epoch)?,
        epoch_manager: Some(to_api_epoch_manager_substate(
            &mapping_context,
            &epoch_manager_substate,
        )?),
        active_validator_set: Some(to_api_validator_set_substate(
            &mapping_context,
            &validator_set_substate,
        )?),
    })
}
