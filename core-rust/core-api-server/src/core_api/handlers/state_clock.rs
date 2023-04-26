use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{ClockOffset, SubstateOffset, CLOCK};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId};
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_state_clock(
    state: State<CoreApiState>,
    Json(request): Json<models::StateClockRequest>,
) -> Result<Json<models::StateClockResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let database = state.database.read();
    let rounded_to_minutes_substate = {
        let substate_offset = SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes);
        let loaded_substate = read_mandatory_substate(
            database.deref(),
            RENodeId::GlobalObject(CLOCK.into()),
            NodeModuleId::SELF,
            &substate_offset,
        )?;
        let PersistedSubstate::CurrentTimeRoundedToMinutes(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::StateClockResponse {
        current_minute: Some(to_api_clock_current_time_rounded_down_to_minutes_substate(
            &rounded_to_minutes_substate,
        )?),
    })
    .map(Json)
}
