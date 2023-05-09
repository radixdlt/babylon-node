use crate::core_api::*;
use radix_engine::blueprints::clock::ClockSubstate;
use radix_engine::types::{ClockOffset, CLOCK};
use radix_engine_interface::types::OBJECT_BASE_MODULE;
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_state_clock(
    state: State<CoreApiState>,
    Json(request): Json<models::StateClockRequest>,
) -> Result<Json<models::StateClockResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let database = state.database.read();
    let clock_substate: ClockSubstate = read_mandatory_substate(
        database.deref(),
        CLOCK.as_node_id(),
        OBJECT_BASE_MODULE,
        &ClockOffset::CurrentTimeRoundedToMinutes.into(),
    )?;

    // TODO: Substate offset (CurrentTimeRoundedToMinutes) doesn't match substate name (ClockSubstate); fix upstream
    Ok(models::StateClockResponse {
        current_minute: Some(to_api_clock_substate(&clock_substate)?),
    })
    .map(Json)
}
