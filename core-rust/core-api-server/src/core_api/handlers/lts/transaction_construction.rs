use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{ClockOffset, EpochManagerOffset, SubstateOffset, CLOCK, EPOCH_MANAGER};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId};
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_transaction_construction(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsTransactionConstructionRequest>,
) -> Result<Json<models::LtsTransactionConstructionResponse>, ResponseError<()>> {
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

    let clock_substate = {
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

    Ok(models::LtsTransactionConstructionResponse {
        current_epoch: to_api_epoch(&mapping_context, epoch_manager_substate.epoch)?,
        ledger_clock: Box::new(to_api_instant_from_safe_timestamp(
            clock_substate.current_time_rounded_to_minutes_ms,
        )?),
    })
    .map(Json)
}
