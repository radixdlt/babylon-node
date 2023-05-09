use crate::core_api::*;
use radix_engine::blueprints::clock::ClockSubstate;
use radix_engine::blueprints::epoch_manager::EpochManagerSubstate;
use radix_engine::types::*;
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_transaction_construction(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsTransactionConstructionRequest>,
) -> Result<Json<models::LtsTransactionConstructionResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let database = state.database.read();

    let epoch_manager_substate: EpochManagerSubstate = read_mandatory_main_field_substate(
        database.deref(),
        EPOCH_MANAGER.as_node_id(),
        &EpochManagerField::EpochManager.into(),
    )?;

    let clock_substate: ClockSubstate = read_mandatory_main_field_substate(
        database.deref(),
        CLOCK.as_node_id(),
        // TODO - change this to a better resolution of time when we have it
        &ClockField::CurrentTimeRoundedToMinutes.into(),
    )?;

    Ok(models::LtsTransactionConstructionResponse {
        current_epoch: to_api_epoch(&mapping_context, epoch_manager_substate.epoch)?,
        ledger_clock: Box::new(to_api_instant_from_safe_timestamp(
            clock_substate.current_time_rounded_to_minutes_ms,
        )?),
    })
    .map(Json)
}
