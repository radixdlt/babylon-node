use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_rc_stream_entity_transactions_basic_outcomes(
    state: State<CoreApiState>,
    request: Json<models::LtsStreamEntityTransactionsBasicOutcomesRequest>,
) -> Result<Json<models::LtsStreamEntityTransactionsBasicOutcomesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_rc_stream_entity_transactions_basic_outcomes_internal,
    )
}

fn handle_rc_stream_entity_transactions_basic_outcomes_internal(
    state_manager: &ActualStateManager,
    request: models::LtsStreamEntityTransactionsBasicOutcomesRequest,
) -> Result<models::LtsStreamEntityTransactionsBasicOutcomesResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    let _mapping_context = MappingContext::new(&state_manager.network);

    todo!();
}
