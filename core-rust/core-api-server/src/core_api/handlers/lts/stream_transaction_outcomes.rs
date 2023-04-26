use crate::core_api::*;
use state_manager::store::traits::{QueryableProofStore, QueryableTransactionStore};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_stream_transaction_outcomes(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStreamTransactionOutcomesRequest>,
) -> Result<Json<models::LtsStreamTransactionOutcomesResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let from_state_version: u64 = extract_api_state_version(request.from_state_version)
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

    let database = state.database.read();

    let max_state_version = database.max_state_version();

    let txns = database.get_committed_transaction_bundles(
        from_state_version,
        limit.try_into().expect("limit out of usize bounds"),
    );

    let committed_transaction_outcomes = txns
        .into_iter()
        .map(|(_ledger_transaction, receipt, identifiers)| {
            Ok(to_api_lts_committed_transaction_outcome(
                &mapping_context,
                receipt,
                identifiers,
            )?)
        })
        .collect::<Result<Vec<models::LtsCommittedTransactionOutcome>, ResponseError<()>>>()?;

    let count: i32 = {
        let transaction_count = committed_transaction_outcomes.len();
        if transaction_count > MAX_STREAM_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    Ok(models::LtsStreamTransactionOutcomesResponse {
        from_state_version: to_api_state_version(from_state_version)?,
        count,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        committed_transaction_outcomes,
    })
    .map(Json)
}
