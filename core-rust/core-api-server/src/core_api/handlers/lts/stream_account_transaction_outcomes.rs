use crate::core_api::*;
use state_manager::store::traits::{
    extensions::IterableAccountChangeIndex, ConfigurableDatabase, QueryableProofStore,
    QueryableTransactionStore,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_lts_stream_account_transaction_outcomes(
    state: State<CoreApiState>,
    Json(request): Json<models::LtsStreamAccountTransactionOutcomesRequest>,
) -> Result<Json<models::LtsStreamAccountTransactionOutcomesResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error(
            "Only component addresses starting with account_ work with this endpoint.",
        ));
    }

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let account_address = extract_global_address(&extraction_context, &request.account_address)
        .map_err(|err| err.into_response_error("account_address"))?;

    let from_state_version: u64 = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: usize = request
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

    if !database.is_local_transaction_execution_index_enabled() {
        return Err(client_error(
            "This endpoint requires that the LocalTransactionExecutionIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config, wipe ledger and restart. \
            Please note the resync will take a while.",
        ));
    }

    if !database.is_account_change_index_enabled() {
        return Err(client_error(
            "This endpoint requires that the AccountChangeIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config and restart the node. \
            Please note the index catchup build will take some time.",
        ));
    }

    let max_state_version = database.max_state_version();

    let state_versions =
        database.get_state_versions_for_account_iter(account_address, from_state_version);

    let mut response = models::LtsStreamAccountTransactionOutcomesResponse {
        from_state_version: to_api_state_version(from_state_version)?,
        count: MAX_STREAM_COUNT_PER_REQUEST as i32, // placeholder to get a better size aproximation for the header
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        committed_transaction_outcomes: Vec::new(),
    };

    // Reserve enough for the "header" fields
    let mut current_total_size = response.get_json_size();
    for state_version in state_versions.take(limit) {
        let committed_transaction_outcome = to_api_lts_committed_transaction_outcome(
            &mapping_context,
            state_version,
            database
                .get_committed_local_transaction_receipt(state_version)
                .expect("Transaction receipt index corrupted"),
            database
                .get_committed_transaction_identifiers(state_version)
                .expect("Transaction identifiers index corrupted"),
        )?;

        let committed_transaction_size = committed_transaction_outcome.get_json_size();
        current_total_size += committed_transaction_size;

        response
            .committed_transaction_outcomes
            .push(committed_transaction_outcome);

        if current_total_size > CAP_STREAM_RESPONSE_WHEN_ABOVE_BYTES {
            break;
        }
    }
    response.count = response.committed_transaction_outcomes.len() as i32;

    Ok(response).map(Json)
}
