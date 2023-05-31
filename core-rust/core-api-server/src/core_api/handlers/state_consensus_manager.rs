use crate::core_api::*;
use radix_engine::types::*;
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_state_consensus_manager(
    state: State<CoreApiState>,
    Json(request): Json<models::StateConsensusManagerRequest>,
) -> Result<Json<models::StateConsensusManagerResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let database = state.database.read();

    let config_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::Config.into(),
    )?;
    let state_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::ConsensusManager.into(),
    )?;
    let current_proposal_statistic_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::CurrentProposalStatistic.into(),
    )?;
    let current_validator_set_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::CurrentValidatorSet.into(),
    )?;
    let current_time_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::CurrentTime.into(),
    )?;
    let current_time_round_to_minutes_substate = read_mandatory_main_field_substate(
        database.deref(),
        CONSENSUS_MANAGER.as_node_id(),
        &ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
    )?;

    Ok(models::StateConsensusManagerResponse {
        config: Some(to_api_consensus_manager_config_substate(&config_substate)?),
        state: Some(to_api_consensus_manager_state_substate(
            &mapping_context,
            &state_substate,
        )?),
        current_proposal_statistic: Some(to_api_current_proposal_statistic_substate(
            &mapping_context,
            &current_proposal_statistic_substate,
        )?),
        current_validator_set: Some(to_api_current_validator_set_substate(
            &mapping_context,
            &current_validator_set_substate,
        )?),
        current_time: Some(to_api_current_time_substate(&current_time_substate)?),
        current_time_rounded_to_minutes: Some(to_api_current_time_rounded_to_minutes_substate(
            &current_time_round_to_minutes_substate,
        )?),
    })
    .map(Json)
}
