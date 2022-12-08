use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::{ClockOffset, GlobalAddress, SubstateOffset, CLOCK};
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_v0_state_clock(
    state: Extension<CoreApiState>,
) -> Result<Json<models::V0StateClockResponse>, RequestHandlingError> {
    core_api_read_handler(state, Json(()), handle_v0_state_clock_internal)
}

fn handle_v0_state_clock_internal(
    state_manager: &ActualStateManager,
    _request: (),
) -> Result<models::V0StateClockResponse, RequestHandlingError> {
    let clock = read_derefed_global_node_id(state_manager, GlobalAddress::System(CLOCK))?;
    let rounded_to_minutes_substate = {
        let substate_offset = SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes);
        let loaded_substate = read_known_substate(state_manager, clock, &substate_offset)?;
        let PersistedSubstate::CurrentTimeRoundedToMinutes(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::V0StateClockResponse {
        current_minute: Some(to_api_clock_current_time_rounded_down_to_minutes_substate(
            &rounded_to_minutes_substate,
        )?),
    })
}
