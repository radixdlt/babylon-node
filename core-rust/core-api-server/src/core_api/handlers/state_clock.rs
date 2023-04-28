use crate::core_api::*;
use radix_engine::blueprints::clock::ClockSubstate;
use radix_engine::types::{ClockOffset, CLOCK};
use radix_engine_interface::types::SysModuleId;
use std::ops::Deref;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_state_clock(
    state: State<CoreApiState>,
    Json(request): Json<models::StateClockRequest>,
) -> Result<Json<models::StateClockResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let database = state.database.read();
    let clock_substate: ClockSubstate = read_mandatory_substate(
        database.deref(),
        CLOCK.as_node_id(),
        SysModuleId::Object.into(),
        &ClockOffset::CurrentTimeRoundedToMinutes.into(),
    )?;

    // TODO: rename current minute (rename upstream substate?)
    Ok(models::StateClockResponse {
        current_minute: Some(to_api_clock_substate(&clock_substate)?),
    })
    .map(Json)
}
