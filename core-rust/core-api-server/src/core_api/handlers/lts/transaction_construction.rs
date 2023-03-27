use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{ClockOffset, EpochManagerOffset, SubstateOffset, CLOCK, EPOCH_MANAGER};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId};
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_lts_transaction_construction(
    state: State<CoreApiState>,
    request: Json<models::LtsTransactionConstructionRequest>,
) -> Result<Json<models::LtsTransactionConstructionResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_lts_transaction_construction_internal)
}

fn handle_lts_transaction_construction_internal(
    state_manager: &ActualStateManager,
    request: models::LtsTransactionConstructionRequest,
) -> Result<models::LtsTransactionConstructionResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    let mapping_context = MappingContext::new(&state_manager.network);

    let epoch_manager_substate = {
        let substate_offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let loaded_substate = read_mandatory_substate(
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

    let clock_substate = {
        let substate_offset = SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes);
        let loaded_substate = read_mandatory_substate(
            state_manager,
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
}
