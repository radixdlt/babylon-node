use crate::core_api::*;
use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_transaction_construction(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsTransactionConstructionRequest>,
) -> Result<Json<models::LtsTransactionConstructionResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let database = state.state_manager.database.read_current();

    let consensus_manager_substate =
        read_mandatory_main_field_substate::<ConsensusManagerStateFieldPayload>(
            database.deref(),
            CONSENSUS_MANAGER.as_node_id(),
            &ConsensusManagerField::State.into(),
        )?
        .into_payload()
        .into_latest();

    let timestamp_substate =
        read_mandatory_main_field_substate::<ConsensusManagerProposerMilliTimestampFieldPayload>(
            database.deref(),
            CONSENSUS_MANAGER.as_node_id(),
            &ConsensusManagerField::ProposerMilliTimestamp.into(),
        )?
        .into_payload()
        .into_latest();

    Ok(models::LtsTransactionConstructionResponse {
        current_epoch: to_api_epoch(&mapping_context, consensus_manager_substate.epoch)?,
        ledger_clock: Box::new(to_api_instant_from_safe_timestamp(
            timestamp_substate.epoch_milli,
        )?),
    })
    .map(Json)
}
