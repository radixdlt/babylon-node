use crate::core_api::*;
use state_manager::store::traits::QueryableProofStore;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_lts_stream_account_transaction_outcomes(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStreamAccountTransactionOutcomesRequest>,
) -> Result<Json<models::LtsStreamAccountTransactionOutcomesResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

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

    let state_manager = state.state_manager.read();
    let read_store = state_manager.store();

    let _max_state_version = read_store.max_state_version();

    Err(not_implemented("Endpoint not implemented yet"))
}
