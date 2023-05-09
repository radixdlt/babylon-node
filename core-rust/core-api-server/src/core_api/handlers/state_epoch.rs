use crate::core_api::*;
use radix_engine::blueprints::epoch_manager::{CurrentValidatorSetSubstate, EpochManagerSubstate};
use radix_engine::types::{EpochManagerOffset, EPOCH_MANAGER};
use radix_engine_interface::types::OBJECT_BASE_MODULE;
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_state_epoch(
    state: State<CoreApiState>,
    Json(request): Json<models::StateEpochRequest>,
) -> Result<Json<models::StateEpochResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let database = state.database.read();
    let epoch_manager_substate: EpochManagerSubstate = read_mandatory_substate(
        database.deref(),
        EPOCH_MANAGER.as_node_id(),
        OBJECT_BASE_MODULE,
        &EpochManagerOffset::EpochManager.into(),
    )?;

    let current_validator_set_substate: CurrentValidatorSetSubstate = read_mandatory_substate(
        database.deref(),
        EPOCH_MANAGER.as_node_id(),
        OBJECT_BASE_MODULE,
        &EpochManagerOffset::CurrentValidatorSet.into(),
    )?;

    Ok(models::StateEpochResponse {
        epoch: to_api_epoch(&mapping_context, epoch_manager_substate.epoch)?,
        epoch_manager: Some(to_api_epoch_manager_substate(
            &mapping_context,
            &epoch_manager_substate,
        )?),
        current_validator_set: Some(to_api_current_validator_set_substate(
            &mapping_context,
            &current_validator_set_substate,
        )?),
    })
    .map(Json)
}
