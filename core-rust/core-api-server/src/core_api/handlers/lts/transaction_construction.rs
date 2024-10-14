use crate::prelude::*;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_transaction_construction(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsTransactionConstructionRequest>,
) -> Result<Json<models::LtsTransactionConstructionResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let database = state.state_manager.database.snapshot();

    let consensus_manager_substate =
        read_mandatory_main_field_substate::<ConsensusManagerStateFieldPayload>(
            database.deref(),
            CONSENSUS_MANAGER.as_node_id(),
            &ConsensusManagerField::State.into(),
        )?
        .into_payload()
        .fully_update_and_into_latest_version();

    let timestamp_substate =
        read_mandatory_main_field_substate::<ConsensusManagerProposerMilliTimestampFieldPayload>(
            database.deref(),
            CONSENSUS_MANAGER.as_node_id(),
            &ConsensusManagerField::ProposerMilliTimestamp.into(),
        )?
        .into_payload()
        .fully_update_and_into_latest_version();

    Ok(Json(models::LtsTransactionConstructionResponse {
        current_epoch: to_api_epoch(&mapping_context, consensus_manager_substate.epoch)?,
        ledger_clock: Box::new(to_api_clamped_instant_from_epoch_milli(
            timestamp_substate.epoch_milli,
        )?),
    }))
}
