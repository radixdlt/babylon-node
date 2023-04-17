use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{EpochManagerOffset, SubstateOffset, EPOCH_MANAGER};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId};
use std::ops::Deref;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_state_epoch(
    state: State<CoreApiState>,
    Json(request): Json<models::StateEpochRequest>,
) -> Result<Json<models::StateEpochResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let database = state.database.read();
    let epoch_manager_substate = {
        let substate_offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let loaded_substate = read_mandatory_substate(
            database.deref(),
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
        let loaded_substate = read_mandatory_substate(
            database.deref(),
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
    .map(Json)
}
