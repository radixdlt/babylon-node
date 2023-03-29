use crate::core_api::*;
use state_manager::{jni::state_manager::ActualStateManager, store::traits::QueryableProofStore};

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_lts_stream_account_transactions_basic_outcomes(
    state: State<CoreApiState>,
    request: Json<models::LtsStreamAccountTransactionsBasicOutcomesRequest>,
) -> Result<Json<models::LtsStreamAccountTransactionsBasicOutcomesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_lts_stream_account_transactions_basic_outcomes_internal,
    )
}

fn handle_lts_stream_account_transactions_basic_outcomes_internal(
    state_manager: &ActualStateManager,
    request: models::LtsStreamAccountTransactionsBasicOutcomesRequest,
) -> Result<models::LtsStreamAccountTransactionsBasicOutcomesResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let _from_state_version: u64 = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: u64 = request
        .limit
        .try_into()
        .map_err(|_| client_error("limit cannot be negative"))?;

    if limit == 0 {
        return Err(client_error("limit must be positive"));
    }

    if limit > MAX_STREAM_COUNT_PER_REQUEST.into() {
        return Err(client_error(format!(
            "limit must <= {MAX_STREAM_COUNT_PER_REQUEST}"
        )));
    }

    let _max_state_version = state_manager.store().max_state_version();

    Err(not_implemented("Endpoint not implemented yet"))
}
