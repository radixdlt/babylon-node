use crate::core_api::{handlers::to_api_committed_transaction, *};
use state_manager::{
    jni::state_manager::ActualStateManager,
    store::traits::{QueryableProofStore, QueryableTransactionStore},
};

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_rc_stream_transactions_basic_outcomes(
    state: State<CoreApiState>,
    request: Json<models::RcStreamTransactionsBasicOutcomesRequest>,
) -> Result<Json<models::RcStreamTransactionsBasicOutcomesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_rc_stream_transactions_basic_outcomes_internal,
    )
}

fn handle_rc_stream_transactions_basic_outcomes_internal(
    state_manager: &ActualStateManager,
    request: models::RcStreamTransactionsBasicOutcomesRequest,
) -> Result<models::RcStreamTransactionsBasicOutcomesResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

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

    let max_state_version = state_manager.store().max_state_version();

    let txns = state_manager.store().get_committed_transaction_bundles(
        from_state_version,
        limit.try_into().expect("limit out of usize bounds"),
    );

    let mapping_context = MappingContext::new(&state_manager.network);

    let api_txns = txns
        .into_iter()
        .map(|(ledger_transaction, receipt, identifiers)| {
            Ok(to_api_committed_transaction(
                &mapping_context,
                ledger_transaction,
                receipt,
                identifiers,
            )?)
        })
        .collect::<Result<Vec<models::CommittedTransaction>, ResponseError<()>>>()?;

    let start_state_version = if api_txns.is_empty() {
        0
    } else {
        from_state_version
    };

    let count: i32 = {
        let transaction_count = api_txns.len();
        if transaction_count > MAX_STREAM_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    Ok(models::RcStreamTransactionsBasicOutcomesResponse {
        from_state_version: to_api_state_version(start_state_version)?,
        count,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        basic_outcomes: api_txns,
    })
}
