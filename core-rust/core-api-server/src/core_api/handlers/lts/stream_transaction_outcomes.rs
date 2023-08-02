use crate::core_api::*;
use state_manager::store::traits::{
    CommittedTransactionBundle, ConfigurableDatabase, IterableTransactionStore, QueryableProofStore,
};
use std::ops::Deref;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_stream_transaction_outcomes(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStreamTransactionOutcomesRequest>,
) -> Result<Json<models::LtsStreamTransactionOutcomesResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let from_state_version = extract_api_state_version(request.from_state_version)
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

    let limit = limit.try_into().expect("limit out of usize bounds");

    let database = state.radix_node.database.read();

    if !database.is_local_transaction_execution_index_enabled() {
        return Err(client_error(
            "This endpoint requires that the LocalTransactionExecutionIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config, wipe ledger and restart. \
            Please note the resync will take a while.",
        ));
    }

    let max_state_version = database.max_state_version();

    let mut response = models::LtsStreamTransactionOutcomesResponse {
        from_state_version: to_api_state_version(from_state_version)?,
        count: MAX_STREAM_COUNT_PER_REQUEST as i32, // placeholder to get a better size aproximation for the header
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        committed_transaction_outcomes: Vec::new(),
    };

    // Reserve enough for the "header" fields
    let mut current_total_size = response.get_json_size();
    let bundles = database
        .get_committed_transaction_bundle_iter(from_state_version)
        .take(limit);
    for bundle in bundles {
        let CommittedTransactionBundle {
            state_version,
            receipt,
            identifiers,
            ..
        } = bundle;
        let committed_transaction = to_api_lts_committed_transaction_outcome(
            database.deref(),
            &mapping_context,
            state_version,
            receipt,
            identifiers,
        )?;

        let committed_transaction_size = committed_transaction.get_json_size();
        current_total_size += committed_transaction_size;

        response
            .committed_transaction_outcomes
            .push(committed_transaction);

        if current_total_size > CAP_STREAM_RESPONSE_WHEN_ABOVE_BYTES {
            break;
        }
    }

    let count: i32 = {
        let transaction_count = response.committed_transaction_outcomes.len();
        if transaction_count > MAX_STREAM_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    response.count = count;

    Ok(response).map(Json)
}
